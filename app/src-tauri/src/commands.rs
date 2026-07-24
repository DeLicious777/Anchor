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
use tauri::State;

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

/// The single entry point every mutating command goes through. `build_payload`
/// receives a read-only view of the current stack so callers like `switch` can
/// decide their exact transition type (Start vs. Switch) under the same lock
/// that will perform the write — no separate peek-then-act race.
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

    Ok(StackView::from(&inner.stack))
}

#[tauri::command]
pub fn switch(
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    apply_transition(&state, |stack| {
        if stack.active.is_none() {
            TransitionPayload::Start { name, project, client }
        } else {
            TransitionPayload::Switch { name, project, client }
        }
    })
}

#[tauri::command]
pub fn interrupt(
    state: State<AppState>,
    name: String,
    project: Option<String>,
    client: Option<String>,
) -> Result<StackView, String> {
    apply_transition(&state, |_| TransitionPayload::Interrupt { name, project, client })
}

#[tauri::command]
pub fn return_previous(state: State<AppState>) -> Result<StackView, String> {
    apply_transition(&state, |_| TransitionPayload::ReturnPrevious)
}

#[tauri::command]
pub fn return_original(state: State<AppState>) -> Result<StackView, String> {
    apply_transition(&state, |_| TransitionPayload::ReturnOriginal)
}

#[tauri::command]
pub fn complete(state: State<AppState>) -> Result<StackView, String> {
    apply_transition(&state, |_| TransitionPayload::Complete)
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
        let (state, torn) = AppState::init(&path).unwrap();
        assert!(!torn);

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
    /// file-backed AppState through a sequence, drop it, replay from the same
    /// file, and assert the reconstructed stack matches the pre-drop state.
    #[test]
    fn restart_restores_full_stack_from_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");

        let pre_drop_view = {
            let (state, torn) = AppState::init(&path).unwrap();
            assert!(!torn);

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
            apply_transition(&state, |_| TransitionPayload::ReturnPrevious).unwrap()
            // `state` (and its open file handle) is dropped at the end of this block.
        };

        // Fresh AppState from the same file — simulates an app restart.
        let (restarted, torn) = AppState::init(&path).unwrap();
        assert!(!torn);
        let post_restart_view = {
            let inner = restarted.inner.lock().unwrap();
            StackView::from(&inner.stack)
        };

        assert_eq!(pre_drop_view.stack.len(), post_restart_view.stack.len());
        assert_eq!(
            pre_drop_view.active.as_ref().map(|b| &b.name),
            post_restart_view.active.as_ref().map(|b| &b.name)
        );
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
}
