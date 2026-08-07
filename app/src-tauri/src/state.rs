//! Shared app state: the in-memory stack and the log writer, behind one Mutex so
//! they can never drift apart (a mutation to one always happens alongside the
//! other, under the same lock).

use crate::log::reader::replay;
use crate::log::snapshot::{compact, CompactionTrigger, Snapshot};
use crate::log::writer::LogWriter;
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
    /// any reason — while something was active) and the gap was long enough to
    /// close it with an inferred end (`EndDetermination::SystemInferred`).
    ///
    /// False when the outage was under the continuity threshold and the block
    /// simply carried on. Whether the task was then auto-resumed is decided by
    /// `crate::gap` per ADR 0007 — startup and sleep/wake share one rule, so
    /// they resolve identically.
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

        // **The shared gap rule** (ADR 0007). Startup used to close any leftover
        // active block unconditionally while the wake path ignored gaps under 90
        // seconds — so a brief crash-relaunch produced a zero-duration block
        // while a brief sleep-wake produced nothing. Both paths now ask
        // `crate::gap`, so they cannot disagree again.
        if stack.active.is_some() {
            // Guaranteed Some: an active entry reached this point either from a
            // parsed line (which sets `last_timestamp`) or from a snapshot
            // (which carries `last_activity_at`). An expect() here surfaces a
            // real bug loudly rather than silently guessing a fallback — and
            // guessing would be the dangerous option, since `Utc::now()` would
            // infer an end covering the entire time the app was closed.
            let last_alive = last_activity_on_disk
                .expect("active entry survived replay but neither the log nor the snapshot supplied a last-activity timestamp");

            let resolution = crate::gap::resolve(stack.active.as_ref(), last_alive, Utc::now());
            for payload in resolution.transitions() {
                let record = writer.append(payload)?;
                stack.apply(&record.payload, record.timestamp, record.seq)?;
                last_activity_at = record.timestamp;
            }
            // Reported only when a block was actually closed, not when the gap
            // was short enough to ignore.
            startup_gap_recovered = !matches!(resolution, crate::gap::GapResolution::Continue);
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
    use crate::model::TransitionPayload;
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

        // Backdated well past the continuity threshold so this is a real gap
        // (ADR 0007). The subject of this test is unchanged — *where* the
        // inferred end comes from after the log is truncated — and a gap short
        // enough to ignore would simply not exercise it.
        let last_write = {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer
                .append(TransitionPayload::Start { name: "long task".into(), project: None, client: None })
                .unwrap();
            Utc::now() - Duration::hours(3)
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

    /// **The reason the whole identity architecture exists, end to end.**
    ///
    /// Live session -> log -> restart -> replay -> `EditIdentity` naming a block
    /// from the *previous* process -> restart again -> the reference still
    /// points at the same block, and the edit survived.
    ///
    /// Every other identity test checks one link: derivation is deterministic,
    /// or replay is stable. This one checks that a reference **persisted by one
    /// process still resolves after later replays**, which is the actual
    /// requirement ADR 0006 was written for. It is also the regression that
    /// would catch an accidental return to `Uuid::new_v4()` — nothing else in
    /// the suite fails loudly on that, because a fresh random id is internally
    /// consistent within any single run.
    #[test]
    fn a_reference_written_in_one_session_still_resolves_after_later_replays() {
        use crate::commands::{apply_transition, StackView};
        use crate::model::CaptureOrigin;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        // Session 1: do some work.
        let target_id = {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
            apply_transition(&state, |_| TransitionPayload::Start {
                name: "Anchor 1".into(),
                project: None,
                client: None,
            })
            .unwrap();
            let view = apply_transition(&state, |_| TransitionPayload::Complete).unwrap();
            view.closed[0].block.id
        };

        // Session 2: a different process replays the log and edits a block it
        // never created, naming it by an id read from the earlier run.
        {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
            let replayed_id = {
                let inner = state.inner.lock().unwrap();
                StackView::from(&inner.stack).closed[0].block.id
            };
            assert_eq!(replayed_id, target_id, "replay reproduced the same id");

            apply_transition(&state, |_| TransitionPayload::EditIdentity {
                target: target_id,
                name: "quarterly report".into(),
                project: Some("Acme".into()),
                client: None,
            })
            .unwrap();
        }

        // Session 3: the durable reference is replayed from the log itself.
        {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
            let inner = state.inner.lock().unwrap();
            let block = &inner.stack.closed[0];
            assert_eq!(block.id, target_id, "identity survived two further replays");
            assert_eq!(block.name, "quarterly report", "and the edit replayed onto the right block");
            assert_eq!(block.project.as_deref(), Some("Acme"));
            assert_eq!(block.capture_origin, CaptureOrigin::LiveCaptureAdjusted);
            assert_eq!(
                inner.stack.next_default_name(block.start),
                "Anchor 2",
                "the renamed block's auto-name is still spent"
            );
        }
    }

    /// The same loop with a compaction in between, so the reference resolves
    /// against a block that came from the **snapshot** rather than from any
    /// surviving log line. A14 as behaviour.
    ///
    /// **It does not guard the derivation, and that is not an oversight.**
    /// Verified by simulating a regression to `Uuid::new_v4()`: this test still
    /// passes, because a snapshotted block's id is *deserialised*, never
    /// re-derived. The derivation is guarded by
    /// `a_reference_written_in_one_session_still_resolves_after_later_replays`
    /// and by `log::reader`'s replay-stability test, both of which fail loudly.
    /// Noted so nobody reads this one as covering more than it does.
    #[test]
    fn a_reference_still_resolves_against_a_block_restored_from_a_snapshot() {
        use crate::commands::apply_transition;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        let target_id = {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
            apply_transition(&state, |_| TransitionPayload::Start {
                name: "old work".into(),
                project: None,
                client: None,
            })
            .unwrap();
            let view = apply_transition(&state, |_| TransitionPayload::Complete).unwrap();
            let id = view.closed[0].block.id;
            compact_on_shutdown(&state);
            id
        };
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "", "only the snapshot holds that block now");

        {
            let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
            apply_transition(&state, |_| TransitionPayload::EditIdentity {
                target: target_id,
                name: "corrected".into(),
                project: None,
                client: None,
            })
            .unwrap();
        }

        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
        let inner = state.inner.lock().unwrap();
        assert_eq!(inner.stack.closed[0].id, target_id);
        assert_eq!(
            inner.stack.closed[0].name, "corrected",
            "a block below the watermark is still a valid reconstruction target"
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
        // The 500 transitions are written back to back, so the restart lands
        // inside the continuity threshold and t499 simply keeps running
        // (ADR 0007). This test's subject is compaction, not gap recovery.
        assert!(!report.startup_gap_recovered, "a sub-threshold restart is not a gap");
        let inner = restarted.inner.lock().unwrap();
        assert_eq!(
            inner.stack.closed.len() as u32,
            CompactionTrigger::THRESHOLD - 1,
            "every closed block survives the snapshot round-trip; t499 is still active"
        );
        assert!(inner.stack.active.is_some(), "and it carried on across the restart");
    }

    /// A dismissal must survive both durability paths identically: replayed
    /// from the log, and restored from a snapshot after the log is truncated.
    ///
    /// Worth asserting end to end rather than trusting the arm. `log::reader`
    /// calls `apply` directly with no dry-run guard, and compaction persists
    /// `InterruptionStack` whole — so a frame removed live but not on replay,
    /// or an outcome that survives replay but not the snapshot, would be a
    /// silent divergence between what the user saw and what the app reopens to.
    #[test]
    fn a_dismissal_survives_replay_and_a_snapshot_round_trip() {
        use crate::commands::apply_transition;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");

        // The whole projection, serialised — the same thing the snapshot
        // persists, so comparing the serialised form proves complete projection
        // equality across all three paths rather than for selected fields.
        let serialised = |s: &AppState| {
            let inner = s.inner.lock().unwrap();
            serde_json::to_string(&inner.stack).unwrap()
        };

        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
        let named = |n: &str| TransitionPayload::Interrupt { name: n.into(), project: None, client: None };
        apply_transition(&state, |_| TransitionPayload::Start { name: "root".into(), project: None, client: None }).unwrap();
        apply_transition(&state, |_| named("phone")).unwrap();
        apply_transition(&state, |_| named("colleague")).unwrap();
        apply_transition(&state, |_| named("walk-up")).unwrap();

        // **Three frames, dismissing index 1** — frames survive on both sides.
        // Three is the minimum that proves non-top removal: at depth 2 the
        // "middle" is also the top, and a pop satisfies the assertion.
        let (bottom, middle, top) = {
            let inner = state.inner.lock().unwrap();
            assert_eq!(inner.stack.stack.len(), 3);
            (
                inner.stack.stack[0].paused_time_block_id,
                inner.stack.stack[1].paused_time_block_id,
                inner.stack.stack[2].paused_time_block_id,
            )
        };
        apply_transition(&state, move |_| TransitionPayload::DismissFrame { target: middle }).unwrap();

        {
            let inner = state.inner.lock().unwrap();
            assert_eq!(
                inner.stack.stack.iter().map(|f| f.paused_time_block_id).collect::<Vec<_>>(),
                vec![bottom, top],
                "live: the middle frame went and the survivors kept their order"
            );
            let dismissed = inner.stack.closed.iter().find(|b| b.id == middle).unwrap();
            assert_eq!(dismissed.interruption_outcome, Some(crate::model::InterruptionOutcome::Skipped));
        }
        let live = serialised(&state);
        drop(state);

        // Path 1: replayed from the log, with no snapshot present.
        assert!(!snapshot_path.exists(), "nothing has compacted yet");
        let (replayed, _) = AppState::init(&path, &snapshot_path).unwrap();
        assert_eq!(serialised(&replayed), live, "replay must reproduce what the live run produced");

        // Path 2: snapshot written and the log truncated, then reopened.
        compact_on_shutdown(&replayed);
        drop(replayed);
        assert!(snapshot_path.exists(), "shutdown compaction wrote a snapshot");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "", "and truncated the log");

        let (restored, _) = AppState::init(&path, &snapshot_path).unwrap();
        assert_eq!(
            serialised(&restored),
            live,
            "the snapshot must restore the dismissal identically, with the log gone"
        );
    }

    #[test]
    fn a_paused_state_survives_replay_and_a_snapshot_round_trip() {
        use crate::commands::apply_transition;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");
        let serialised = |state: &AppState| {
            let inner = state.inner.lock().unwrap();
            serde_json::to_string(&inner.stack).unwrap()
        };

        let (state, _) = AppState::init(&path, &snapshot_path).unwrap();
        apply_transition(&state, |_| TransitionPayload::Start {
            name: "root".into(),
            project: None,
            client: None,
        })
        .unwrap();
        apply_transition(&state, |_| TransitionPayload::Interrupt {
            name: "phone".into(),
            project: None,
            client: None,
        })
        .unwrap();
        apply_transition(&state, |_| TransitionPayload::Pause).unwrap();

        {
            let inner = state.inner.lock().unwrap();
            assert!(inner.stack.active.is_none());
            assert_eq!(inner.stack.stack.len(), 2, "root plus the paused phone task");
        }
        let live = serialised(&state);
        let lines_before_restart = std::fs::read_to_string(&path).unwrap().lines().count();
        drop(state);

        let (replayed, report) = AppState::init(&path, &snapshot_path).unwrap();
        assert!(
            !report.startup_gap_recovered,
            "paused state has no active block to recover"
        );
        assert_eq!(serialised(&replayed), live);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap().lines().count(),
            lines_before_restart,
            "restart while paused appends no gap transition"
        );

        compact_on_shutdown(&replayed);
        drop(replayed);
        assert!(snapshot_path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "");

        let (restored, report) = AppState::init(&path, &snapshot_path).unwrap();
        assert!(!report.startup_gap_recovered);
        assert_eq!(
            serialised(&restored),
            live,
            "snapshot restore reproduces the paused projection exactly"
        );
    }

    #[test]
    fn an_old_paused_snapshot_stays_silent_at_any_elapsed_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let snapshot_path = dir.path().join("snapshot.json");
        let paused_at = Utc::now() - Duration::hours(12);

        let mut stack = InterruptionStack::new();
        stack
            .apply(
                &TransitionPayload::Start { name: "waiting".into(), project: None, client: None },
                paused_at - Duration::hours(1),
                0,
            )
            .unwrap();
        stack.apply(&TransitionPayload::Pause, paused_at, 1).unwrap();
        let expected = serde_json::to_string(&stack).unwrap();
        Snapshot::new(1, stack, paused_at)
            .write(&snapshot_path)
            .unwrap();

        let (state, report) = AppState::init(&path, &snapshot_path).unwrap();
        assert!(
            !report.startup_gap_recovered,
            "elapsed time cannot create a gap when nothing is active"
        );
        let inner = state.inner.lock().unwrap();
        assert_eq!(serde_json::to_string(&inner.stack).unwrap(), expected);
        assert!(inner.stack.active.is_none());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "",
            "no transition is appended on restart"
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

    /// **Rewritten for ADR 0007, not weakened.** This previously asserted that
    /// a leftover active entry is closed and *nothing* resumes. That was ADR
    /// 0005 open item 9, which ADR 0007 supersedes: within the resume limit the
    /// task now restarts. What has not changed, and is still asserted, is that
    /// the *closed* block gets an inferred end and that a paused frame is left
    /// untouched by recovery.
    #[test]
    fn leftover_active_entry_is_closed_as_recovered_gap_and_resumed_within_the_limit() {
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
        // The writes above are immediate, so the gap is under the continuity
        // threshold and the block simply carries on — the case that used to
        // produce a zero-duration block on this path but not on the wake path.
        assert!(!report.startup_gap_recovered, "a sub-threshold outage is not a gap");

        let inner = state.inner.lock().unwrap();
        assert_eq!(
            inner.stack.active.as_ref().map(|b| b.name.as_str()),
            Some("B"),
            "the block was never interrupted, so it is still the active one"
        );
        // "A" (paused on the stack) is untouched by recovery either way — it is
        // not the active entry, so it stays pending until explicitly resumed.
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
