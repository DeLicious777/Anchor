//! 60-second heartbeat: while a task is active, append a `Heartbeat` transition
//! so `recovered-gap` inference (crash or live sleep/wake) never goes stale by
//! more than the heartbeat interval, per ADR 0004 / the feature doc.

use crate::commands::{apply_transition, emit_state_changed};
use crate::model::TransitionPayload;
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub const DEFAULT_INTERVAL_SECS: u64 = 60;

/// Overridable via `ANCHOR_HEARTBEAT_SECS` for manual testing — production
/// default stays 60.
pub fn interval() -> Duration {
    let secs = std::env::var("ANCHOR_HEARTBEAT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_INTERVAL_SECS);
    Duration::from_secs(secs)
}

/// Pure logic — should a heartbeat be appended given whether a task is
/// currently active? Separated from the thread/sleep loop so it's testable
/// without waiting on a real timer. Trivial today, but keeps the "when do we
/// beat" decision in one place if the condition ever grows.
pub fn should_beat(active: bool) -> bool {
    active
}

/// Runs forever on its own thread (spawned from `.setup()`), waking every
/// `interval()` to check whether anything is active and, if so, append a
/// heartbeat. Never runs while idle — an idle app writes nothing.
pub fn run(app: AppHandle) {
    let period = interval();
    loop {
        std::thread::sleep(period);

        let Some(state) = app.try_state::<AppState>() else {
            continue;
        };
        let is_active = {
            let inner = state.inner.lock().unwrap();
            inner.stack.active.is_some()
        };
        if !should_beat(is_active) {
            continue;
        }

        match apply_transition(&state, |_| TransitionPayload::Heartbeat) {
            Ok(view) => emit_state_changed(&app, &view),
            Err(e) => eprintln!("heartbeat append failed: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_beats_while_something_is_active() {
        assert!(should_beat(true));
        assert!(!should_beat(false));
    }

    #[test]
    fn interval_defaults_to_60_seconds() {
        // Only meaningful if the env var isn't set in this test process;
        // ANCHOR_HEARTBEAT_SECS is not set by the test harness by default.
        if std::env::var("ANCHOR_HEARTBEAT_SECS").is_err() {
            assert_eq!(interval(), Duration::from_secs(DEFAULT_INTERVAL_SECS));
        }
    }
}
