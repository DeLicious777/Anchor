//! Shared app state: the in-memory stack and the log writer, behind one Mutex so
//! they can never drift apart (a mutation to one always happens alongside the
//! other, under the same lock).

use crate::log::reader::replay;
use crate::log::writer::LogWriter;
use crate::model::TransitionPayload;
use crate::stack::InterruptionStack;
use chrono::{DateTime, Utc};
use std::path::Path;
use std::sync::Mutex;

pub struct AppState {
    pub inner: Mutex<Inner>,
}

pub struct Inner {
    pub stack: InterruptionStack,
    pub writer: LogWriter,
    /// Timestamp of the most recent durable write (any transition, including
    /// heartbeats) — read by the heartbeat and live sleep/hibernate-resume logic
    /// to decide how stale the currently-active entry's last known-alive point is.
    pub last_activity_at: DateTime<Utc>,
}

/// What happened during `AppState::init`, so the caller can surface it (this
/// slice has no dedicated UI for either — at minimum, don't drop them silently).
pub struct InitReport {
    pub torn_line_discarded: bool,
    /// True if replay left an active entry (the process stopped running — for
    /// any reason — while something was active) and it was closed as
    /// `recovered-gap`. Deliberately does NOT auto-resume: see `stack.rs`'s
    /// `RecoverGap` docs for why a restart and a live sleep/wake are handled
    /// differently.
    pub startup_gap_recovered: bool,
}

impl AppState {
    pub fn init(path: impl AsRef<Path>) -> Result<(Self, InitReport), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let result = replay(path, None, None)?;
        let mut writer = LogWriter::open(path, result.next_seq)?;
        let mut stack = result.stack;

        let mut startup_gap_recovered = false;
        let mut last_activity_at = result.last_timestamp.unwrap_or_else(Utc::now);

        if stack.active.is_some() {
            // Guaranteed Some: if replay left something active, at least the
            // line that started it was successfully parsed, so last_timestamp
            // must be Some too. An expect() here surfaces a real bug loudly
            // rather than silently guessing a fallback timestamp.
            let inferred_end = result
                .last_timestamp
                .expect("active entry survived replay but last_timestamp is None — replay invariant violated");

            let record = writer.append(TransitionPayload::RecoverGap { inferred_end })?;
            stack.apply(&record.payload, record.timestamp)?;
            last_activity_at = record.timestamp;
            startup_gap_recovered = true;
        }

        let state = AppState {
            inner: Mutex::new(Inner { stack, writer, last_activity_at }),
        };
        Ok((state, InitReport { torn_line_discarded: result.torn_line_discarded, startup_gap_recovered }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CompletionReason;

    #[test]
    fn leftover_active_entry_is_closed_as_recovered_gap_with_no_auto_resume() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer
                .append(TransitionPayload::Start { name: "A".into(), project: None, client: None })
                .unwrap();
            writer
                .append(TransitionPayload::Interrupt { name: "B".into(), project: None, client: None })
                .unwrap();
            // No Complete/Switch/Return — simulates a crash (or a graceful close)
            // while "B" was active. "A" is still pending on the stack.
        }

        let (state, report) = AppState::init(&path).unwrap();
        assert!(!report.torn_line_discarded);
        assert!(report.startup_gap_recovered);

        let inner = state.inner.lock().unwrap();
        assert!(inner.stack.active.is_none(), "startup recovery must not auto-resume");
        let b = inner.stack.closed.iter().find(|b| b.name == "B").unwrap();
        assert_eq!(b.completion_reason, Some(CompletionReason::RecoveredGap));
        assert!(b.end.is_some());
        // "A" (paused on the stack) is untouched by startup recovery — it's not
        // the active entry, so it stays pending until explicitly resumed.
        let a = inner.stack.closed.iter().find(|b| b.name == "A").unwrap();
        assert_eq!(a.completion_reason, None);
        assert_eq!(inner.stack.stack_depth(), 1);
    }

    #[test]
    fn clean_state_with_nothing_active_reports_no_gap_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer
                .append(TransitionPayload::Start { name: "A".into(), project: None, client: None })
                .unwrap();
            writer.append(TransitionPayload::Complete).unwrap();
        }

        let (state, report) = AppState::init(&path).unwrap();
        assert!(!report.startup_gap_recovered);
        let inner = state.inner.lock().unwrap();
        assert!(inner.stack.active.is_none());
    }
}
