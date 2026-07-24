use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How a Time Block's boundary was determined. `None` on a `TimeBlock` means the
/// block was closed by an Interrupt but its fate (explicit return vs. skipped) is
/// not yet decided — see `docs/product/features/interruption-stack.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompletionReason {
    Explicit,
    AutoCompletedOnSkip,
    RecoveredGap,
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
    pub completion_reason: Option<CompletionReason>,
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
            completion_reason: None,
        }
    }

    /// Duration is derived from start/end, never stored — avoids a redundant field
    /// that could drift out of sync with the two timestamps it's computed from.
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end.map(|end| end - self.start)
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
    ReturnPrevious,
    ReturnOriginal,
    Complete,
    /// Not implemented this slice (Slice 2: heartbeat timer) — included now so the
    /// on-disk schema doesn't need a breaking change later.
    Heartbeat,
}
