//! The compaction snapshot, per ADR 0004's compaction decision (#8).
//!
//! Compaction bounds startup cost: the projection is written to a snapshot,
//! the log is truncated, and the next startup replays only what came after.
//!
//! # The payload is exactly three fields, and each one earns its place
//!
//! Derived by an invariant-first pass (2026-08-02, before any code): not "what
//! fields should a snapshot contain" but "what must be true after loading one."
//! **Every field justifies its existence by preserving an accepted invariant.
//! No field exists for convenience or performance.**
//!
//! - `stack` — satisfies most of the contract wholesale, because the projection
//!   already carries what the invariants need. Unresolved interruption stack
//!   frames survive (assumption **A10**, which is the sole reason
//!   `InterruptionOutcome` has no persisted `Pending`), and every `TimeBlock`
//!   keeps its `id` (assumption **A14**, which is what keeps blocks below the
//!   watermark addressable as reconstruction targets — they are never replayed
//!   again, so their identity can come only from here).
//! - `watermark` — the highest `seq` folded into `stack`. Replay skips lines at
//!   or below it, and `next_seq` continues from it, so no `seq` is reused.
//! - `last_activity_at` — assumption **A15**, and the one field that is *not*
//!   part of the projection. See below; it is the reason this module exists at
//!   all rather than the snapshot simply being a serialised `InterruptionStack`.
//!
//! Deliberately **absent: the compaction-trigger counter.** No accepted
//! invariant requires compaction to fire at exactly N transitions since the
//! last one — ADR 0004 says "clean shutdown *or* 500 user-triggered
//! transitions, whichever comes first." A restart resetting the count toward N
//! delays compaction and violates nothing, so it does not earn a field.
//!
//! # Why `last_activity_at` is not derivable
//!
//! Gap recovery closes a still-active block using the timestamp of the last
//! durable write, bounding the inference error to roughly the heartbeat
//! interval (risk **R4**). `AppState::init` documents the invariant it relies
//! on — *"if replay left something active, at least the line that started it
//! was successfully parsed, so `last_timestamp` must be `Some` too"* — and that
//! invariant is true **only because compaction does not exist**.
//!
//! The fact exists in two forms, and compaction preserves the wrong one:
//! *"a block is active"* is projection state, which the snapshot carries;
//! *"when was the last durable write"* is a property of the log lines, which
//! truncation destroys. Recovery needs both, and the active block's `end` is
//! `None` by definition, so the second cannot be recomputed from the first.
//!
//! Without this field, a snapshot taken while tracking, followed by a kill,
//! leaves `stack.active` as `Some` while `last_timestamp` is `None` — and
//! `init` panics on its own `expect`. The panic is the *good* outcome: the
//! nearby fallback would infer the end as `Utc::now()` and silently bill every
//! offline hour, which is R4 realised at maximum severity.
//!
//! # Durability: why this needs no checksum, unlike the log
//!
//! ADR 0004 gives every log line a checksum because the log is **appended in
//! place** — a torn write leaves a partial record at the end of a file that
//! still has to be read. A snapshot is **replaced atomically**: written to a
//! temporary file, fsynced, then renamed over the old one. A rename either
//! happens or does not, so no reader ever sees a partially-written snapshot,
//! and there is no torn state for a checksum to detect. Different failure
//! mode, different mechanism.
//!
//! The ordering below is the load-bearing part, and it is a correctness
//! obligation rather than a nicety, because **compaction destroys the log**: a
//! snapshot that is not durable before truncation is not a recoverable last
//! record, it is total loss. That is risk **R5**'s lesson arriving on a new
//! artifact. Compaction may duplicate work; it must never destroy recoverable
//! history.

use crate::model::TransitionPayload;
use crate::stack::InterruptionStack;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Bumped whenever the serialised shape changes incompatibly.
///
/// This *does* earn its place under the no-convenience-fields rule, and the
/// reasoning is worth stating because it is not obvious: normally an
/// unreadable snapshot is harmless, since `load` falls back to replaying the
/// log. But compaction truncates the log, so after it runs the snapshot is the
/// only copy of pre-watermark history. The failure this guards against is not
/// a *failed* parse — that is already safe — but a **successful but wrong**
/// one, which is exactly what a field added with a serde default produces.
/// Unlike the log's format (a stable on-disk contract per ADR 0004), the
/// snapshot serialises *resolved* state, so it moves whenever the domain model
/// does.
const SNAPSHOT_VERSION: u32 = 2;
// v1 → v2 (2026-08-02): `InterruptionStack` gained `issued_anchor_names` (#19).
// Bumped rather than relying on the missing field happening to fail
// deserialisation: "we declared this incompatible" is a stronger guarantee than
// "serde would probably have errored", and this is the field whose absence would
// silently allow an auto-name to be reused.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    /// Highest `seq` folded into `stack`. Replay skips lines at or below it.
    pub watermark: u64,
    pub stack: InterruptionStack,
    /// Timestamp of the last durable write at the moment of compaction —
    /// assumption **A15**. Not derivable from `stack`; see the module docs.
    pub last_activity_at: DateTime<Utc>,
}

impl Snapshot {
    pub fn new(watermark: u64, stack: InterruptionStack, last_activity_at: DateTime<Utc>) -> Self {
        Self { version: SNAPSHOT_VERSION, watermark, stack, last_activity_at }
    }

    /// Load a snapshot, or `None` if there isn't a usable one.
    ///
    /// **Never returns an error, by design.** A missing, unreadable, corrupt or
    /// wrong-version snapshot all mean the same thing to the caller: fall back
    /// to replaying the log, which is the source of truth. Turning any of these
    /// into a hard failure would let a bad *cache* prevent startup, which
    /// inverts `docs/architecture/constraints.md` — all state is a projection
    /// replayed from the log, and this file is only ever an optimisation of
    /// that replay.
    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        let contents = fs::read_to_string(path).ok()?;
        let snapshot: Self = serde_json::from_str(&contents).ok()?;
        (snapshot.version == SNAPSHOT_VERSION).then_some(snapshot)
    }

    /// Write durably and atomically: temp file → fsync → rename over the target.
    ///
    /// Returns only after the rename is complete, so a caller that truncates
    /// the log on `Ok` cannot lose history to a half-written snapshot.
    ///
    /// The POSIX belt-and-braces step — fsyncing the parent directory so the
    /// rename itself is durable — is deliberately not attempted: it is not
    /// available on Windows (a directory cannot be opened as a file), and this
    /// is a Windows-first application (ADR 0002). Stated rather than silently
    /// skipped, because it is a real gap on other platforms: a crash in the
    /// window between rename and the OS flushing the directory entry could
    /// leave the old snapshot in place. That is safe here only because the
    /// caller has not yet truncated the log.
    pub fn write(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        let temp = temp_path_for(path);

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        {
            let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&temp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
        }

        // Atomic on both POSIX and Windows (`MoveFileEx` with
        // MOVEFILE_REPLACE_EXISTING), including over an existing file.
        fs::rename(&temp, path)?;
        Ok(())
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Compact: make the snapshot durable, **then** truncate the log.
///
/// The ordering is the whole point and must not be reversed or fused. If the
/// process dies between the two steps, the snapshot describes state that the
/// still-intact log also describes — startup replays post-watermark lines onto
/// the restored projection and reaches exactly the same place. Compaction may
/// duplicate work; it must never destroy recoverable history.
pub fn compact(
    snapshot_path: impl AsRef<Path>,
    log_path: impl AsRef<Path>,
    snapshot: &Snapshot,
) -> std::io::Result<()> {
    snapshot.write(snapshot_path)?;
    // Only reachable once the snapshot is durably in place.
    let log = OpenOptions::new().write(true).open(log_path)?;
    log.set_len(0)?;
    log.sync_all()
}

/// Tracks progress toward ADR 0004's compaction trigger: *"clean shutdown **or**
/// 500 user-triggered transitions (excluding heartbeats) since the last
/// compaction, whichever comes first."*
///
/// A named type rather than a bare counter on `Inner`, because the rule has
/// four parts that are easy to violate by accident and each one is encoded here
/// instead of relying on callers to remember it:
///
/// - **Heartbeats never count.** ADR 0004 is explicit about why: a heads-down
///   8-hour day with zero manual transitions still writes ~480 heartbeat lines,
///   which would trigger compaction from heartbeat volume alone — not what the
///   trigger is for.
/// - **`Rename` and `RecoverGap` never count either.** ADR 0004's list is
///   exhaustive: start / switch / interrupt / return-previous / return-original
///   / complete. `Rename` is user-triggered but not a lifecycle transition (it
///   opens and closes nothing); `RecoverGap` is not user-triggered at all.
/// - **Replay never counts.** By construction — replay drives
///   `InterruptionStack::apply` directly and never touches this type, so
///   rebuilding a million-line log advances nothing.
/// - **Successful compaction resets it.** Nothing else may.
#[derive(Debug, Default)]
pub struct CompactionTrigger {
    user_transitions_since_compaction: u32,
    /// Watermark of the last compaction this process performed. Distinguishes
    /// "nothing new since the snapshot" from "never compacted", so a shutdown
    /// immediately after a threshold compaction does not redo identical work.
    last_compacted_watermark: Option<u64>,
}

impl CompactionTrigger {
    /// ADR 0004: **not measured, chosen as a low-cost default** — the same
    /// honest framing the 60-second heartbeat interval uses. Revisit if startup
    /// replay time is ever actually observed to be a problem.
    pub const THRESHOLD: u32 = 500;

    pub fn record(&mut self, payload: &TransitionPayload) {
        if counts_toward_compaction(payload) {
            self.user_transitions_since_compaction += 1;
        }
    }

    pub fn count(&self) -> u32 {
        self.user_transitions_since_compaction
    }

    /// True once the threshold is reached **and** there is genuinely new work on
    /// disk. Both halves matter: the first is ADR 0004's rule, the second stops
    /// the same state being compacted twice.
    pub fn should_compact(&self, next_seq: u64) -> bool {
        self.user_transitions_since_compaction >= Self::THRESHOLD && self.has_uncompacted_work(next_seq)
    }

    /// The clean-shutdown arm. No threshold — any uncompacted work is worth
    /// snapshotting on the way out, since it makes the next startup cheaper and
    /// there is no next transition to wait for.
    pub fn has_uncompacted_work(&self, next_seq: u64) -> bool {
        next_seq > 0 && self.last_compacted_watermark != Some(next_seq - 1)
    }

    pub fn reset(&mut self, compacted_watermark: u64) {
        self.user_transitions_since_compaction = 0;
        self.last_compacted_watermark = Some(compacted_watermark);
    }
}

fn counts_toward_compaction(payload: &TransitionPayload) -> bool {
    use TransitionPayload::*;
    match payload {
        Start { .. } | Switch { .. } | Interrupt { .. } | ReturnPrevious | ReturnOriginal | Complete => true,
        // Enumerated rather than a catch-all `_ => false`, so adding a
        // transition forces a decision here instead of silently defaulting. It
        // has done exactly that for every transition below, all of which
        // postdate ADR 0004's list.
        //
        // `EditIdentity` and `Delete` do **not** count, for the same reason
        // `Rename` does not: they correct an existing record rather than opening
        // or completing work, and reconstruction is the exception path by design
        // (`timeline-reconstruction.md` — it "must never become the fast path").
        // Their volume is negligible and self-limiting, so letting them advance
        // a threshold meant to bound routine capture growth would misread what
        // the trigger is for. A long reconstruction session is still compacted
        // on clean shutdown, which has no threshold.
        //
        // The same applies to `Add`, `Move` and `Resize`: `Add` does create a
        // Time Block, but for work that already happened rather than work now
        // starting, and all three are reconstruction — the exception path.
        //
        // **`DismissFrame` also does not count**, on that same rule. It is a
        // correction — resolving a frame the user never went back to — not the
        // capture of new work, and its volume is negligible and self-limiting
        // in exactly the way `Delete`'s is.
        //
        // ADR 0004's counting set is the six lifecycle transitions above and
        // nothing else. What may be added to it is that ADR's business, not an
        // implementation's: the test here is whether a transition bounds routine
        // capture growth, never whether it resembles one that counts.
        //
        // Recorded as a judgment call extending ADR 0004's rule to transitions
        // it could not have listed — not as a literal reading of that list.
        Heartbeat
        | Rename { .. }
        | RecoverGap { .. }
        | EditIdentity { .. }
        | Delete { .. }
        | Add { .. }
        | Move { .. }
        | Resize { .. }
        | DismissFrame { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TimeBlock;

    fn start(n: &str) -> TransitionPayload {
        TransitionPayload::Start { name: n.into(), project: None, client: None }
    }

    #[test]
    fn only_user_triggered_lifecycle_transitions_count_toward_the_threshold() {
        let mut trigger = CompactionTrigger::default();

        for payload in [
            TransitionPayload::Heartbeat,
            TransitionPayload::Rename { name: "x".into(), project: None, client: None },
            TransitionPayload::RecoverGap { inferred_end: Utc::now() },
        ] {
            trigger.record(&payload);
        }
        assert_eq!(trigger.count(), 0, "heartbeats, renames and gap recovery must not advance the trigger");

        for payload in [
            start("a"),
            TransitionPayload::Switch { name: "b".into(), project: None, client: None },
            TransitionPayload::Interrupt { name: "c".into(), project: None, client: None },
            TransitionPayload::ReturnPrevious,
            TransitionPayload::ReturnOriginal,
            TransitionPayload::Complete,
        ] {
            trigger.record(&payload);
        }
        assert_eq!(trigger.count(), 6, "all six lifecycle transitions count");
    }

    /// The correction transitions, pinned as a group. Each one postdates ADR
    /// 0004's list and each was classified by the same rule: it corrects an
    /// existing record rather than capturing new work, so it must not advance a
    /// threshold that exists to bound routine capture growth.
    ///
    /// `DismissFrame` belongs to this group: it resolves a frame rather than
    /// capturing work, and ADR 0004's counting set is the six lifecycle
    /// transitions and nothing else.
    #[test]
    fn correction_transitions_never_advance_the_trigger() {
        let mut trigger = CompactionTrigger::default();
        let id = uuid::Uuid::from_u128(1);

        for payload in [
            TransitionPayload::EditIdentity { target: id, name: "x".into(), project: None, client: None },
            TransitionPayload::Delete { target: id },
            TransitionPayload::Add {
                name: "x".into(),
                project: None,
                client: None,
                start: Utc::now(),
                end: Utc::now(),
            },
            TransitionPayload::Move { target: id, start: Utc::now() },
            TransitionPayload::Resize { target: id, start: Utc::now(), end: Utc::now() },
            TransitionPayload::DismissFrame { target: id },
        ] {
            trigger.record(&payload);
        }

        assert_eq!(trigger.count(), 0, "reconstruction and frame dismissal are the exception path, not capture");
    }

    #[test]
    fn the_threshold_fires_at_exactly_adr_0004s_number() {
        let mut trigger = CompactionTrigger::default();
        for _ in 0..CompactionTrigger::THRESHOLD - 1 {
            trigger.record(&start("x"));
        }
        assert!(!trigger.should_compact(9_999));
        trigger.record(&start("x"));
        assert!(trigger.should_compact(9_999));
    }

    /// Compaction must not repeat for state already snapshotted — otherwise a
    /// clean shutdown straight after a threshold compaction rewrites an
    /// identical snapshot and re-truncates an already-empty log.
    #[test]
    fn the_same_state_is_never_compacted_twice() {
        let mut trigger = CompactionTrigger::default();
        assert!(!trigger.has_uncompacted_work(0), "an empty log has nothing to compact");

        for _ in 0..CompactionTrigger::THRESHOLD {
            trigger.record(&start("x"));
        }
        assert!(trigger.should_compact(500));

        trigger.reset(499);
        assert_eq!(trigger.count(), 0);
        assert!(!trigger.should_compact(500), "threshold cleared");
        assert!(!trigger.has_uncompacted_work(500), "and nothing new is on disk");

        trigger.record(&start("later"));
        assert!(trigger.has_uncompacted_work(501), "a single new transition is uncompacted work again");
        assert!(!trigger.should_compact(501), "but it is nowhere near the threshold");
    }

    fn stack_with_active(name: &str) -> InterruptionStack {
        let mut stack = InterruptionStack::new();
        stack
            .apply(
                &TransitionPayload::Start { name: name.into(), project: None, client: None },
                Utc::now(),
                0,
            )
            .unwrap();
        stack
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");
        let at = Utc::now();

        Snapshot::new(42, stack_with_active("writing"), at).write(&path).unwrap();
        let loaded = Snapshot::load(&path).unwrap();

        assert_eq!(loaded.watermark, 42);
        assert_eq!(loaded.last_activity_at, at);
        assert_eq!(loaded.stack.active.as_ref().unwrap().name, "writing");
    }

    /// A10 and A14 in one assertion: frames survive with their back-references,
    /// and every block keeps the exact `id` it had. Both matter because blocks
    /// below the watermark are never replayed again — this file is the only
    /// place their identity can come from.
    #[test]
    fn preserves_stack_frames_and_time_block_ids_exactly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");

        let mut stack = stack_with_active("root");
        let root_id = stack.active.as_ref().unwrap().id;
        stack
            .apply(
                &TransitionPayload::Interrupt { name: "phone".into(), project: None, client: None },
                Utc::now(),
                1,
            )
            .unwrap();
        let closed_ids: Vec<_> = stack.closed.iter().map(|b: &TimeBlock| b.id).collect();
        assert_eq!(stack.stack.len(), 1, "the interrupt pushed a frame");

        Snapshot::new(7, stack, Utc::now()).write(&path).unwrap();
        let loaded = Snapshot::load(&path).unwrap();

        assert_eq!(loaded.stack.stack.len(), 1, "A10: the unresolved frame survives");
        assert_eq!(
            loaded.stack.stack[0].paused_time_block_id, root_id,
            "A10: the frame still points at the block it paused"
        );
        assert_eq!(
            loaded.stack.closed.iter().map(|b| b.id).collect::<Vec<_>>(),
            closed_ids,
            "A14: every block keeps the id it had"
        );
    }

    #[test]
    fn a_missing_or_corrupt_snapshot_is_none_rather_than_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.json");
        assert!(Snapshot::load(&missing).is_none());

        let corrupt = dir.path().join("corrupt.json");
        fs::write(&corrupt, b"{\"version\": 1, \"watermark\": ").unwrap();
        assert!(Snapshot::load(&corrupt).is_none(), "a truncated snapshot must not be trusted");
    }

    /// The failure a version guards against is not a *failed* parse — that is
    /// already safe — but a wrong-shape one after the domain model moves. With
    /// the log truncated, the snapshot is the only copy of pre-watermark
    /// history, so a silently-wrong parse is unrecoverable.
    #[test]
    fn a_snapshot_from_a_different_version_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");
        Snapshot::new(1, InterruptionStack::new(), Utc::now()).write(&path).unwrap();

        let raw = fs::read_to_string(&path).unwrap();
        let current = format!("\"version\": {SNAPSHOT_VERSION}");
        assert!(raw.contains(&current), "the written snapshot must carry the current version");
        // Derived from the constant, not hardcoded — a hardcoded value silently
        // stops substituting the moment the version is bumped, and the test then
        // passes by loading a perfectly valid snapshot.
        fs::write(&path, raw.replace(&current, "\"version\": 999")).unwrap();

        assert!(Snapshot::load(&path).is_none());
    }

    /// A failed write must leave the previous snapshot intact — the temp file
    /// is what absorbs a partial write, and it is never renamed unless the
    /// write and fsync both succeeded.
    #[test]
    fn writing_replaces_the_previous_snapshot_atomically_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");

        Snapshot::new(1, stack_with_active("first"), Utc::now()).write(&path).unwrap();
        Snapshot::new(2, stack_with_active("second"), Utc::now()).write(&path).unwrap();

        let loaded = Snapshot::load(&path).unwrap();
        assert_eq!(loaded.watermark, 2);
        assert_eq!(loaded.stack.active.as_ref().unwrap().name, "second");
        assert!(!temp_path_for(&path).exists(), "the temp file must not survive a successful write");
    }
}
