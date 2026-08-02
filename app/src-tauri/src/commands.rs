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
use crate::model::{DerivedInterruptionStatus, StackFrame, TaskTemplate, TimeBlock, TransitionPayload};
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

/// A closed Time Block as the UI sees it: the block, plus its canonical
/// `DerivedInterruptionStatus`.
///
/// The status is attached HERE rather than left for the frontend to compute,
/// because ADR 0005 makes that projection mandatory and singular — an absent
/// `interruption_outcome` means *never interrupted* OR *interrupted and
/// unresolved*, and only the live stack tells them apart. Shipping the answer
/// removes any opportunity for a view to invent its own.
#[derive(Debug, Serialize, Clone)]
pub struct ClosedBlockView {
    #[serde(flatten)]
    pub block: TimeBlock,
    pub derived_interruption_status: DerivedInterruptionStatus,
}

#[derive(Debug, Serialize, Clone)]
pub struct StackView {
    pub active: Option<TimeBlock>,
    pub stack: Vec<StackFrame>,
    pub closed: Vec<ClosedBlockView>,
}

impl From<&InterruptionStack> for StackView {
    fn from(s: &InterruptionStack) -> Self {
        StackView {
            active: s.active.clone(),
            stack: s.stack.clone(),
            closed: s
                .closed
                .iter()
                .map(|block| ClosedBlockView {
                    derived_interruption_status: s.derived_status(block),
                    block: block.clone(),
                })
                .collect(),
        }
    }
}

/// Broadcast the current state to every window. Best-effort: a failed emit
/// (e.g. no windows currently exist) must never fail the underlying transition,
/// which has already durably committed by the time this is called.
pub fn emit_state_changed(app: &AppHandle, view: &StackView) {
    let _ = app.emit(STATE_CHANGED_EVENT, view);
}

/// The single entry point every mutating command goes through. `build_payload`
/// receives a read-only view of the current stack so callers can derive payload
/// details from it under the same lock that performs the write — `name_or_default`
/// needs it to compute the next "Anchor N". It is NOT for choosing between
/// transition types: `switch` used to pick Start-vs-Switch here, which ADR 0005
/// rejected. That choice now lives in the presentation layer.
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
        .apply(&payload, chrono::Utc::now(), inner.writer.next_seq())
        .map_err(|e| e.to_string())?;

    let record = inner.writer.append(payload).map_err(|e| e.to_string())?;
    inner
        .stack
        .apply(&record.payload, record.timestamp, record.seq)
        .map_err(|e| format!("internal inconsistency after a validated dry-run: {e}"))?;
    inner.last_activity_at = record.timestamp;

    // ADR 0004's threshold arm. Runs after the transition is durable and
    // applied, so a compaction failure can never affect whether the user's
    // action succeeded.
    inner.compaction.record(&record.payload);
    let view = StackView::from(&inner.stack);
    crate::state::compact_if_due(&mut inner);

    Ok(view)
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
    let view = apply_transition(&state, |stack| TransitionPayload::Switch {
        name: name_or_default(name, stack),
        project,
        client,
    })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

/// Begin tracking when nothing is active.
///
/// Extracted out of `switch` (2026-07-29). `switch` used to branch on
/// `stack.active.is_none()` and emit `Start` instead — one command describing
/// two transitions depending on state, which [ADR 0005] rejected: each of the
/// five actions has one meaning and its own precondition. `Start` requires
/// nothing active (`AlreadyActive` otherwise); `Switch` requires something
/// active (`NoActiveTask` otherwise).
///
/// **Choosing between them is a presentation concern, not a domain one.** The
/// dashboard shows Start or Switch based on `view.active`; the capture hotkey
/// peeks `AppState::has_active` and dispatches. Neither is inside a transition.
///
/// [ADR 0005]: ../../../docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md
#[tauri::command]
pub fn start(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |stack| TransitionPayload::Start {
        name: name_or_default(name, stack),
        project,
        client,
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

/// Creates a Time Block for work that happened but was never captured.
///
/// The first command to carry author-chosen boundaries rather than deriving
/// them from when it was invoked. Thin like the rest: no clamping, no snapping,
/// no time arithmetic. `start` and `end` are passed through exactly as given,
/// and the domain decides whether that span is legal — it rejects overlaps, a
/// non-positive duration, and any end in the future, against the transition's
/// own timestamp rather than a clock read here.
#[tauri::command]
pub fn add_block(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::Add { name, project, client, start, end })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

/// Translates a block to a new start, preserving its duration exactly.
///
/// Takes only `start` — the same shape as the transition, and for the same
/// reason: the end is the block's own duration applied at the new position, so
/// neither this command nor its caller can express a duration change. A caller
/// that wants one is asking for `resize_block`.
#[tauri::command]
pub fn move_block(
    app: AppHandle,
    state: State<AppState>,
    target: Uuid,
    start: DateTime<Utc>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::Move { target, start })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

/// Reshapes a block's span — the mechanism risks R9 and R4 have been promised
/// since 2026-07-23 and never had.
///
/// Passes both boundaries through untouched. Whether the end actually changed,
/// and therefore whether `EndDetermination` becomes `UserDetermined`, is the
/// domain's determination and not this layer's to anticipate.
#[tauri::command]
pub fn resize_block(
    app: AppHandle,
    state: State<AppState>,
    target: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::Resize { target, start, end })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

/// Corrects the identity of a Time Block that has **already finished** — the
/// History View's row action, and the historical counterpart to
/// `rename_active`.
///
/// Deliberately a thin adapter: it collects `target` and the new fields and
/// hands them to the state machine. Every rule — the block must exist, must not
/// be the active one, and its frame's copy must be updated with it — lives in
/// `InterruptionStack::apply`, so the UI cannot drift from replay. A failure
/// surfaces the domain's own error rather than being pre-empted here.
#[tauri::command]
pub fn edit_identity(
    app: AppHandle,
    state: State<AppState>,
    target: Uuid,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::EditIdentity {
        target,
        name,
        project,
        client,
    })?;
    emit_state_changed(&app, &view);
    Ok(view)
}

/// Removes a Time Block from the timeline — the History View's other row
/// action.
///
/// **Confirmation is the frontend's job and is not optional** (accepted design:
/// Delete is confirmed and MVP has no undo). It is a presentation concern, so
/// it lives there; what lives here is the guarantee that reaching this function
/// writes exactly one transition. The domain still rejects a delete that would
/// orphan an open interruption frame, so a UI that skipped its confirmation
/// could not corrupt anything — it would only be rude.
#[tauri::command]
pub fn delete_block(app: AppHandle, state: State<AppState>, target: Uuid) -> Result<StackView, String> {
    let view = apply_transition(&state, |_| TransitionPayload::Delete { target })?;
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

    fn log_lines(path: &std::path::Path) -> usize {
        std::fs::read_to_string(path).map(|s| s.lines().count()).unwrap_or(0)
    }

    /// **The layering invariant: a History View command is a pure adapter over
    /// the append-only log.** Every successful one appends exactly one
    /// transition — no extra mutations, no bypassing the transition system, no
    /// batching. If a future refactor lets UI concerns reach around `apply()`,
    /// this fails immediately rather than at the next replay.
    ///
    /// The failed cases matter as much as the successful ones: a rejected
    /// command must write *nothing*, or the log accumulates transitions that
    /// never happened.
    #[test]
    fn every_reconstruction_command_appends_exactly_one_transition_and_a_rejected_one_appends_none() {
        use chrono::Duration;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, _) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();

        let now = Utc::now();
        let ago = |mins: i64| now - Duration::minutes(mins);

        // A finished block to act on, plus a free span well before it.
        apply_transition(&state, |_| TransitionPayload::Start { name: "work".into(), project: None, client: None })
            .unwrap();
        let view = apply_transition(&state, |_| TransitionPayload::Complete).unwrap();
        let target = view.closed[0].block.id;

        // Each of the five, checked one at a time so a failure names the culprit.
        let expect_one = |label: &str, payload: TransitionPayload| {
            let before = log_lines(&path);
            let result = apply_transition(&state, |_| payload);
            assert!(result.is_ok(), "{label} should succeed: {:?}", result.err());
            assert_eq!(log_lines(&path), before + 1, "{label} must append exactly one line");
        };

        expect_one(
            "Add",
            TransitionPayload::Add {
                name: "forgotten".into(),
                project: None,
                client: None,
                start: ago(120),
                end: ago(90),
            },
        );
        let added = {
            let inner = state.inner.lock().unwrap();
            inner.stack.closed.iter().find(|b| b.name == "forgotten").unwrap().id
        };
        expect_one("Move", TransitionPayload::Move { target: added, start: ago(240) });
        expect_one("Resize", TransitionPayload::Resize { target: added, start: ago(240), end: ago(200) });
        expect_one(
            "EditIdentity",
            TransitionPayload::EditIdentity { target, name: "corrected".into(), project: None, client: None },
        );
        expect_one("Delete", TransitionPayload::Delete { target });

        // And the rejection half, which matters as much: a command the domain
        // refuses must leave the log untouched, or it accumulates transitions
        // for things that never happened.
        let unknown = Uuid::nil();
        let rejections: Vec<(&str, TransitionPayload)> = vec![
            ("Add overlapping", TransitionPayload::Add {
                name: "clash".into(),
                project: None,
                client: None,
                start: ago(230),
                end: ago(210),
            }),
            ("Add ending in the future", TransitionPayload::Add {
                name: "later".into(),
                project: None,
                client: None,
                start: ago(10),
                end: now + Duration::minutes(10),
            }),
            ("Move unknown", TransitionPayload::Move { target: unknown, start: ago(600) }),
            ("Resize unknown", TransitionPayload::Resize { target: unknown, start: ago(600), end: ago(500) }),
            ("EditIdentity unknown", TransitionPayload::EditIdentity {
                target: unknown,
                name: "x".into(),
                project: None,
                client: None,
            }),
            ("Delete unknown", TransitionPayload::Delete { target: unknown }),
        ];
        for (label, payload) in rejections {
            let before = log_lines(&path);
            assert!(apply_transition(&state, |_| payload).is_err(), "{label} should be rejected");
            assert_eq!(log_lines(&path), before, "{label} must append nothing");
        }
    }

    /// One per command: invoke → append → replay → final state. Deliberately
    /// not a re-run of the domain tests — what this checks is that the command
    /// layer writes a payload replay can reconstruct the same state from, with
    /// nothing held only in memory.
    #[test]
    fn each_reconstruction_command_survives_a_replay_with_identical_state() {
        use chrono::Duration;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");
        let now = Utc::now();
        let ago = |mins: i64| now - Duration::minutes(mins);

        let (added_id, kept_id) = {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();

            apply_transition(&state, |_| TransitionPayload::Add {
                name: "reconstructed".into(),
                project: None,
                client: None,
                start: ago(300),
                end: ago(240),
            })
            .unwrap();
            apply_transition(&state, |_| TransitionPayload::Add {
                name: "kept".into(),
                project: None,
                client: None,
                start: ago(100),
                end: ago(60),
            })
            .unwrap();

            let (added_id, kept_id) = {
                let inner = state.inner.lock().unwrap();
                let f = |n: &str| inner.stack.closed.iter().find(|b| b.name == n).unwrap().id;
                (f("reconstructed"), f("kept"))
            };

            apply_transition(&state, |_| TransitionPayload::Move { target: added_id, start: ago(400) }).unwrap();
            apply_transition(&state, |_| TransitionPayload::Resize {
                target: added_id,
                start: ago(400),
                end: ago(330),
            })
            .unwrap();
            apply_transition(&state, |_| TransitionPayload::EditIdentity {
                target: added_id,
                name: "renamed".into(),
                project: Some("Acme".into()),
                client: None,
            })
            .unwrap();

            let inner = state.inner.lock().unwrap();
            let live = inner.stack.closed.iter().find(|b| b.id == added_id).unwrap();
            assert_eq!(live.name, "renamed");
            assert_eq!(live.start, ago(400));
            assert_eq!(live.end, Some(ago(330)));
            assert_eq!(live.capture_origin, crate::model::CaptureOrigin::ManualEntryAdjusted);
            (added_id, kept_id)
        };

        let (restarted, _) = AppState::init(&path, &snapshot_path).unwrap();
        let inner = restarted.inner.lock().unwrap();
        let replayed = inner.stack.closed.iter().find(|b| b.id == added_id).unwrap();

        assert_eq!(replayed.name, "renamed", "the identity edit replayed");
        assert_eq!(replayed.start, ago(400), "the move replayed");
        assert_eq!(replayed.end, Some(ago(330)), "the resize replayed");
        assert_eq!(
            replayed.capture_origin,
            crate::model::CaptureOrigin::ManualEntryAdjusted,
            "origin preserved, adjusted set — live and replayed agree"
        );
        assert_eq!(
            replayed.end_determination,
            Some(crate::model::EndDetermination::UserDetermined)
        );
        assert!(inner.stack.closed.iter().any(|b| b.id == kept_id), "the untouched neighbour survived");
    }

    /// The domain owns validation, not the UI. A command against a block that
    /// an unresolved interruption still refers to must surface the domain's own
    /// error — the frontend does not pre-empt it, so the two can never disagree.
    #[test]
    fn a_command_surfaces_the_domain_error_rather_than_the_ui_preventing_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, _) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();

        apply_transition(&state, |_| TransitionPayload::Start { name: "root".into(), project: None, client: None }).unwrap();
        let view = apply_transition(&state, |_| TransitionPayload::Interrupt {
            name: "phone".into(),
            project: None,
            client: None,
        })
        .unwrap();
        let paused = view.stack[0].paused_time_block_id;

        let before = log_lines(&path);
        let err = apply_transition(&state, |_| TransitionPayload::Delete { target: paused }).unwrap_err();
        assert!(err.contains("unresolved interruption"), "got: {err}");
        assert_eq!(log_lines(&path), before, "and nothing was written");

        // Editing that same block is permitted — identity only, per the tier rule.
        apply_transition(&state, |_| TransitionPayload::EditIdentity {
            target: paused,
            name: "renamed".into(),
            project: None,
            client: None,
        })
        .unwrap();
    }

    /// Delete's effect must reach the artifact that gets billed, and survive a
    /// restart — the two places a "removed from the timeline" claim can quietly
    /// fail to hold.
    #[test]
    fn a_deleted_block_leaves_the_history_view_the_export_and_the_replayed_state() {
        use crate::export::{blocks_in_range, group};
        use chrono::{Duration, Utc};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        let (kept, doomed) = {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
            apply_transition(&state, |_| TransitionPayload::Start { name: "kept".into(), project: None, client: None }).unwrap();
            apply_transition(&state, |_| TransitionPayload::Switch { name: "mistake".into(), project: None, client: None }).unwrap();
            let view = apply_transition(&state, |_| TransitionPayload::Complete).unwrap();
            let kept = view.closed.iter().find(|b| b.block.name == "kept").unwrap().block.id;
            let doomed = view.closed.iter().find(|b| b.block.name == "mistake").unwrap().block.id;

            let view = apply_transition(&state, |_| TransitionPayload::Delete { target: doomed }).unwrap();
            assert!(
                !view.closed.iter().any(|b| b.block.id == doomed),
                "gone from the view the History View renders"
            );

            let inner = state.inner.lock().unwrap();
            let rows = group(&blocks_in_range(
                &inner.stack.closed,
                inner.stack.active.as_ref(),
                Utc::now() - Duration::days(1),
                Utc::now() + Duration::days(1),
                Utc::now(),
            ));
            assert!(!rows.iter().any(|r| r.name == "mistake"), "gone from the export");
            assert!(rows.iter().any(|r| r.name == "kept"), "and the neighbour is untouched");
            (kept, doomed)
        };

        let (restarted, _) = AppState::init(&path, &snapshot_path).unwrap();
        let inner = restarted.inner.lock().unwrap();
        assert!(inner.stack.closed.iter().any(|b| b.id == kept), "the survivor keeps its identity");
        assert!(
            !inner.stack.closed.iter().any(|b| b.id == doomed),
            "replay reproduces the deletion — live and replayed state agree"
        );
    }

    /// `switch` used to branch internally and emit `Start` when nothing was
    /// active — one command, two transitions, which ADR 0005 rejected. These
    /// two tests pin the split: each command now has exactly one precondition,
    /// and choosing between them is the caller's job.
    #[test]
    fn switch_is_rejected_when_nothing_is_active() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, _) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();

        let err = apply_transition(&state, |stack| TransitionPayload::Switch {
            name: name_or_default(String::new(), stack),
            project: None,
            client: None,
        })
        .unwrap_err();
        assert!(err.contains("no active task"), "got: {err}");

        assert!(!path.exists() || std::fs::read_to_string(&path).unwrap().lines().count() == 0,
            "a rejected transition must never be written to the durable log");
    }

    #[test]
    fn start_is_rejected_when_something_is_already_active() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, _) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();

        apply_transition(&state, |stack| TransitionPayload::Start {
            name: name_or_default(String::new(), stack),
            project: None,
            client: None,
        })
        .unwrap();

        let err = apply_transition(&state, |stack| TransitionPayload::Start {
            name: name_or_default(String::new(), stack),
            project: None,
            client: None,
        })
        .unwrap_err();
        assert!(err.contains("already active"), "got: {err}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap().lines().count(),
            1,
            "only the accepted Start was written"
        );
    }

    /// `has_active` is what lets the capture hotkey and the dashboard pick the
    /// right command without either of them branching inside a transition.
    #[test]
    fn has_active_tracks_whether_a_task_is_being_tracked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, _) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
        assert!(!state.has_active(), "nothing active on a fresh state");

        apply_transition(&state, |stack| TransitionPayload::Start {
            name: name_or_default(String::new(), stack),
            project: None,
            client: None,
        })
        .unwrap();
        assert!(state.has_active());

        apply_transition(&state, |_| TransitionPayload::Complete).unwrap();
        assert!(!state.has_active(), "completing leaves nothing active — capture offers Start again");
    }

    #[test]
    fn rejects_return_previous_on_empty_stack_without_writing_to_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let (state, report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
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
            let (state, report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
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

        let (restarted, report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
        assert!(!report.torn_line_discarded);
        assert!(!report.startup_gap_recovered, "nothing was left active, so no gap should be detected");
        let post_restart_view = {
            let inner = restarted.inner.lock().unwrap();
            StackView::from(&inner.stack)
        };

        assert!(post_restart_view.active.is_none());
        assert_eq!(pre_drop_view.closed.len(), post_restart_view.closed.len());
        // **Identity survives the restart** (ADR 0006). This assertion used to
        // say the opposite — that ids were freshly random per `TimeBlock::new()`
        // and nothing relied on them being stable — which was true until
        // reconstruction needed to name a block written in an earlier session.
        // Ids now derive from the `seq` of the creating transition, so the same
        // log yields the same ids on every replay, forever.
        for (pre, post) in pre_drop_view.closed.iter().zip(post_restart_view.closed.iter()) {
            assert_eq!(pre.block.id, post.block.id, "a block keeps its identity across a restart");
            assert_eq!(pre.block.name, post.block.name);
            assert_eq!(pre.block.project, post.block.project);
            assert_eq!(pre.block.client, post.block.client);
            assert_eq!(pre.block.start, post.block.start);
            assert_eq!(pre.block.end, post.block.end);
            // All three metadata fields must survive replay identically — they
            // are derived by the state machine, never read back from the log,
            // so this is what proves the derivation is deterministic.
            assert_eq!(pre.block.end_determination, post.block.end_determination);
            assert_eq!(pre.block.capture_origin, post.block.capture_origin);
            assert_eq!(pre.block.interruption_outcome, post.block.interruption_outcome);
            assert_eq!(
                pre.derived_interruption_status, post.derived_interruption_status,
                "the canonical projection must agree across a restart too"
            );
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
            let (state, _report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
            apply_transition(&state, |_| TransitionPayload::Start {
                name: "A".into(),
                project: None,
                client: None,
            })
            .unwrap();
            // Dropped here with "A" still active — no Complete/Switch/Return.
        }

        let (restarted, report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
        assert!(report.startup_gap_recovered);
        let inner = restarted.inner.lock().unwrap();
        assert!(inner.stack.active.is_none());
        let a = inner.stack.closed.iter().find(|b| b.name == "A").unwrap();
        assert_eq!(a.end_determination, Some(crate::model::EndDetermination::SystemInferred));
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

        let (app_state, _report) = AppState::init(&log_path, dir.path().join("snapshot.json")).unwrap();
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
    fn export_writes_nothing_to_the_log_when_no_other_writer_is_active() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("log.jsonl");

        let (app_state, _report) = AppState::init(&log_path, dir.path().join("snapshot.json")).unwrap();
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
        // Byte equality is a legitimate assertion *here* and only here: nothing
        // is active, so no heartbeat is due, and this test is the quiescent
        // case. It is NOT the architectural invariant — see the companion test
        // below, which holds while a legitimate concurrent writer is running.
        assert_eq!(before, after, "with no other writer active, an export must leave the log untouched");
        assert!(xlsx_path.exists());
        assert!(json_path.exists());
    }

    /// The architectural invariant, stated in `docs/product/features/export.md`:
    /// **export itself performs no writes**. The test above proves only the
    /// quiescent case — it passes partly because nothing else was writing.
    ///
    /// This one runs a real concurrent writer on the heartbeat's exact append
    /// path (`apply_transition` with a `Heartbeat` payload, the same call
    /// `heartbeat::run` makes) throughout an export, and proves write
    /// *ownership*: the log may legitimately grow, but every added record must
    /// be attributable to the heartbeat and none to the export.
    ///
    /// **What this does not cover:** the heartbeat's 60-second timer. The real
    /// `heartbeat::run` needs a `tauri::AppHandle`, unavailable in a unit test,
    /// and waiting on a real interval would make this slow and timing-fragile.
    /// The timer decision is already covered by `heartbeat::tests`. What is
    /// exercised here is the part that matters for this invariant — a second
    /// thread appending to the same log, through the same lock, while export
    /// runs.
    ///
    /// **Determinism:** no sleeps and no timing assertions. The writer thread
    /// counts its own successful appends, and the assertion is an exact
    /// equality against that count — so it holds whether the thread manages one
    /// heartbeat or a thousand. A channel handshake guarantees at least one
    /// heartbeat has landed *before* the export starts, so the test cannot
    /// silently degenerate into the quiescent case.
    #[test]
    fn export_writes_nothing_to_the_log_while_the_heartbeat_is_appending() {
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::sync::{mpsc, Arc};

        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("log.jsonl");

        let (app_state, _report) = AppState::init(&log_path, dir.path().join("snapshot.json")).unwrap();
        let app_state = Arc::new(app_state);

        // Something must be active for a heartbeat to be due at all
        // (`heartbeat::should_beat`), so this mirrors the real precondition.
        apply_transition(&app_state, |_| TransitionPayload::Start {
            name: "A".into(),
            project: Some("Acme".into()),
            client: None,
        })
        .unwrap();

        let lines_before = std::fs::read_to_string(&log_path).unwrap().lines().count();

        let stop = Arc::new(AtomicBool::new(false));
        let beats = Arc::new(AtomicUsize::new(0));
        let (first_beat_tx, first_beat_rx) = mpsc::channel::<()>();

        let writer = {
            let state = Arc::clone(&app_state);
            let stop = Arc::clone(&stop);
            let beats = Arc::clone(&beats);
            std::thread::spawn(move || {
                // Land one heartbeat and announce it, so the export below is
                // guaranteed to run against a log a concurrent writer has
                // already touched — no sleep, no race.
                apply_transition(&state, |_| TransitionPayload::Heartbeat).unwrap();
                beats.fetch_add(1, Ordering::SeqCst);
                first_beat_tx.send(()).unwrap();

                // Keep appending for the duration of the export. However many
                // land is irrelevant to the assertions — they scale together.
                //
                // Capped, and yielding between appends, purely for runtime:
                // every append fsyncs, so an unbounded spin made this test
                // ~3s against a 0.2s suite and grew the log to no purpose. The
                // cap never weakens the assertion (which is an equality against
                // whatever count is reached) and the stop flag still ends the
                // loop first in the normal case.
                const MAX_BEATS: usize = 24;
                while !stop.load(Ordering::SeqCst) && beats.load(Ordering::SeqCst) < MAX_BEATS {
                    if apply_transition(&state, |_| TransitionPayload::Heartbeat).is_ok() {
                        beats.fetch_add(1, Ordering::SeqCst);
                    }
                    std::thread::yield_now();
                }
            })
        };

        first_beat_rx.recv().expect("writer thread must land its first heartbeat");

        // The export under test: the locked read, plus the formatting and file
        // writes that happen OUTSIDE the lock — which is precisely the window
        // where a concurrent heartbeat can interleave.
        let range_start = chrono::Utc::now() - chrono::Duration::days(1);
        let range_end = chrono::Utc::now() + chrono::Duration::days(1);
        let blocks = export_blocks_in_range(&app_state, range_start, range_end).unwrap();

        let xlsx_path = dir.path().join("out.xlsx");
        export::write_xlsx(&export::xlsx_rows(&blocks, Some(15)), &xlsx_path).unwrap();
        let json_path = dir.path().join("out.json");
        std::fs::write(&json_path, serde_json::to_string_pretty(&export::json_export(&blocks, None)).unwrap())
            .unwrap();

        stop.store(true, Ordering::SeqCst);
        writer.join().unwrap();

        let beats_written = beats.load(Ordering::SeqCst);
        let lines_after = std::fs::read_to_string(&log_path).unwrap().lines().count();

        // THE assertion. Every line added is one the heartbeat thread counted
        // for itself; anything else means export wrote — appended, rewrote,
        // checkpointed, whatever. Exact equality, but not timing-dependent:
        // both sides scale together with however many beats actually landed.
        assert_eq!(
            lines_after - lines_before,
            beats_written,
            "every record added during the export must be attributable to the heartbeat, none to the export"
        );
        assert!(beats_written >= 1, "the concurrent writer must actually have written, or this proves nothing");

        // Export produced its outputs and the data is right despite the
        // concurrent writes: one still-active block, "A".
        assert!(xlsx_path.exists());
        assert!(json_path.exists());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "A");
        // The exported copy carries a SYNTHETIC end (elapsed-so-far, computed
        // at the export moment) — documented behaviour in `export.md`, and the
        // sharpest illustration of what read-only means here: export may shape
        // its own output freely, so long as nothing flows back.
        assert!(blocks[0].end.is_some(), "an active block exports with its elapsed-so-far end");

        // …and the stored state proves nothing flowed back. Heartbeats have no
        // stack effect, so replaying must yield exactly the state we started
        // with: "A" still active, with NO end. A stray transition from export
        // would surface here even if it had somehow balanced the line count.
        let replayed = crate::log::reader::replay(&log_path, None, None).unwrap();
        let stored_active = replayed.stack.active.as_ref().expect("A must still be active in stored state");
        assert_eq!(stored_active.name, "A");
        assert!(stored_active.end.is_none(), "the synthetic export end must never be written back");
        assert!(replayed.stack.closed.is_empty(), "export must not have closed or created any block");
        assert_eq!(replayed.stack.stack_depth(), 0);
    }
}
