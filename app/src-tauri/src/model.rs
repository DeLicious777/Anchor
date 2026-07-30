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

impl TimeBlock {
    pub fn new(name: String, project: Option<String>, client: Option<String>, start: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
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
}
