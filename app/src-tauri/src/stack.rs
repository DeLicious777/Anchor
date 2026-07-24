//! The interruption stack state machine. Pure in-memory logic, no I/O — this is
//! deliberately the ONE place "what does an Interrupt/Return mean" lives, so that
//! live commands and log replay can never diverge (see `log::reader`).
//!
//! Confirmed semantics (see conversation, 2026-07-24):
//! - Interrupting a task closes its Time Block IMMEDIATELY (end = now), giving it
//!   an accurate duration that never includes time spent on whatever interrupted
//!   it. Its `completion_reason` stays pending (`None`) until its fate is decided.
//! - Resuming a paused task (via either Return operation) never reopens the
//!   original Time Block — it starts a brand-new one. Each pause/resume cycle is
//!   an independent, flat entry (per the accepted Concept: "each block counts as
//!   an independent entry, aggregated at export time").
//! - Return to Previous resolves exactly the frame it pops as `explicit`.
//! - Return to Original resolves every skipped frame as `auto-completed-on-skip`,
//!   and the root frame it lands on as `explicit` (it was directly engaged with,
//!   same as Return to Previous's target).

use crate::model::{CompletionReason, StackFrame, TimeBlock, TransitionPayload};
use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StackError {
    #[error("no active task to act on")]
    NoActiveTask,
    #[error("a task is already active — complete, switch, or interrupt it first")]
    AlreadyActive,
    #[error("the interruption stack is empty — nothing to return to")]
    StackEmpty,
    #[error("cannot Complete while the interruption stack is non-empty — return to it first")]
    CannotCompleteWithOpenStack,
    #[error("internal inconsistency: paused Time Block {0} not found among closed blocks")]
    PausedBlockNotFound(Uuid),
}

#[derive(Debug, Default, Clone)]
pub struct InterruptionStack {
    pub active: Option<TimeBlock>,
    pub stack: Vec<StackFrame>,
    /// Every Time Block that is no longer active, in the order it closed.
    /// A `completion_reason` of `None` means "closed by an Interrupt, fate not
    /// yet decided" — see module docs.
    pub closed: Vec<TimeBlock>,
}

impl InterruptionStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Apply one transition. The single entry point used by both live commands
    /// and log replay, so the two paths can never disagree about what a
    /// transition means.
    pub fn apply(&mut self, payload: &TransitionPayload, timestamp: DateTime<Utc>) -> Result<(), StackError> {
        match payload {
            TransitionPayload::Start { name, project, client } => {
                if self.active.is_some() {
                    return Err(StackError::AlreadyActive);
                }
                self.active = Some(TimeBlock::new(name.clone(), project.clone(), client.clone(), timestamp));
                Ok(())
            }
            TransitionPayload::Switch { name, project, client } => {
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(timestamp);
                current.completion_reason = Some(CompletionReason::Explicit);
                self.closed.push(current);
                self.active = Some(TimeBlock::new(name.clone(), project.clone(), client.clone(), timestamp));
                Ok(())
            }
            TransitionPayload::Interrupt { name, project, client } => {
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(timestamp);
                // completion_reason stays None: pending, resolved on Return.
                let paused_id = current.id;
                // The frame must carry the PAUSED task's identity (what to
                // resume later), not the incoming interrupting task's identity.
                let frame = StackFrame {
                    paused_time_block_id: paused_id,
                    name: current.name.clone(),
                    project: current.project.clone(),
                    client: current.client.clone(),
                };
                self.closed.push(current);
                self.stack.push(frame);
                self.active = Some(TimeBlock::new(name.clone(), project.clone(), client.clone(), timestamp));
                Ok(())
            }
            TransitionPayload::ReturnPrevious => {
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                let frame = self.stack.pop().ok_or(StackError::StackEmpty)?;
                current.end = Some(timestamp);
                current.completion_reason = Some(CompletionReason::Explicit);
                self.closed.push(current);

                self.resolve_paused(frame.paused_time_block_id, CompletionReason::Explicit)?;
                self.active = Some(TimeBlock::new(frame.name, frame.project, frame.client, timestamp));
                Ok(())
            }
            TransitionPayload::ReturnOriginal => {
                if self.stack.is_empty() {
                    return Err(StackError::StackEmpty);
                }
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(timestamp);
                current.completion_reason = Some(CompletionReason::Explicit);
                self.closed.push(current);

                // Pop every frame down to (but not including) the root, marking
                // each skipped one auto-completed-on-skip.
                while self.stack.len() > 1 {
                    let skipped = self.stack.pop().expect("len > 1 checked above");
                    self.resolve_paused(skipped.paused_time_block_id, CompletionReason::AutoCompletedOnSkip)?;
                }
                // The root frame is directly engaged with — explicit, same as
                // Return to Previous's target.
                let root = self.stack.pop().expect("stack was non-empty");
                self.resolve_paused(root.paused_time_block_id, CompletionReason::Explicit)?;
                self.active = Some(TimeBlock::new(root.name, root.project, root.client, timestamp));
                Ok(())
            }
            TransitionPayload::Complete => {
                if !self.stack.is_empty() {
                    return Err(StackError::CannotCompleteWithOpenStack);
                }
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(timestamp);
                current.completion_reason = Some(CompletionReason::Explicit);
                self.closed.push(current);
                Ok(())
            }
            TransitionPayload::Heartbeat => {
                // No stack-state effect this slice — heartbeats only bound
                // recovered-gap inference accuracy (Slice 2), not implemented yet.
                Ok(())
            }
        }
    }

    fn resolve_paused(&mut self, id: Uuid, reason: CompletionReason) -> Result<(), StackError> {
        let block = self
            .closed
            .iter_mut()
            .rev()
            .find(|b| b.id == id)
            .ok_or(StackError::PausedBlockNotFound(id))?;
        block.completion_reason = Some(reason);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::sync::LazyLock;

    // A single fixed base instant, captured once, so repeated calls to `t()`
    // with the same offset always produce byte-identical timestamps — using
    // `Utc::now()` per call was a test bug (microsecond drift between calls
    // broke equality assertions on otherwise-correct behavior).
    static BASE: LazyLock<DateTime<Utc>> = LazyLock::new(Utc::now);

    fn t(offset_secs: i64) -> DateTime<Utc> {
        *BASE + Duration::seconds(offset_secs)
    }

    fn start(stack: &mut InterruptionStack, name: &str, at: i64) {
        stack
            .apply(
                &TransitionPayload::Start { name: name.into(), project: None, client: None },
                t(at),
            )
            .unwrap();
    }

    fn interrupt(stack: &mut InterruptionStack, name: &str, at: i64) {
        stack
            .apply(
                &TransitionPayload::Interrupt { name: name.into(), project: None, client: None },
                t(at),
            )
            .unwrap();
    }

    #[test]
    fn switch_closes_current_as_explicit_and_does_not_push_stack() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        s.apply(
            &TransitionPayload::Switch { name: "B".into(), project: None, client: None },
            t(10),
        )
        .unwrap();

        assert_eq!(s.stack_depth(), 0);
        assert_eq!(s.closed.len(), 1);
        assert_eq!(s.closed[0].name, "A");
        assert_eq!(s.closed[0].completion_reason, Some(CompletionReason::Explicit));
        assert_eq!(s.closed[0].end, Some(t(10)));
        assert_eq!(s.active.as_ref().unwrap().name, "B");
    }

    #[test]
    fn interrupt_closes_current_with_pending_reason_and_pushes_stack() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);

        assert_eq!(s.stack_depth(), 1);
        assert_eq!(s.closed.len(), 1);
        assert_eq!(s.closed[0].name, "A");
        assert_eq!(s.closed[0].end, Some(t(10)), "A's duration must not include time spent on B");
        assert_eq!(s.closed[0].completion_reason, None, "fate pending until returned to or skipped");
        assert_eq!(s.active.as_ref().unwrap().name, "B");
    }

    #[test]
    fn return_previous_resolves_popped_frame_as_explicit_and_starts_new_block() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        let a_id = s.closed[0].id;

        s.apply(&TransitionPayload::ReturnPrevious, t(20)).unwrap();

        assert_eq!(s.stack_depth(), 0);
        // B closed explicitly, A resolved explicitly, new "A" block created (not reopened).
        assert_eq!(s.closed.len(), 2);
        assert_eq!(s.closed[1].name, "B");
        assert_eq!(s.closed[1].completion_reason, Some(CompletionReason::Explicit));
        let resolved_a = s.closed.iter().find(|b| b.id == a_id).unwrap();
        assert_eq!(resolved_a.completion_reason, Some(CompletionReason::Explicit));

        let new_active = s.active.as_ref().unwrap();
        assert_eq!(new_active.name, "A");
        assert_ne!(new_active.id, a_id, "resuming must create a brand-new Time Block, never reopen the original");
        assert_eq!(new_active.start, t(20));
    }

    #[test]
    fn return_original_skips_intermediate_frames_as_auto_completed() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        interrupt(&mut s, "C", 20);
        let a_id = s.closed[0].id;
        let b_id = s.closed[1].id;

        s.apply(&TransitionPayload::ReturnOriginal, t(30)).unwrap();

        assert_eq!(s.stack_depth(), 0);
        let resolved_a = s.closed.iter().find(|b| b.id == a_id).unwrap();
        let resolved_b = s.closed.iter().find(|b| b.id == b_id).unwrap();
        assert_eq!(resolved_a.completion_reason, Some(CompletionReason::Explicit), "root is directly engaged with, never auto-completed");
        assert_eq!(resolved_b.completion_reason, Some(CompletionReason::AutoCompletedOnSkip), "B was skipped, never explicit");

        let new_active = s.active.as_ref().unwrap();
        assert_eq!(new_active.name, "A");
        assert_ne!(new_active.id, a_id);
    }

    #[test]
    fn complete_requires_empty_stack() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        let err = s.apply(&TransitionPayload::Complete, t(20)).unwrap_err();
        assert_eq!(err, StackError::CannotCompleteWithOpenStack);
    }

    #[test]
    fn depth_12_interrupts_then_12_return_previous_unwinds_correctly() {
        let mut s = InterruptionStack::new();
        start(&mut s, "task-0", 0);
        for i in 1..=12 {
            interrupt(&mut s, &format!("task-{i}"), i as i64 * 10);
        }
        assert_eq!(s.stack_depth(), 12);
        assert_eq!(s.active.as_ref().unwrap().name, "task-12");

        for i in (0..12).rev() {
            s.apply(&TransitionPayload::ReturnPrevious, t(200 + (12 - i) * 10)).unwrap();
            assert_eq!(s.active.as_ref().unwrap().name, format!("task-{i}"));
        }
        assert_eq!(s.stack_depth(), 0);
        // Every paused block resolved explicit — none were skipped via Return to Original.
        assert!(s.closed.iter().all(|b| b.completion_reason == Some(CompletionReason::Explicit)));
    }

    #[test]
    fn depth_12_interrupts_then_return_original_skips_11() {
        let mut s = InterruptionStack::new();
        start(&mut s, "root", 0);
        for i in 1..=12 {
            interrupt(&mut s, &format!("task-{i}"), i as i64 * 10);
        }
        assert_eq!(s.stack_depth(), 12);

        s.apply(&TransitionPayload::ReturnOriginal, t(500)).unwrap();

        assert_eq!(s.stack_depth(), 0);
        assert_eq!(s.active.as_ref().unwrap().name, "root");
        let skipped_count = s
            .closed
            .iter()
            .filter(|b| b.completion_reason == Some(CompletionReason::AutoCompletedOnSkip))
            .count();
        assert_eq!(skipped_count, 11, "task-1..task-11 skipped; task-12 (the closing active) and root are explicit");
    }

    #[test]
    fn start_rejects_when_already_active() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        let err = s
            .apply(&TransitionPayload::Start { name: "B".into(), project: None, client: None }, t(10))
            .unwrap_err();
        assert_eq!(err, StackError::AlreadyActive);
    }

    #[test]
    fn return_previous_rejects_when_stack_empty() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        let err = s.apply(&TransitionPayload::ReturnPrevious, t(10)).unwrap_err();
        assert_eq!(err, StackError::StackEmpty);
    }
}
