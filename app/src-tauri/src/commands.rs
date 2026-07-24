//! Tauri commands exposing the interruption-stack operations to the frontend.
//!
//! Every mutating command follows the same pattern (`apply_transition`): dry-run
//! the proposed transition against a clone of the in-memory stack FIRST — a
//! transition that would fail its precondition (e.g. Return with an empty
//! stack) is rejected before anything is written to the durable log. Only once
//! the dry-run succeeds does the real transition get appended (fsync'd) and
//! then applied to the real in-memory stack. This is the concrete mechanism
//! behind "never durably log a transition that couldn't actually happen."

use crate::model::{StackFrame, TimeBlock, TransitionPayload};
use crate::stack::InterruptionStack;
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

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

#[tauri::command]
pub fn switch(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    let view = apply_transition(&state, |stack| {
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
    let view = apply_transition(&state, |_| TransitionPayload::Interrupt { name, project, client })?;
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
}
