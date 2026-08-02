//! Shared app state: the in-memory stack and the log writer, behind one Mutex so
//! they can never drift apart (a mutation to one always happens alongside the
//! other, under the same lock).

use crate::log::reader::replay;
use crate::log::snapshot::{compact, CompactionTrigger, Snapshot};
use crate::log::writer::LogWriter;
use crate::model::TransitionPayload;
use crate::stack::InterruptionStack;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
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
    /// Progress toward ADR 0004's compaction trigger. Deliberately **not**
    /// persisted: no accepted invariant requires compaction to fire at exactly
    /// N since the last one, so a restart resetting it delays compaction and
    /// violates nothing. See `log::snapshot`.
    pub compaction: CompactionTrigger,
    /// Where `compact` writes. Held here so the trigger can fire from wherever
    /// a transition is applied, without threading a path through every caller.
    pub snapshot_path: PathBuf,
}

/// Snapshot the current projection and truncate the log, if the trigger says so.
///
/// Failure is deliberately **not** propagated to the caller. Compaction is an
/// optimisation of startup replay, never a source of truth — a failed
/// compaction leaves the log intact and fully replayable, so turning it into a
/// user-visible error for a transition that already succeeded durably would
/// report a problem the user has not got. It is reported and the trigger is
/// left unreset, so the next transition simply tries again.
pub fn compact_if_due(inner: &mut Inner) {
    let next_seq = inner.writer.next_seq();
    if !inner.compaction.should_compact(next_seq) {
        return;
    }
    let watermark = next_seq - 1;
    let snapshot = Snapshot::new(watermark, inner.stack.clone(), inner.last_activity_at);
    match compact(&inner.snapshot_path, inner.writer.path(), &snapshot) {
        Ok(()) => inner.compaction.reset(watermark),
        Err(e) => eprintln!("compaction failed, log left intact and will be retried: {e}"),
    }
}

/// The clean-shutdown arm of ADR 0004's trigger. No threshold — on the way out,
/// any uncompacted work is worth snapshotting, because it makes the next
/// startup cheaper and there is no next transition to wait for.
pub fn compact_on_shutdown(state: &AppState) {
    let Ok(mut inner) = state.inner.lock() else { return };
    let next_seq = inner.writer.next_seq();
    if !inner.compaction.has_uncompacted_work(next_seq) {
        return;
    }
    let watermark = next_seq - 1;
    let snapshot = Snapshot::new(watermark, inner.stack.clone(), inner.last_activity_at);
    match compact(&inner.snapshot_path, inner.writer.path(), &snapshot) {
        Ok(()) => inner.compaction.reset(watermark),
        Err(e) => eprintln!("shutdown compaction failed, log left intact: {e}"),
    }
}

/// What happened during `AppState::init`, so the caller can surface it (this
/// slice has no dedicated UI for either — at minimum, don't drop them silently).
pub struct InitReport {
    pub torn_line_discarded: bool,
    /// True if replay left an active entry (the process stopped running — for
    /// any reason — while something was active) and it was closed with an
    /// inferred end (`EndDetermination::SystemInferred`). Deliberately does NOT
    /// auto-resume — and since 2026-07-29 the live sleep/wake path does not
    /// either (ADR 0005 open item 9): wake and crash are the same class of
    /// event, so they resolve identically.
    pub startup_gap_recovered: bool,
}

impl AppState {
    /// `snapshot_path` may not exist — a missing, corrupt or wrong-version
    /// snapshot simply means a full replay of the log, which is always correct
    /// (`docs/architecture/constraints.md`: the log is the source of truth and
    /// the snapshot is only ever an optimisation of replaying it).
    pub fn init(
        path: impl AsRef<Path>,
        snapshot_path: impl AsRef<Path>,
    ) -> Result<(Self, InitReport), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let snapshot_path = snapshot_path.as_ref().to_path_buf();

        let snapshot = Snapshot::load(&snapshot_path);
        let (watermark, starting_stack, snapshot_activity) = match snapshot {
            Some(s) => (Some(s.watermark), Some(s.stack), Some(s.last_activity_at)),
            None => (None, None, None),
        };

        let result = replay(path, watermark, starting_stack)?;
        let mut writer = LogWriter::open(path, result.next_seq)?;
        let mut stack = result.stack;

        let mut startup_gap_recovered = false;

        // **Assumption A15.** The last durable write is a property of the log
        // *lines*, and compaction truncates them — so after a compaction the
        // only remaining copy is the one the snapshot carried. Falling back to
        // it, rather than to `Utc::now()`, is what keeps the inferred end
        // bounded to roughly the heartbeat interval (risk R4) instead of
        // silently billing every hour the process was not running.
        let last_activity_on_disk = result.last_timestamp.or(snapshot_activity);
        let mut last_activity_at = last_activity_on_disk.unwrap_or_else(Utc::now);

        if stack.active.is_some() {
            // Guaranteed Some: an active entry reached this point either from a
            // parsed line (which sets `last_timestamp`) or from a snapshot
            // (which carries `last_activity_at`). An expect() here surfaces a
            // real bug loudly rather than silently guessing a fallback — and
            // guessing would be the dangerous option, since `Utc::now()` would
            // infer an end covering the entire time the app was closed.
            let inferred_end = last_activity_on_disk
                .expect("active entry survived replay but neither the log nor the snapshot supplied a last-activity timestamp");

            let record = writer.append(TransitionPayload::RecoverGap { inferred_end })?;
            stack.apply(&record.payload, record.timestamp, record.seq)?;
            last_activity_at = record.timestamp;
            startup_gap_recovered = true;
        }

        let state = AppState {
            inner: Mutex::new(Inner {
                stack,
                writer,
                last_activity_at,
                compaction: CompactionTrigger::default(),
                snapshot_path,
            }),
        };
        Ok((state, InitReport { torn_line_discarded: result.torn_line_discarded, startup_gap_recovered }))
    }

    /// Read-only peek, for presentation layers deciding which capture action to
    /// offer — `Start` when nothing is active, `Switch` when something is (see
    /// `commands::start`). Deliberately NOT a transition, and deliberately not
    /// used to make that choice inside a command: a command that branches on
    /// state is the overload ADR 0005 rejected.
    ///
    /// The peek-then-act this enables is a two-step, so state can in principle
    /// change in between (the sleep/wake path can close the active entry from
    /// another thread). That race is safe by construction: the follow-up
    /// transition simply fails its precondition and reports it, rather than
    /// silently applying the wrong one.
    pub fn has_active(&self) -> bool {
        self.inner.lock().map(|inner| inner.stack.active.is_some()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::snapshot::{compact, Snapshot};
    use crate::model::EndDetermination;
    use chrono::Duration;

    /// **A15, as behaviour.** Compaction runs while a block is active, so the
    /// snapshot holds "something is active" and the truncated log holds nothing
    /// at all. Gap recovery must still bound the inferred end to the last
    /// durable write — which now exists only in the snapshot.
    ///
    /// Without `Snapshot.last_activity_at` this test does not merely fail, it
    /// **panics on `init`'s own expect**, because `last_timestamp` is `None`
    /// while `stack.active` is `Some`. And the panic is the *good* outcome: the
    /// tempting fallback, `Utc::now()`, would infer an end covering every hour
    /// the process was not running and bill it silently — risk **R4** at
    /// maximum severity, in the artifact that has to be trustworthy for billing.
    #[test]
    fn gap_recovery_after_compaction_uses_the_snapshot_timestamp_not_now() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        let last_write = {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            let record = writer
                .append(TransitionPayload::Start { name: "long task".into(), project: None, client: None })
                .unwrap();
            record.timestamp
        };

        // Compaction, mid-session, with the block still running.
        let replayed = replay(&path, None, None).unwrap();
        assert!(replayed.stack.active.is_some());
        let snapshot = Snapshot::new(replayed.next_seq - 1, replayed.stack, last_write);
        compact(&snapshot_path, &path, &snapshot).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "", "the log is truncated");

        // The machine is off for a long time, then the app starts again.
        let (state, report) = AppState::init(&path, &snapshot_path).unwrap();
        assert!(report.startup_gap_recovered);

        let inner = state.inner.lock().unwrap();
        let block = inner.stack.closed.iter().find(|b| b.name == "long task").unwrap();
        assert_eq!(block.end_determination, Some(EndDetermination::SystemInferred));
        assert_eq!(
            block.end,
            Some(last_write),
            "the end is the last durable write carried by the snapshot, not the moment of restart"
        );
        assert!(
            Utc::now() - block.end.unwrap() >= Duration::zero(),
            "sanity: the inferred end is in the past"
        );
    }

    /// **The durability ordering.** `compact` makes the snapshot durable before
    /// truncating, so a crash between the two steps is survivable: the snapshot
    /// and the still-intact log describe overlapping history, and replay folds
    /// the post-watermark lines onto the restored projection to reach exactly
    /// the same place.
    ///
    /// Compaction may duplicate work. It must never destroy recoverable history.
    #[test]
    fn a_crash_between_writing_the_snapshot_and_truncating_the_log_loses_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(TransitionPayload::Start { name: "first".into(), project: None, client: None }).unwrap();
        writer.append(TransitionPayload::Complete).unwrap();

        // Snapshot written — and then the process dies, so the log is NOT truncated.
        let replayed = replay(&path, None, None).unwrap();
        let watermark = replayed.next_seq - 1;
        Snapshot::new(watermark, replayed.stack, Utc::now()).write(&snapshot_path).unwrap();

        // More work happens in a later session, above the watermark.
        let mut writer = LogWriter::open(&path, watermark + 1).unwrap();
        writer.append(TransitionPayload::Start { name: "second".into(), project: None, client: None }).unwrap();
        writer.append(TransitionPayload::Complete).unwrap();
        drop(writer);

        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
        let inner = state.inner.lock().unwrap();

        let names: Vec<_> = inner.stack.closed.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["first", "second"], "both sessions survive, neither is duplicated");
    }

    /// A snapshot that cannot be trusted must not block startup: the log is the
    /// source of truth, and the snapshot is only ever an optimisation of
    /// replaying it (`docs/architecture/constraints.md`).
    #[test]
    fn a_corrupt_snapshot_falls_back_to_replaying_the_intact_log() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(TransitionPayload::Start { name: "work".into(), project: None, client: None }).unwrap();
        writer.append(TransitionPayload::Complete).unwrap();
        drop(writer);

        std::fs::write(&snapshot_path, b"{\"version\": 1, \"watermark\":").unwrap();

        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
        let inner = state.inner.lock().unwrap();
        assert_eq!(inner.stack.closed.len(), 1);
        assert_eq!(inner.stack.closed[0].name, "work");
    }

    /// The threshold arm, end to end: transitions accumulate through the real
    /// command path, compaction fires on its own, and the projection survives a
    /// restart from a snapshot plus a truncated log.
    #[test]
    fn the_threshold_arm_fires_and_the_projection_survives_the_restart() {
        use crate::commands::apply_transition;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");
        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();

        // One Start, then Switch until the threshold is crossed.
        apply_transition(&state, |_| TransitionPayload::Start {
            name: "t0".into(),
            project: None,
            client: None,
        })
        .unwrap();
        for i in 1..CompactionTrigger::THRESHOLD {
            apply_transition(&state, move |_| TransitionPayload::Switch {
                name: format!("t{i}"),
                project: None,
                client: None,
            })
            .unwrap();
        }

        {
            let inner = state.inner.lock().unwrap();
            assert_eq!(inner.compaction.count(), 0, "the trigger reset, so compaction ran");
        }
        assert!(snapshot_path.exists(), "a snapshot was written");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "", "and the log was truncated");
        drop(state);

        let (restarted, report) = AppState::init(&path, &snapshot_path).unwrap();
        assert!(report.startup_gap_recovered, "t499 was still active when compaction ran");
        let inner = restarted.inner.lock().unwrap();
        assert_eq!(
            inner.stack.closed.len() as u32,
            CompactionTrigger::THRESHOLD,
            "every block survives the snapshot round-trip"
        );
    }

    /// Heartbeats must never drive compaction. ADR 0004 is explicit: a
    /// heads-down day with no manual transitions still writes ~480 heartbeat
    /// lines, and compacting on that volume is not what the trigger is for.
    #[test]
    fn heartbeats_alone_never_trigger_compaction() {
        use crate::commands::apply_transition;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");
        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();

        apply_transition(&state, |_| TransitionPayload::Start {
            name: "heads down".into(),
            project: None,
            client: None,
        })
        .unwrap();
        for _ in 0..CompactionTrigger::THRESHOLD + 100 {
            apply_transition(&state, |_| TransitionPayload::Heartbeat).unwrap();
        }

        let inner = state.inner.lock().unwrap();
        assert_eq!(inner.compaction.count(), 1, "only the Start counted");
        assert!(!snapshot_path.exists(), "no compaction from heartbeat volume alone");
    }

    /// The shutdown arm has no threshold, but it must still not redo work.
    #[test]
    fn shutdown_compacts_pending_work_once_and_not_again() {
        use crate::commands::apply_transition;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");
        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();

        apply_transition(&state, |_| TransitionPayload::Start {
            name: "brief".into(),
            project: None,
            client: None,
        })
        .unwrap();
        apply_transition(&state, |_| TransitionPayload::Complete).unwrap();

        compact_on_shutdown(&state);
        assert!(snapshot_path.exists());
        let after_first = std::fs::metadata(&snapshot_path).unwrap().modified().unwrap();

        // A second call with nothing new must be a no-op, not an identical rewrite.
        compact_on_shutdown(&state);
        assert_eq!(
            std::fs::metadata(&snapshot_path).unwrap().modified().unwrap(),
            after_first,
            "the same state must never be compacted twice"
        );
    }

    /// The quiet case: compaction with nothing running. No gap to recover, and
    /// `next_seq` must still continue from the watermark rather than restarting.
    #[test]
    fn compaction_while_idle_recovers_no_gap_and_continues_the_sequence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(TransitionPayload::Start { name: "done".into(), project: None, client: None }).unwrap();
        writer.append(TransitionPayload::Complete).unwrap();
        drop(writer);

        let replayed = replay(&path, None, None).unwrap();
        let watermark = replayed.next_seq - 1;
        let snapshot = Snapshot::new(watermark, replayed.stack, Utc::now());
        compact(&snapshot_path, &path, &snapshot).unwrap();

        let (state, report) = AppState::init(&path, &snapshot_path).unwrap();
        assert!(!report.startup_gap_recovered, "nothing was active, so there is no gap");

        let inner = state.inner.lock().unwrap();
        assert_eq!(inner.stack.closed.len(), 1, "history below the watermark comes from the snapshot");
        assert_eq!(
            inner.writer.next_seq(),
            watermark + 1,
            "seq continues past the watermark — a truncated log must not restart it at 0"
        );
    }

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

        let (state, report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
        assert!(!report.torn_line_discarded);
        assert!(report.startup_gap_recovered);

        let inner = state.inner.lock().unwrap();
        assert!(inner.stack.active.is_none(), "startup recovery must not auto-resume");
        let b = inner.stack.closed.iter().find(|b| b.name == "B").unwrap();
        assert_eq!(b.end_determination, Some(EndDetermination::SystemInferred));
        assert!(b.end.is_some());
        // "A" (paused on the stack) is untouched by startup recovery — it's not
        // the active entry, so it stays pending until explicitly resumed.
        let a = inner.stack.closed.iter().find(|b| b.name == "A").unwrap();
        assert_eq!(a.end_determination, Some(EndDetermination::UserDetermined), "A was closed by the Interrupt, not by the gap");
        assert_eq!(a.interruption_outcome, None, "never resolved — its frame was still open at the crash");
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

        let (state, report) = AppState::init(&path, dir.path().join("snapshot.json")).unwrap();
        assert!(!report.startup_gap_recovered);
        let inner = state.inner.lock().unwrap();
        assert!(inner.stack.active.is_none());
    }
}
