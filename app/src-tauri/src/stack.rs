//! The interruption stack state machine. Pure in-memory logic, no I/O — this is
//! deliberately the ONE place "what does an Interrupt/Return mean" lives, so that
//! live commands and log replay can never diverge (see `log::reader`).
//!
//! Confirmed semantics (see conversation, 2026-07-24):
//! - Interrupting a task closes its Time Block IMMEDIATELY (end = now), giving it
//!   an accurate duration that never includes time spent on whatever interrupted
//!   it. Its END is user-determined at that moment; its INTERRUPTION OUTCOME
//!   stays pending until it is returned to, skipped, or dismissed.
//! - Resuming a paused task (via either Return operation) never reopens the
//!   original Time Block — it starts a brand-new one. Each pause/resume cycle is
//!   an independent, flat entry (per the accepted Concept: "each block counts as
//!   an independent entry, aggregated at export time").
//! - Return to Previous resolves exactly the frame it pops as `Resumed`.
//! - Return to Original resolves every skipped frame as `Skipped`, and the root
//!   frame it lands on as `Resumed` (it was directly engaged with, same as
//!   Return to Previous's target).
//!
//! Legal-state contract (ADR 0005 amendment, 2026-07-29):
//! - `active == None` with a NON-EMPTY stack is a supported state, not an error.
//!   It is already reachable today: `state::AppState::init` appends `RecoverGap`
//!   when replay leaves an entry active, and `RecoverGap` deliberately does not
//!   auto-resume — so a crash inside an interruption produces it on next launch,
//!   frames intact. It will also arise from Pause once that is built.
//! - Both Return operations MUST therefore work with nothing active. Previously
//!   they demanded an active task, so the only escape from that state was
//!   `commands::switch` silently acting as `Start` — forcing the user to begin a
//!   task they may not have been doing in order to unwind orphaned frames. That
//!   is `docs/principles.md` #3's failure mode ("the state model must never force
//!   users to create inaccurate records simply to satisfy the software").

use crate::model::{
    DerivedInterruptionStatus, EndDetermination, InterruptionOutcome, StackFrame, TimeBlock, TransitionPayload,
};
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
    /// An `interruption_outcome` of `None` is ambiguous on its own (never
    /// interrupted vs. interrupted-and-unresolved) — use `derived_status`.
    pub closed: Vec<TimeBlock>,
}

impl InterruptionStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// The default name assigned when a task is started without one (the
    /// Switch/Interrupt hotkeys no longer require typing a name first —
    /// see `docs/product/features/interruption-stack.md` revision). Numbered
    /// "Anchor N", counting only Time Blocks already using this convention
    /// that started at or after `today_start` — so the count resets every day
    /// without a separate persisted counter, purely from the existing
    /// timeline. `today_start` (a UTC instant marking local midnight) is
    /// computed by the caller, keeping this function pure and independent of
    /// the host's timezone/wall clock for testing.
    pub fn next_default_name(&self, today_start: DateTime<Utc>) -> String {
        let max_n = self
            .closed
            .iter()
            .chain(self.active.iter())
            .filter(|b| b.start >= today_start)
            .filter_map(|b| b.name.strip_prefix("Anchor ").and_then(|rest| rest.parse::<u32>().ok()))
            .max()
            .unwrap_or(0);
        format!("Anchor {}", max_n + 1)
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
                current.end_determination = Some(EndDetermination::UserDetermined);
                self.closed.push(current);
                self.active = Some(TimeBlock::new(name.clone(), project.clone(), client.clone(), timestamp));
                Ok(())
            }
            TransitionPayload::Interrupt { name, project, client } => {
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(timestamp);
                // The interrupt fixed this block's end moment, so its END is
                // user-determined like any other. What stays pending is its
                // INTERRUPTION OUTCOME — resolved on Return, or on dismissal.
                // Splitting those apart is the whole point of ADR 0005: the old
                // single field left this block indistinguishable from one that
                // was never interrupted.
                current.end_determination = Some(EndDetermination::UserDetermined);
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
            TransitionPayload::Rename { name, project, client } => {
                let current = self.active.as_mut().ok_or(StackError::NoActiveTask)?;
                current.name = name.clone();
                current.project = project.clone();
                current.client = client.clone();
                Ok(())
            }
            TransitionPayload::ReturnPrevious => {
                // Pop BEFORE touching `active`: a StackEmpty error must not leave
                // a half-applied mutation behind. (Live commands dry-run on a
                // clone, so this was never live corruption — but `log::reader`
                // calls `apply` directly, with no such guard.)
                let frame = self.stack.pop().ok_or(StackError::StackEmpty)?;
                self.close_active_if_any(timestamp);

                self.resolve_paused(frame.paused_time_block_id, InterruptionOutcome::Resumed)?;
                self.active = Some(TimeBlock::new(frame.name, frame.project, frame.client, timestamp));
                Ok(())
            }
            TransitionPayload::ReturnOriginal => {
                if self.stack.is_empty() {
                    return Err(StackError::StackEmpty);
                }
                self.close_active_if_any(timestamp);

                // Pop every frame down to (but not including) the root, marking
                // each skipped one auto-completed-on-skip.
                while self.stack.len() > 1 {
                    let skipped = self.stack.pop().expect("len > 1 checked above");
                    self.resolve_paused(skipped.paused_time_block_id, InterruptionOutcome::Skipped)?;
                }
                // The root frame is directly engaged with — explicit, same as
                // Return to Previous's target.
                let root = self.stack.pop().expect("stack was non-empty");
                self.resolve_paused(root.paused_time_block_id, InterruptionOutcome::Resumed)?;
                self.active = Some(TimeBlock::new(root.name, root.project, root.client, timestamp));
                Ok(())
            }
            TransitionPayload::Complete => {
                if !self.stack.is_empty() {
                    return Err(StackError::CannotCompleteWithOpenStack);
                }
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(timestamp);
                current.end_determination = Some(EndDetermination::UserDetermined);
                self.closed.push(current);
                Ok(())
            }
            TransitionPayload::Heartbeat => {
                // No stack-state effect — heartbeats exist purely to bound
                // recovered-gap inference accuracy (see `heartbeat.rs`), not to
                // change what's active.
                Ok(())
            }
            TransitionPayload::RecoverGap { inferred_end } => {
                // Deliberately does NOT start a new active entry — whether to
                // auto-resume is a caller decision (startup: no; live sleep/wake
                // resume: yes, via a separate Start transition), not baked in here.
                let mut current = self.active.take().ok_or(StackError::NoActiveTask)?;
                current.end = Some(*inferred_end);
                current.end_determination = Some(EndDetermination::SystemInferred);
                self.closed.push(current);
                Ok(())
            }
        }
    }

    /// Close the active Time Block if there is one, otherwise do nothing.
    ///
    /// The "otherwise do nothing" is the point: `active == None` with a
    /// non-empty stack is a legal state (see module docs), so a Return must
    /// resolve its frame from that state rather than demanding a synthetic task
    /// start first. Infallible by construction — there is no failure mode in
    /// "close it if it exists" — which is what lets the Return arms stay
    /// straight-line.
    fn close_active_if_any(&mut self, timestamp: DateTime<Utc>) {
        if let Some(mut current) = self.active.take() {
            current.end = Some(timestamp);
            current.end_determination = Some(EndDetermination::UserDetermined);
            self.closed.push(current);
        }
    }

    /// The canonical interruption status for a block — the ONE interpretation
    /// every consumer must use (ADR 0005). Reading `interruption_outcome`
    /// directly is a bug: absent means *never interrupted* OR *interrupted and
    /// unresolved*, and only the live stack tells those apart.
    pub fn derived_status(&self, block: &TimeBlock) -> DerivedInterruptionStatus {
        match block.interruption_outcome {
            Some(InterruptionOutcome::Resumed) => DerivedInterruptionStatus::Resumed,
            Some(InterruptionOutcome::Skipped) => DerivedInterruptionStatus::Skipped,
            None if self.stack.iter().any(|f| f.paused_time_block_id == block.id) => {
                DerivedInterruptionStatus::Pending
            }
            None => DerivedInterruptionStatus::NeverInterrupted,
        }
    }

    fn resolve_paused(&mut self, id: Uuid, outcome: InterruptionOutcome) -> Result<(), StackError> {
        let block = self
            .closed
            .iter_mut()
            .rev()
            .find(|b| b.id == id)
            .ok_or(StackError::PausedBlockNotFound(id))?;
        block.interruption_outcome = Some(outcome);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Only the tests exercise these two — the state machine itself never
    // constructs anything but a live capture.
    use crate::model::{CaptureOrigin, CaptureSource};
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

    /// The whole point of `derived_status`: an absent `interruption_outcome`
    /// means two different things, and only the live stack distinguishes them.
    /// A block read in isolation cannot tell "never interrupted" from
    /// "interrupted, still pending" — which is exactly the confusion the old
    /// single `completion_reason` field allowed, and which R1's audit trail
    /// depends on preventing.
    #[test]
    fn derived_status_distinguishes_pending_from_never_interrupted() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        s.apply(&TransitionPayload::Switch { name: "B".into(), project: None, client: None }, t(10))
            .unwrap();
        interrupt(&mut s, "C", 20);

        // A: switched away from, never interrupted. B: interrupted by C, still
        // pending. Both carry `interruption_outcome: None`.
        let a = s.closed.iter().find(|b| b.name == "A").unwrap();
        let b = s.closed.iter().find(|b| b.name == "B").unwrap();
        assert_eq!(a.interruption_outcome, None);
        assert_eq!(b.interruption_outcome, None, "identical in the persisted field");

        assert_eq!(s.derived_status(a), DerivedInterruptionStatus::NeverInterrupted);
        assert_eq!(s.derived_status(b), DerivedInterruptionStatus::Pending, "its frame is still on the stack");
    }

    #[test]
    fn derived_status_reports_resolved_outcomes() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        interrupt(&mut s, "C", 20);
        s.apply(&TransitionPayload::ReturnOriginal, t(30)).unwrap();

        let a = s.closed.iter().find(|b| b.name == "A").unwrap();
        let b = s.closed.iter().find(|b| b.name == "B").unwrap();
        assert_eq!(s.derived_status(a), DerivedInterruptionStatus::Resumed);
        assert_eq!(s.derived_status(b), DerivedInterruptionStatus::Skipped);
    }

    /// Every block the state machine creates is live-captured by definition —
    /// it is created as the work happens. Manual entry (#15) is the exception
    /// path and does not exist yet, so Capture Rate is trivially 100% today.
    /// That is the correct answer, not a placeholder.
    #[test]
    fn state_machine_only_ever_produces_live_captured_blocks() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        s.apply(&TransitionPayload::RecoverGap { inferred_end: t(15) }, t(20)).unwrap();

        assert!(s.closed.iter().all(|b| b.capture_origin == CaptureOrigin::LiveCapture));
        assert!(s.closed.iter().all(|b| !b.capture_origin.is_adjusted()));
    }

    #[test]
    fn capture_origin_adjustment_preserves_source_and_is_idempotent() {
        assert_eq!(CaptureOrigin::LiveCapture.adjusted(), CaptureOrigin::LiveCaptureAdjusted);
        assert_eq!(CaptureOrigin::ManualEntry.adjusted(), CaptureOrigin::ManualEntryAdjusted);
        // Adjusting twice must not change anything further.
        assert_eq!(CaptureOrigin::LiveCaptureAdjusted.adjusted(), CaptureOrigin::LiveCaptureAdjusted);
        // Origin survives adjustment — a manually entered block nudged by one
        // second must never become indistinguishable from a live capture.
        assert_eq!(CaptureOrigin::ManualEntryAdjusted.origin(), CaptureSource::Manual);
        assert_eq!(CaptureOrigin::LiveCaptureAdjusted.origin(), CaptureSource::Live);
    }

    /// Reproduces the exact state a crash inside an interruption leaves behind:
    /// `state::AppState::init` appends `RecoverGap` when replay finds something
    /// active, and `RecoverGap` deliberately does not auto-resume.
    fn crashed_mid_interruption() -> InterruptionStack {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        s.apply(&TransitionPayload::RecoverGap { inferred_end: t(15) }, t(20)).unwrap();

        assert!(s.active.is_none(), "gap recovery must not auto-resume");
        assert_eq!(s.stack_depth(), 1, "A's frame survives the crash");
        s
    }

    /// Regression: before ADR 0005's amendment, EVERY transition from this state
    /// was illegal — Return*/Interrupt/Rename on `NoActiveTask`, Complete on
    /// `CannotCompleteWithOpenStack` — so the only escape was `commands::switch`
    /// silently acting as `Start`, forcing the user to begin a task they may not
    /// have been doing just to unwind orphaned frames (`principles.md` #3).
    #[test]
    fn return_previous_unwinds_after_crash_recovery_without_starting_a_task() {
        let mut s = crashed_mid_interruption();
        let a_id = s.closed[0].id;

        s.apply(&TransitionPayload::ReturnPrevious, t(30)).unwrap();

        assert_eq!(s.stack_depth(), 0, "the orphaned frame is resolved");
        assert_eq!(s.active.as_ref().unwrap().name, "A", "resumed without a synthetic Start");
        assert_eq!(
            s.closed.iter().find(|b| b.id == a_id).unwrap().interruption_outcome,
            Some(InterruptionOutcome::Resumed),
            "the returned-to frame resolves explicitly, exactly as with an active task"
        );
        assert_eq!(
            s.closed.len(),
            2,
            "nothing extra is closed — there was no active block to close"
        );
    }

    #[test]
    fn return_original_unwinds_after_crash_recovery_without_starting_a_task() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);
        interrupt(&mut s, "B", 10);
        interrupt(&mut s, "C", 20);
        s.apply(&TransitionPayload::RecoverGap { inferred_end: t(25) }, t(30)).unwrap();
        assert!(s.active.is_none());
        assert_eq!(s.stack_depth(), 2);

        let a_id = s.closed[0].id;
        let b_id = s.closed[1].id;

        s.apply(&TransitionPayload::ReturnOriginal, t(40)).unwrap();

        assert_eq!(s.stack_depth(), 0);
        assert_eq!(s.active.as_ref().unwrap().name, "A");
        assert_eq!(
            s.closed.iter().find(|b| b.id == b_id).unwrap().interruption_outcome,
            Some(InterruptionOutcome::Skipped),
            "skipped frames stay distinguishable even when nothing was active"
        );
        assert_eq!(
            s.closed.iter().find(|b| b.id == a_id).unwrap().interruption_outcome,
            Some(InterruptionOutcome::Resumed)
        );
    }

    /// `Complete` is deliberately still rejected here — that hole is Pause's to
    /// close, not this fix's. Pinned so the two changes stay distinguishable.
    #[test]
    fn complete_is_still_rejected_after_crash_recovery_with_an_open_stack() {
        let mut s = crashed_mid_interruption();
        assert_eq!(
            s.apply(&TransitionPayload::Complete, t(30)),
            Err(StackError::CannotCompleteWithOpenStack)
        );
    }

    /// A failed Return must not consume the active block. Live commands dry-run
    /// on a clone so this was never observable there, but `log::reader` calls
    /// `apply` directly with no such guard.
    #[test]
    fn return_previous_on_empty_stack_leaves_the_active_block_intact() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);

        assert_eq!(s.apply(&TransitionPayload::ReturnPrevious, t(10)), Err(StackError::StackEmpty));
        assert_eq!(s.active.as_ref().unwrap().name, "A", "the active block survives the rejection");
        assert!(s.closed.is_empty());
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
        assert_eq!(s.closed[0].end_determination, Some(EndDetermination::UserDetermined));
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
        assert_eq!(s.closed[0].interruption_outcome, None, "outcome pending until returned to or skipped");
        assert_eq!(s.closed[0].end_determination, Some(EndDetermination::UserDetermined), "the interrupt still fixed its end");
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
        assert_eq!(s.closed[1].end_determination, Some(EndDetermination::UserDetermined));
        let resolved_a = s.closed.iter().find(|b| b.id == a_id).unwrap();
        assert_eq!(resolved_a.interruption_outcome, Some(InterruptionOutcome::Resumed));

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
        assert_eq!(resolved_a.interruption_outcome, Some(InterruptionOutcome::Resumed), "root is directly engaged with, never skipped");
        assert_eq!(resolved_b.interruption_outcome, Some(InterruptionOutcome::Skipped), "B was skipped, never resumed");

        let new_active = s.active.as_ref().unwrap();
        assert_eq!(new_active.name, "A");
        assert_ne!(new_active.id, a_id);
    }

    #[test]
    fn recover_gap_closes_active_with_inferred_end_and_does_not_auto_resume() {
        let mut s = InterruptionStack::new();
        start(&mut s, "A", 0);

        let inferred_end = t(5); // e.g. the last heartbeat before a crash/sleep
        s.apply(&TransitionPayload::RecoverGap { inferred_end }, t(100)).unwrap();

        assert_eq!(s.closed.len(), 1);
        assert_eq!(s.closed[0].name, "A");
        assert_eq!(s.closed[0].end, Some(inferred_end), "end must be the inferred time, not the moment the gap was detected");
        assert_eq!(s.closed[0].end_determination, Some(EndDetermination::SystemInferred));
        assert!(s.active.is_none(), "RecoverGap must not auto-resume — that decision belongs to the caller");
    }

    #[test]
    fn recover_gap_requires_an_active_task() {
        let mut s = InterruptionStack::new();
        let err = s
            .apply(&TransitionPayload::RecoverGap { inferred_end: t(0) }, t(10))
            .unwrap_err();
        assert_eq!(err, StackError::NoActiveTask);
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
        assert!(s.closed.iter().all(|b| b.end_determination == Some(EndDetermination::UserDetermined)));
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
            .filter(|b| b.interruption_outcome == Some(InterruptionOutcome::Skipped))
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

    #[test]
    fn rename_changes_active_task_fields_without_touching_start_stack_or_closed() {
        let mut s = InterruptionStack::new();
        start(&mut s, "Anchor 1", 0);
        s.apply(
            &TransitionPayload::Rename { name: "Real name".into(), project: Some("Acme".into()), client: None },
            t(5),
        )
        .unwrap();

        let active = s.active.as_ref().unwrap();
        assert_eq!(active.name, "Real name");
        assert_eq!(active.project, Some("Acme".to_string()));
        assert_eq!(active.start, t(0), "rename must not change the start time");
        assert!(s.closed.is_empty());
        assert_eq!(s.stack_depth(), 0);
    }

    #[test]
    fn rename_requires_an_active_task() {
        let mut s = InterruptionStack::new();
        let err = s
            .apply(&TransitionPayload::Rename { name: "X".into(), project: None, client: None }, t(0))
            .unwrap_err();
        assert_eq!(err, StackError::NoActiveTask);
    }

    #[test]
    fn next_default_name_starts_at_1_when_nothing_matches_today() {
        let s = InterruptionStack::new();
        assert_eq!(s.next_default_name(t(0)), "Anchor 1");
    }

    #[test]
    fn next_default_name_increments_past_existing_anchor_names_started_today() {
        let mut s = InterruptionStack::new();
        start(&mut s, "Anchor 1", 0);
        s.apply(&TransitionPayload::Switch { name: "Anchor 2".into(), project: None, client: None }, t(10))
            .unwrap();
        assert_eq!(s.next_default_name(t(-100)), "Anchor 3");
    }

    #[test]
    fn next_default_name_ignores_anchor_names_that_started_before_today() {
        let mut s = InterruptionStack::new();
        start(&mut s, "Anchor 5", 0);
        // today_start is after this entry's start — yesterday's count must not carry over.
        assert_eq!(s.next_default_name(t(50)), "Anchor 1");
    }

    #[test]
    fn next_default_name_ignores_non_anchor_names() {
        let mut s = InterruptionStack::new();
        start(&mut s, "Some real task", 0);
        assert_eq!(s.next_default_name(t(-10)), "Anchor 1");
    }
}
