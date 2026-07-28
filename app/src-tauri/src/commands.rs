//! Tauri commands exposing the interruption-stack operations to the frontend.
//!
//! Every mutating command follows the same pattern (`apply_transition`): dry-run
//! the proposed transition against a clone of the in-memory stack FIRST — a
//! transition that would fail its precondition (e.g. Return with an empty
//! stack) is rejected before anything is written to the durable log. Only once
//! the dry-run succeeds does the real transition get appended (fsync'd) and
//! then applied to the real in-memory stack. This is the concrete mechanism
//! behind "never durably log a transition that couldn't actually happen."

use crate::export;
use crate::export_settings::{mutate_export_settings, ExportSettings, ExportSettingsState};
use crate::hotkeys::{apply_remap, HotkeyState};
use crate::model::{StackFrame, TaskTemplate, TimeBlock, TransitionPayload};
use crate::settings::HotkeyBindings;
use crate::stack::InterruptionStack;
use crate::state::AppState;
use crate::templates::{mutate_templates, TemplateState};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Every window listens for this to stay in sync — emitted after every
/// successful mutation, from the command boundary and from the background
/// heartbeat/gap-recovery threads alike. This (not polling) is what makes the
/// mini widget and dashboard agree within milliseconds, by construction.
pub const STATE_CHANGED_EVENT: &str = "state-changed";

#[derive(Debug, Serialize, Clone)]
pub struct StackView {
    pub active: Option<TimeBlock>,
    pub stack: Vec<StackFrame>,
    pub closed: Vec<TimeBlock>,
}

impl From<&InterruptionStack> for StackView {
    fn from(s: &InterruptionStack) -> Self {
        StackView { active: s.active.clone(), stack: s.stack.clone(), closed: s.closed.clone() }
    }
}

/// Broadcast the current state to every window. Best-effort: a failed emit
/// (e.g. no windows currently exist) must never fail the underlying transition,
/// which has already durably committed by the time this is called.
pub fn emit_state_changed(app: &AppHandle, view: &StackView) {
    let _ = app.emit(STATE_CHANGED_EVENT, view);
}

/// The single entry point every mutating command goes through. `build_payload`
/// receives a read-only view of the current stack so callers like `switch` can
/// decide their exact transition type (Start vs. Switch) under the same lock
/// that will perform the write — no separate peek-then-act race.
///
/// Deliberately takes no `AppHandle` — this stays fully unit-testable without a
/// running Tauri app. Callers that have a handle (the `#[tauri::command]`
/// wrappers below, and the background heartbeat/gap-recovery threads) emit
/// `state-changed` themselves right after calling this.
pub fn apply_transition(
    state: &AppState,
    build_payload: impl FnOnce(&InterruptionStack) -> TransitionPayload,
) -> Result<StackView, String> {
    let mut inner = state.inner.lock().map_err(|_| "state lock poisoned".to_string())?;
    let payload = build_payload(&inner.stack);

    // Dry-run: reject before writing anything durable if this would fail.
    let mut check = inner.stack.clone();
    check
        .apply(&payload, chrono::Utc::now())
        .map_err(|e| e.to_string())?;

    let record = inner.writer.append(payload).map_err(|e| e.to_string())?;
    inner
        .stack
        .apply(&record.payload, record.timestamp)
        .map_err(|e| format!("internal inconsistency after a validated dry-run: {e}"))?;
    inner.last_activity_at = record.timestamp;

    Ok(StackView::from(&inner.stack))
}

/// A UTC instant marking the start of "today" in the host's local timezone —
/// the boundary `InterruptionStack::next_default_name` counts against, so its
/// "Anchor N" numbering resets once per real calendar day rather than every
/// 24 hours from an arbitrary point. Falls back to the current instant on the
/// (very rare) DST-transition edge case where local midnight is ambiguous or
/// doesn't exist — at worst under-counts today's auto-named tasks by one,
/// never a correctness issue for anything durable.
fn today_start_utc() -> DateTime<Utc> {
    let local_now = chrono::Local::now();
    let midnight = local_now.date_naive().and_hms_opt(0, 0, 0).expect("midnight is always a valid time");
    match midnight.and_local_timezone(chrono::Local) {
        chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
        chrono::LocalResult::Ambiguous(dt, _) => dt.with_timezone(&Utc),
        chrono::LocalResult::None => local_now.with_timezone(&Utc),
    }
}

/// An empty/whitespace-only name means "start this without naming it" —
/// covers both the Switch/Interrupt hotkeys (which no longer require typing a
/// name first) and the dashboard's Switch/Interrupt buttons when clicked with
/// nothing typed. A real "Anchor N" name is generated and durably logged;
/// nothing blank ever reaches the log or an export.
fn name_or_default(name: String, stack: &InterruptionStack) -> String {
    if name.trim().is_empty() {
        stack.next_default_name(today_start_utc())
    } else {
        name
    }
}

#[tauri::command]
pub fn switch(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |stack| {
        let name = name_or_default(name, stack);
        if stack.active.is_none() {
            TransitionPayload::Start { name, project, client }
        } else {
            TransitionPayload::Switch { name, project, client }
        }
    })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn interrupt(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |stack| TransitionPayload::Interrupt {
        name: name_or_default(name, stack),
        project,
        client,
    })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

/// Renames the currently active task in place — no new Time Block, no stack
/// effect, start time untouched. Lets a task started unnamed (or under an
/// existing template/past name) be corrected or made more specific while
/// it's still running.
#[tauri::command]
pub fn rename_active(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::Rename { name, project, client })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn return_previous(app: AppHandle, state: State<AppState>) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::ReturnPrevious)?;
    emit_state_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn return_original(app: AppHandle, state: State<AppState>) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::ReturnOriginal)?;
    emit_state_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn complete(app: AppHandle, state: State<AppState>) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::Complete)?;
    emit_state_changed(&app, &view);
    Ok(view)
}

#[tauri::command]
pub fn get_state(state: State<AppState>) -> Result<StackView, String> {
    let inner = state.inner.lock().map_err(|_| "state lock poisoned".to_string())?;
    Ok(StackView::from(&inner.stack))
}

/// Templates are an entirely separate slice from the interruption stack — no
/// transition log, no dry-run (CRUD here is unconditional), own event so
/// listeners never have to guess which part of the app state changed.
pub const TEMPLATES_CHANGED_EVENT: &str = "templates-changed";

pub fn emit_templates_changed(app: &AppHandle, templates: &[TaskTemplate]) {
    let _ = app.emit(TEMPLATES_CHANGED_EVENT, templates);
}

#[tauri::command]
pub fn create_template(
    app: AppHandle,
    templates: State<TemplateState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<TaskTemplate, String> {
    let (created, list) = mutate_templates(&templates, |store| Ok(store.create(name, project, client)))?;
    emit_templates_changed(&app, &list);
    Ok(created)
}

#[tauri::command]
pub fn update_template(
    app: AppHandle,
    templates: State<TemplateState>,
    id: Uuid,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<TaskTemplate, String> {
    let (updated, list) = mutate_templates(&templates, |store| store.update(id, name, project, client))?;
    emit_templates_changed(&app, &list);
    Ok(updated)
}

#[tauri::command]
pub fn delete_template(app: AppHandle, templates: State<TemplateState>, id: Uuid) -> Result<(), String> {
    let (_, list) = mutate_templates(&templates, |store| store.delete(id))?;
    emit_templates_changed(&app, &list);
    Ok(())
}

#[tauri::command]
pub fn list_templates(templates: State<TemplateState>) -> Result<Vec<TaskTemplate>, String> {
    let store = templates.inner.lock().map_err(|_| "template store lock poisoned".to_string())?;
    Ok(store.list().to_vec())
}

/// Export is a read-only view over the current in-memory stack (itself a
/// faithful materialization of the durable log via replay) — no separate
/// re-read of the log file, and no `.apply()` call, so the durable timeline
/// can never be touched by exporting.
#[tauri::command]
pub fn get_export_settings(settings: State<ExportSettingsState>) -> Result<ExportSettings, String> {
    let inner = settings.inner.lock().map_err(|_| "export settings lock poisoned".to_string())?;
    Ok(inner.clone())
}

#[tauri::command]
pub fn update_export_settings(
    settings: State<ExportSettingsState>,
    rounding_enabled: bool,
    rounding_interval_minutes: u32,
) -> Result<ExportSettings, String> {
    mutate_export_settings(&settings, |s| {
        s.rounding_enabled = rounding_enabled;
        s.rounding_interval_minutes = rounding_interval_minutes;
    })
}

fn rounding_interval(rounding_enabled: bool, rounding_interval_minutes: u32) -> Option<u32> {
    rounding_enabled.then_some(rounding_interval_minutes)
}

/// Reads the current in-memory stack under its lock and filters to the given
/// range. Takes `&AppState` (not `State<AppState>`) so it's directly callable
/// from unit tests without a running Tauri app — the `#[tauri::command]`
/// wrappers below just pass their `State` through (it derefs to `&AppState`).
fn export_blocks_in_range(state: &AppState, range_start: DateTime<Utc>, range_end: DateTime<Utc>) -> Result<Vec<TimeBlock>, String> {
    let inner = state.inner.lock().map_err(|_| "state lock poisoned".to_string())?;
    Ok(export::blocks_in_range(&inner.stack.closed, inner.stack.active.as_ref(), range_start, range_end, chrono::Utc::now()))
}

#[tauri::command]
pub fn export_xlsx(
    state: State<AppState>,
    path: String,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
    rounding_enabled: bool,
    rounding_interval_minutes: u32,
) -> Result<(), String> {
    let blocks = export_blocks_in_range(&state, range_start, range_end)?;
    let rows = export::xlsx_rows(&blocks, rounding_interval(rounding_enabled, rounding_interval_minutes));
    export::write_xlsx(&rows, std::path::Path::new(&path))
}

#[tauri::command]
pub fn export_json(
    state: State<AppState>,
    path: String,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
    rounding_enabled: bool,
    rounding_interval_minutes: u32,
) -> Result<(), String> {
    let blocks = export_blocks_in_range(&state, range_start, range_end)?;
    let payload = export::json_export(&blocks, rounding_interval(rounding_enabled, rounding_interval_minutes));
    let json = serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

/// Reports the last known-good bindings — what's persisted and (barring an
/// OS-level failure the user hasn't yet been asked to resolve) what's
/// actually registered. This is what the Settings tab reads to populate its
/// remap form.
#[tauri::command]
pub fn get_hotkey_bindings(state: State<HotkeyState>) -> Result<HotkeyBindings, String> {
    let bindings = state.bindings.lock().map_err(|_| "hotkey state lock poisoned".to_string())?;
    Ok(bindings.clone())
}

/// Atomically remaps all five hotkeys: either all five new accelerators
/// register and persist, or none of them do and the previous bindings stay
/// live — see `hotkeys::apply_remap`.
#[tauri::command]
pub fn update_hotkey_bindings(
    app: AppHandle,
    state: State<HotkeyState>,
    switch: String,
    interrupt: String,
    return_previous: String,
    return_original: String,
    complete: String,
) -> Result<HotkeyBindings, String> {
    let new_bindings = HotkeyBindings { switch, interrupt, return_previous, return_original, complete };
    apply_remap(&app, &state, new_bindings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_return_previous_on_empty_stack_without_writing_to_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, report) = AppState::init(&path).unwrap();
        assert!(!report.torn_line_discarded);

        apply_transition(&state, |_| TransitionPayload::Start {
            name: "A".into(),
            project: None,
            client: None,
        })
        .unwrap();

        let err = apply_transition(&state, |_| TransitionPayload::ReturnPrevious).unwrap_err();
        assert!(err.contains("empty"));

        // The rejected transition must not have been written: the log should
        // contain exactly one line (the Start), not two.
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 1, "a rejected transition must never be written to the durable log");
    }

    /// The integration test required by the implementation plan: drive a real
    /// file-backed AppState through a sequence ending with something explicitly
    /// completed (not left active), drop it, replay from the same file, and
    /// assert the reconstructed *history* matches the pre-drop state exactly.
    #[test]
    fn restart_restores_full_history_when_nothing_was_left_active() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");

        let pre_drop_view = {
            let (state, report) = AppState::init(&path).unwrap();
            assert!(!report.torn_line_discarded);

            apply_transition(&state, |_| TransitionPayload::Start {
                name: "A".into(),
                project: Some("Acme".into()),
                client: None,
            })
            .unwrap();
            apply_transition(&state, |_| TransitionPayload::Interrupt {
                name: "B".into(),
                project: None,
                client: None,
            })
            .unwrap();
            apply_transition(&state, |_| TransitionPayload::Interrupt {
                name: "C".into(),
                project: None,
                client: None,
            })
            .unwrap();
            apply_transition(&state, |_| TransitionPayload::ReturnPrevious).unwrap(); // back to B, stack=[A]
            apply_transition(&state, |_| TransitionPayload::ReturnPrevious).unwrap(); // back to A, stack=[]
            apply_transition(&state, |_| TransitionPayload::Complete).unwrap()
            // `state` (and its open file handle) is dropped at the end of this
            // block, with nothing left active — so restart should reconstruct
            // history exactly, with no gap recovery triggered.
        };

        let (restarted, report) = AppState::init(&path).unwrap();
        assert!(!report.torn_line_discarded);
        assert!(!report.startup_gap_recovered, "nothing was left active, so no gap should be detected");
        let post_restart_view = {
            let inner = restarted.inner.lock().unwrap();
            StackView::from(&inner.stack)
        };

        assert!(post_restart_view.active.is_none());
        assert_eq!(pre_drop_view.closed.len(), post_restart_view.closed.len());
        // Time Block IDs are freshly random per `TimeBlock::new()` call, so
        // replay naturally produces different IDs than the original run — by
        // design, nothing relies on stable IDs across restarts (Time Blocks are
        // independent flat entries, aggregated by name/project/client, not ID).
        // What must match is everything that actually carries meaning.
        for (pre, post) in pre_drop_view.closed.iter().zip(post_restart_view.closed.iter()) {
            assert_eq!(pre.name, post.name);
            assert_eq!(pre.project, post.project);
            assert_eq!(pre.client, post.client);
            assert_eq!(pre.start, post.start);
            assert_eq!(pre.completion_reason, post.completion_reason);
            assert_eq!(pre.end, post.end);
        }
    }

    /// The counterpart case: something IS left active across a restart —
    /// regardless of why (crash or just closing the app) — and must come back
    /// as `recovered-gap`, not silently resumed as if nothing happened.
    #[test]
    fn restart_with_something_left_active_recovers_it_as_gap_not_resumed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");

        {
            let (state, _report) = AppState::init(&path).unwrap();
            apply_transition(&state, |_| TransitionPayload::Start {
                name: "A".into(),
                project: None,
                client: None,
            })
            .unwrap();
            // Dropped here with "A" still active — no Complete/Switch/Return.
        }

        let (restarted, report) = AppState::init(&path).unwrap();
        assert!(report.startup_gap_recovered);
        let inner = restarted.inner.lock().unwrap();
        assert!(inner.stack.active.is_none());
        let a = inner.stack.closed.iter().find(|b| b.name == "A").unwrap();
        assert_eq!(a.completion_reason, Some(crate::model::CompletionReason::RecoveredGap));
    }

    #[test]
    fn create_update_delete_template_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let templates_state = TemplateState::init(dir.path().join("templates.json"));

        let (created, list) =
            mutate_templates(&templates_state, |store| Ok(store.create("Standup".into(), Some("Acme".into()), None)))
                .unwrap();
        assert_eq!(list.len(), 1);

        let (updated, list) = mutate_templates(&templates_state, |store| {
            store.update(created.id, "Standup".into(), Some("Globex".into()), None)
        })
        .unwrap();
        assert_eq!(updated.project, Some("Globex".to_string()));
        assert_eq!(list.len(), 1);

        let (_, list) = mutate_templates(&templates_state, |store| store.delete(created.id)).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn templates_persist_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("templates.json");

        {
            let templates_state = TemplateState::init(&path);
            mutate_templates(&templates_state, |store| Ok(store.create("Standup".into(), None, None))).unwrap();
        }

        let reloaded = TemplateState::init(&path);
        assert_eq!(reloaded.inner.lock().unwrap().list().len(), 1);
    }

    /// The explicit Acceptance-Criteria-proving test: editing (or deleting) a
    /// template must never retroactively affect a Time Block already recorded
    /// from it. Proves the two systems are decoupled at the DATA level, not
    /// just in the UI — a future regression (e.g. adding a `template_id` field
    /// to `TimeBlock`) would break this test.
    #[test]
    fn editing_and_deleting_a_template_does_not_change_an_already_recorded_time_block() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("log.jsonl");
        let templates_path = dir.path().join("templates.json");

        let (app_state, _report) = AppState::init(&log_path).unwrap();
        let templates_state = TemplateState::init(&templates_path);

        let (template, _) = mutate_templates(&templates_state, |store| {
            Ok(store.create("Standup".into(), Some("Acme".into()), None))
        })
        .unwrap();

        // Stands in for "the user selected the template via autocomplete, then
        // pressed Switch" — the frontend only ever sends plain strings, never a
        // template reference, so this is a faithful simulation.
        apply_transition(&app_state, |_| TransitionPayload::Start {
            name: template.name.clone(),
            project: template.project.clone(),
            client: None,
        })
        .unwrap();

        // Now edit the template's project.
        mutate_templates(&templates_state, |store| {
            store.update(template.id, "Standup".into(), Some("Globex".into()), None)
        })
        .unwrap();

        let inner = app_state.inner.lock().unwrap();
        let recorded = inner.stack.active.as_ref().unwrap();
        assert_eq!(recorded.project, Some("Acme".to_string()), "already-recorded Time Block must keep its original value");
        drop(inner);

        // Deleting the template afterward must not touch the Time Block either
        // — trivially true since TimeBlock has no template reference at all,
        // but asserted explicitly as a regression guard.
        mutate_templates(&templates_state, |store| store.delete(template.id)).unwrap();
        let inner = app_state.inner.lock().unwrap();
        assert_eq!(inner.stack.active.as_ref().unwrap().project, Some("Acme".to_string()));
    }

    /// The explicit AC-proving test for Export: exporting (both XLSX and JSON)
    /// must never touch the durable transition log — not "the code doesn't
    /// call write on it" but an actual byte-for-byte comparison of the log
    /// file before and after.
    #[test]
    fn exporting_leaves_the_underlying_transition_log_byte_for_byte_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("log.jsonl");

        let (app_state, _report) = AppState::init(&log_path).unwrap();
        apply_transition(&app_state, |_| TransitionPayload::Start {
            name: "A".into(),
            project: Some("Acme".into()),
            client: None,
        })
        .unwrap();
        apply_transition(&app_state, |_| TransitionPayload::Interrupt {
            name: "B".into(),
            project: None,
            client: None,
        })
        .unwrap();
        apply_transition(&app_state, |_| TransitionPayload::ReturnPrevious).unwrap();
        apply_transition(&app_state, |_| TransitionPayload::Complete).unwrap();

        let before = std::fs::read(&log_path).unwrap();

        let range_start = chrono::Utc::now() - chrono::Duration::days(1);
        let range_end = chrono::Utc::now() + chrono::Duration::days(1);
        let blocks = export_blocks_in_range(&app_state, range_start, range_end).unwrap();
        // Start A, Interrupt B, ReturnPrevious (resolves paused A as explicit
        // and starts a brand-new A2 block — resuming never reopens the
        // original), Complete (closes A2) => three independent closed blocks.
        assert_eq!(blocks.len(), 3, "the paused-then-resolved A, the interrupting B, and the resumed A2 must all be in range");

        let xlsx_path = dir.path().join("out.xlsx");
        let rows = export::xlsx_rows(&blocks, Some(15));
        export::write_xlsx(&rows, &xlsx_path).unwrap();

        let json_path = dir.path().join("out.json");
        let payload = export::json_export(&blocks, None);
        std::fs::write(&json_path, serde_json::to_string_pretty(&payload).unwrap()).unwrap();

        let after = std::fs::read(&log_path).unwrap();
        assert_eq!(before, after, "exporting must never mutate the durable transition log");
        assert!(xlsx_path.exists());
        assert!(json_path.exists());
    }
}
