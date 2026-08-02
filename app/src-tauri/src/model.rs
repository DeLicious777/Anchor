use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// **How was this block's end time established?** — and nothing else.
///
/// Replaced `CompletionReason` (ADR 0005, migrated 2026-07-29), which conflated
/// three independent questions. The clearest symptom was
/// `auto-completed-on-skip`: it described the fate of a *stack frame*, not a
/// reason a block ended. `explicit` was equally muddled — it was documented as
/// "user-finished" while the code wrote it for Switch-ended and Return-ended
/// blocks too, which were user-*ended*, not user-*finished*.
///
/// Under this field, `Switch` closing a block as `UserDetermined` is simply
/// correct: the user's own action fixed the moment. Only inference differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EndDetermination {
    /// The user's action fixed the end moment — Complete, Switch, Interrupt, or
    /// either Return. Exact.
    UserDetermined,
    /// Anchor inferred it after a detected gap (crash, or sleep/hibernate),
    /// accurate to roughly the heartbeat interval. Was `recovered-gap`.
    SystemInferred,
}

/// **How did this block enter the system, and how much do we trust it?**
///
/// Two axes in one wire value: where the block came from, and whether it has
/// since been adjusted. Kept as a flat enum rather than nested fields so the
/// serialised form stays a single string, with `origin()`/`is_adjusted()`
/// recovering the axes.
///
/// Origin survives adjustment deliberately. A manually entered block that later
/// got a one-second nudge must stay distinguishable from a live capture that
/// needed correcting — collapsing both to "adjusted" would lose the more
/// important signal. See `docs/vision/vision.md`'s Capture Rate, which counts
/// live-captured minutes *including* adjusted ones, precisely so the metric
/// measures capture discipline rather than editing habits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureOrigin {
    /// Captured as the work happened, never since modified.
    LiveCapture,
    /// Captured live, then edited or confirmed by the user.
    LiveCaptureAdjusted,
    /// Reconstructed after the fact; never existed as a live capture.
    ManualEntry,
    /// Reconstructed after the fact, then further edited.
    ManualEntryAdjusted,
}

/// Where a block came from, ignoring whether it was later adjusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureSource {
    Live,
    Manual,
}

impl CaptureOrigin {
    pub fn origin(self) -> CaptureSource {
        match self {
            Self::LiveCapture | Self::LiveCaptureAdjusted => CaptureSource::Live,
            Self::ManualEntry | Self::ManualEntryAdjusted => CaptureSource::Manual,
        }
    }

    pub fn is_adjusted(self) -> bool {
        matches!(self, Self::LiveCaptureAdjusted | Self::ManualEntryAdjusted)
    }

    /// Idempotent: adjusting an already-adjusted block changes nothing, and
    /// adjustment never rewrites origin.
    pub fn adjusted(self) -> Self {
        match self {
            Self::LiveCapture | Self::LiveCaptureAdjusted => Self::LiveCaptureAdjusted,
            Self::ManualEntry | Self::ManualEntryAdjusted => Self::ManualEntryAdjusted,
        }
    }
}

/// **What ultimately happened to this interrupted work?**
///
/// Absorbs the old `auto-completed-on-skip`. Deliberately has no `Pending`
/// variant: an unresolved obligation is represented by the stack frame itself,
/// which the snapshot persists (ADR 0005). Persist stable facts, derive
/// transient process state.
///
/// Consequence: an absent value is genuinely ambiguous — *never interrupted* or
/// *interrupted and not yet resolved*. **No view may read this field directly**;
/// use `InterruptionStack::derived_status`, which disambiguates against the
/// live stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterruptionOutcome {
    /// The user returned to this work.
    Resumed,
    /// Interrupted and never resumed — whether by jumping past it via Return to
    /// Original Task, or by explicitly dismissing its frame. Same domain fact,
    /// different route (`docs/principles.md` #6).
    Skipped,
}

/// The canonical, **non-persisted** projection every consumer must use to
/// display interruption state — History View, Timeline Editor, diagnostics.
///
/// Exists because `InterruptionOutcome`'s absence is ambiguous on its own, and
/// no UI or export may invent its own interpretation of it. Computed by
/// `InterruptionStack::derived_status` from the block plus the *current* stack.
///
/// Deliberately NOT included in exports: it is computed against live state, so
/// embedding it would make an export of last Tuesday yield different values
/// depending on when it ran — unacceptable in an artifact that has to be
/// reproducible for billing (ADR 0005 open item 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DerivedInterruptionStatus {
    NeverInterrupted,
    /// Interrupted, obligation still open — its frame is on the stack.
    Pending,
    Resumed,
    Skipped,
}

/// The atomic tracked-work record. Independent flat entry — no persistent link to
/// other Time Blocks of the same task; aggregation happens at export time by
/// matching name/project/client (see docs/concept/concept.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlock {
    pub id: Uuid,
    pub name: String,
    pub project: Option<String>,
    pub client: Option<String>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    /// `None` while the block is still active — an end time that hasn't
    /// happened yet has no determination.
    pub end_determination: Option<EndDetermination>,
    pub capture_origin: CaptureOrigin,
    /// `None` means *never interrupted* OR *interrupted and unresolved* — read
    /// via `InterruptionStack::derived_status`, never directly.
    pub interruption_outcome: Option<InterruptionOutcome>,
}

/// The fixed UUIDv5 namespace Time Block identity is derived under
/// ([ADR 0006](../../../docs/decisions/0006-stable-persistent-time-block-identity.md)).
///
/// **Chosen once and never changed.** It is part of a durable on-disk contract
/// from the moment any reconstruction transition references a derived id —
/// altering it would silently re-identify every block in every log, orphaning
/// every stored reference and leaving the app unable to start.
pub const ANCHOR_NAMESPACE: Uuid = Uuid::from_bytes([
    0xbe, 0xcb, 0xca, 0x30, 0x95, 0x8b, 0x45, 0x68, 0xa9, 0xec, 0xdd, 0x5e, 0xd1, 0xdb, 0xf6, 0x12,
]);

/// Derive a Time Block's identity from the `seq` of the transition that created
/// it, per ADR 0006.
///
/// **The encoding is as load-bearing as the namespace and is equally fixed:**
/// the hashed name is `seq`'s ASCII decimal form, unpadded and unprefixed.
/// UUIDv5 hashes a byte string, so `b"42"`, `42u64.to_be_bytes()` and
/// `42u64.to_le_bytes()` yield three different ids from the same number.
/// Decimal is chosen because it matches how `seq` already appears in the log's
/// JSON, so an id can be reproduced by hand from a log line.
pub fn time_block_id(seq: u64) -> Uuid {
    Uuid::new_v5(&ANCHOR_NAMESPACE, seq.to_string().as_bytes())
}

impl TimeBlock {
    /// `seq` is the sequence number of the transition creating this block —
    /// the sole input to its identity (ADR 0006). Replay passes the `seq` it
    /// read; the live path passes the `seq` the writer is about to assign.
    /// Both therefore produce the same id for the same block, on every replay,
    /// forever — which is what makes a block referenceable across restarts.
    pub fn new(name: String, project: Option<String>, client: Option<String>, start: DateTime<Utc>, seq: u64) -> Self {
        Self {
            id: time_block_id(seq),
            name,
            project,
            client,
            start,
            end: None,
            // Every block the state machine creates is captured live, by
            // definition — it is being created as the work happens. Manual
            // entry is the exception path (timeline reconstruction, #15), and
            // will construct blocks that say so.
            capture_origin: CaptureOrigin::LiveCapture,
            end_determination: None,
            interruption_outcome: None,
        }
    }

    /// Duration is derived from start/end, never stored — avoids a redundant field
    /// that could drift out of sync with the two timestamps it's computed from.
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end.map(|end| end - self.start)
    }
}

/// A reusable preset pre-filling a Time Block's name/project/client for fast
/// starts on recurring activities (see `docs/product/features/task-templates.md`).
/// Deliberately never referenced by `TimeBlock` — a template only ever pre-fills
/// the quick-input's plain string fields, so editing/deleting one can never
/// retroactively affect an already-recorded Time Block; that guarantee falls out
/// of this struct simply not being linked, not from any enforcement elsewhere.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskTemplate {
    pub id: Uuid,
    pub name: String,
    pub project: Option<String>,
    pub client: Option<String>,
}

impl TaskTemplate {
    pub fn new(name: String, project: Option<String>, client: Option<String>) -> Self {
        Self { id: Uuid::new_v4(), name, project, client }
    }
}

/// A task paused by an Interrupt, waiting on the stack to be resumed or skipped.
/// Carries enough identity to start a brand-new Time Block when it's eventually
/// resumed — resuming never reopens the original block (see conversation:
/// each pause/resume cycle is a new, independently-timed Time Block).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// The already-closed, pending-completion-reason Time Block this frame paused.
    pub paused_time_block_id: Uuid,
    pub name: String,
    pub project: Option<String>,
    pub client: Option<String>,
}

/// One line in the transition log, per ADR 0004. Deliberately minimal: only what's
/// needed to deterministically replay `InterruptionStack::apply()` — resolved
/// state (which blocks close, which frames get pushed/popped) is derived by the
/// state machine itself, not duplicated into the log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    /// Every line gets one, including heartbeats — required for watermark-based
    /// replay filtering (ADR 0004). Distinct from the lifecycle-only counter the
    /// writer tracks in memory to decide when to trigger compaction.
    pub seq: u64,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub payload: TransitionPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TransitionPayload {
    Start { name: String, project: Option<String>, client: Option<String> },
    Switch { name: String, project: Option<String>, client: Option<String> },
    Interrupt { name: String, project: Option<String>, client: Option<String> },
    /// Changes the currently active Time Block's identity fields in place —
    /// no new block, no stack effect, start time untouched. Lets a task
    /// started without a name (see `InterruptionStack::next_default_name`)
    /// be renamed, or renamed to an existing template/past task name, while
    /// it's still running.
    Rename { name: String, project: Option<String>, client: Option<String> },
    ReturnPrevious,
    ReturnOriginal,
    Complete,
    Heartbeat,
    /// Closes the active entry as `recovered-gap` with an explicitly-carried end
    /// time (not "now" — the gap was detected after the fact, whether at startup
    /// after a crash or on live resume from sleep/hibernate). Deliberately does
    /// NOT start a new active entry itself: startup recovery and live-resume
    /// recovery differ on whether to auto-resume, so that decision is made by the
    /// caller (see `state::resolve_startup_gap` vs. the Slice 2 power-resume path),
    /// not baked into this transition.
    RecoverGap { inferred_end: DateTime<Utc> },
    /// Corrects the name/project/client of a Time Block that has **already
    /// finished** — the historical counterpart to `Rename`, and deliberately a
    /// separate transition rather than a generalisation of it.
    ///
    /// The two produce identical state and are kept distinct because they answer
    /// different questions about how that state came to be: `Rename` changes the
    /// identity of work that is *still happening*, this corrects work that *has
    /// happened*. Merging them would supersede a shipped, accepted transition
    /// and `interruption-stack.md`'s "requires an active task" rule for a
    /// modelling-tidiness gain (`timeline-reconstruction.md`, alternative D).
    ///
    /// `target` is the block's derived id (ADR 0006), which is why this could
    /// not exist before that scheme was implemented — a `Uuid` regenerated per
    /// replay cannot name a block written in an earlier session.
    EditIdentity {
        target: Uuid,
        name: String,
        project: Option<String>,
        client: Option<String>,
    },
    /// Creates a Time Block for work that happened but was never captured
    /// (risk **R3**) — the first transition to carry author-chosen boundaries
    /// rather than deriving them from when it was logged.
    ///
    /// Always an independent block: it never touches the interruption stack,
    /// carries no `InterruptionOutcome`, and pushes no frame. There is
    /// deliberately no way to reconstruct an interruption *relationship* after
    /// the fact — the stack records what the user actually did in the moment,
    /// and inventing one retroactively would fabricate provenance.
    Add {
        name: String,
        project: Option<String>,
        client: Option<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    /// Translates a block to a new start, **preserving its duration exactly**.
    ///
    /// Carries only the new `start` on purpose: duration preservation is Move's
    /// entire problem statement ("a 60-minute meeting recorded 30 minutes early
    /// has the right duration in the wrong place"), so the payload is shaped so
    /// it *cannot* express a duration change. The end is derived from the
    /// block's own span at replay time, which is deterministic because that span
    /// is itself a product of earlier transitions.
    Move { target: Uuid, start: DateTime<Utc> },
    /// Reshapes a block's span — the mechanism risks **R9** and **R4** have been
    /// promised since 2026-07-23 and never had.
    ///
    /// Distinct from `Move` even though both can produce the same state, for the
    /// same reason `EditIdentity` is distinct from `Rename`: they answer
    /// different questions about how that state came to be. Resize says *the
    /// timing was wrong*; Move says *the duration was right and the position was
    /// wrong*.
    Resize {
        target: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    /// Removes a Time Block from the timeline — for work that never happened,
    /// e.g. tracking started by mistake.
    ///
    /// **Physical removal, not a tombstone.** Tombstoning was designed and
    /// reviewed in full and deliberately not adopted: it buys durable,
    /// restart-surviving undo that no accepted requirement asks for, while
    /// creating obligations across export, Capture Rate, the snapshot payload,
    /// the reconstruction domain and `next_default_name`. MVP has **no undo**;
    /// Delete is confirmed instead, and a mistaken delete is recovered by
    /// re-adding (`timeline-reconstruction.md`).
    Delete { target: Uuid },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The durable contract, pinned by assertion rather than by prose.**
    ///
    /// ADR 0006 fixes both the namespace and the encoding, and states the test
    /// vectors these assert. A change to either — including a plausible-looking
    /// byte transposition in the constant, which is exactly what this caught
    /// while it was being written — silently re-identifies every block in every
    /// log. Nothing else in the codebase would notice.
    #[test]
    fn time_block_identity_matches_adr_0006s_test_vectors() {
        assert_eq!(
            ANCHOR_NAMESPACE.to_string(),
            "becbca30-958b-4568-a9ec-dd5ed1dbf612",
            "the namespace constant must equal the one ADR 0006 fixes"
        );

        for (seq, expected) in [
            (0u64, "18885148-20dd-5b7c-a7d3-d7844af7a220"),
            (1, "24d50f72-3754-5b11-8e57-ec5329a394af"),
            (42, "7d8aafb1-dfaf-59a2-bab1-d1a0a0f3e7f7"),
        ] {
            assert_eq!(time_block_id(seq).to_string(), expected, "id for seq {seq}");
        }
    }

    /// The property the whole scheme exists for: the same log yields the same
    /// ids every time it is read, so a reference written in one session still
    /// resolves in the next.
    #[test]
    fn identity_is_stable_across_derivations_and_unique_per_seq() {
        assert_eq!(time_block_id(7), time_block_id(7), "same seq, same id, always");
        assert_ne!(time_block_id(7), time_block_id(8));

        let ids: std::collections::HashSet<_> = (0..1_000).map(time_block_id).collect();
        assert_eq!(ids.len(), 1_000, "no collisions across a realistic run of sequence numbers");
    }
}
