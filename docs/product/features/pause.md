---
status: accepted
date: 2026-08-07
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/principles.md, docs/risks.md, docs/assumptions.md, docs/glossary.md, docs/product/features/interruption-stack.md, docs/product/features/interruption-history.md, docs/product/features/timeline-reconstruction.md, docs/product/features/timeline-editor.md, docs/product/features/visual-redesign.md, docs/product/features/export.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0006-stable-persistent-time-block-identity.md, docs/decisions/0007-auto-resume-after-a-short-gap.md]
---

# Pause

> Created 2026-08-07. Follows `.claude/workflows/design.md`. **The last MVP-scope item without a feature doc** (`planning/milestones.md`).
>
> **This document decides far less than it first appears to**, because [ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md) already decided the event model and is `accepted`. What it decides is stated in "What ADR 0005 already settled" below and is **not reopened**. This doc covers what remains: the command surface, the UI, and eleven state interactions nobody had checked.
>
> **Why the verification bar is higher here than usual.** Pause is risk **R11**'s most expensive instance: it was once decided as *"Complete + Start"*, a model the state machine rejects outright — `apply`'s `Complete` arm returns `CannotCompleteWithOpenStack` whenever the stack is non-empty, so the decision was impossible on the day it was made and nobody checked. Every implementation claim below was therefore verified against the code, and **three accepted claims turned out to be false** — all three in the *optimistic* direction, meaning Pause is smaller than the documents say.

## What ADR 0005 already settled

Quoted rather than paraphrased, because this document must not drift from it:

- **"Pause is a specialised Interrupt: it creates an interruption frame without creating a successor task. It closes the active Time Block, pushes its frame onto the interruption stack, and leaves `active == None`. Nothing is started."**
- **"The paused task keeps its return path."** It is the top frame, and Return to Previous lands on it.
- **"Continue Session is not a transition. It is a UI action only."** After Pause the state is already correct, and replay reconstructs it from the persisted stack.
- **"No second mechanism for preserving return intent."** Pause reuses the interruption machinery entirely — no `PauseSpan`, no persisted `Paused` state.
- **`active == None` with a non-empty interruption stack is an accepted, supported application state**, and the state machine must offer legal transitions out of it without a synthetic task start.

This feature adds **one transition** and nothing else to the record.

## Problem

`Complete` is rejected while the interruption stack is non-empty (`apply`'s `Complete` arm, `CannotCompleteWithOpenStack`). So a user who is three interruptions deep and wants to stop working — lunch, end of day, a meeting they will not track — has no legal way to stop. The options today are:

1. **Unwind the stack first** with Return to Original, which marks every intermediate block `Skipped` — a false record, since the work was not skipped, it was postponed.
2. **Leave a task running** across the break, which bills the break.
3. **Kill the app**, which converts a deliberate stop into gap recovery: the block's end becomes `SystemInferred` and, past an hour, ADR 0007 closes it without resuming — a wrong end time deliberately created, which is risk **R4** self-inflicted.

All three corrupt the record to express something the product has no word for. ADR 0005 states the cost precisely: without Pause the user must *"start a task they did not do purely to make an unwinding transition legal"* — [`principles.md`](../principles.md) #3's failure mode.

**This is a hole-closing change, not a convenience feature** (`mvp.md:24`).

### Three accepted claims that the code contradicts

Verified before designing, because `mvp.md:24` scopes this feature and two of its three clauses are wrong.

| Accepted claim | What the code says |
|---|---|
| `mvp.md:24`: Pause "requires `ReturnPrevious`/`ReturnOriginal` to become legal with no active task" | **Already legal.** Neither arm requires an active task: `ReturnPrevious` pops the frame then calls `close_active_if_any`, which is a no-op when `active` is `None`; `ReturnOriginal` checks only that the stack is non-empty. Both then set `active` unconditionally. No change is needed. |
| `mvp.md:24`: "`Start` to be exposed as its own command (the transition exists; no command does)" | **A `start` command exists** (`commands.rs:179`), is registered (`lib.rs:142`), is bound to the tray/empty-name path, and is called by the frontend's `startTask`. This was true when written and was fixed by the ADR 0005 command split; the doc was never updated. |
| ADR 0005:110: `RecoverGap` "deliberately does not auto-resume" | **Superseded by [ADR 0007](../../decisions/0007-auto-resume-after-a-short-gap.md)** (2026-08-03): a gap of 90 s–1 h now closes *and* auto-resumes. ADR 0005's conclusion still holds — the `active == None` + non-empty-stack state still arises, now via the ≥ 1 h zone — but its stated mechanism is out of date. Not amended in place: ADRs are append-only and ADR 0007 is the superseding record. |

**So Pause's implementation surface is one transition, one command, and a UI** — not the three-part change `mvp.md` describes. `mvp.md:24` is amended accordingly.

## Goals

- **Stopping work is expressible without falsifying the record.** Measurable: from any stack depth, the user can reach a state where nothing is active, no block is running, and **no block has been marked `Skipped`** by the act of stopping.
- **The paused task is resumable by the path that already exists.** Return to Previous lands on it; no new return path (`mvp.md:25`).
- **A paused Anchor is unmistakable.** The user can tell at a glance that nothing is being tracked *on purpose*, distinctly from Anchor having lost track — and distinctly from the app simply being idle.
- **Pause survives a restart, a crash, a sleep and a wake with no special handling**, because the paused state is ordinary persisted state.
- **The fast path is untouched** for existing actions. No markup, handler, or command on the current hotkey or widget path changes behaviour.

Explicit non-goals: no persisted pause entity, no pause *duration* record, no third return path, no change to export output, and **no auto-pause on idle** — that is activity inference, barred by [ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md).

## Users

The single segment in `docs/product/users.md`: **the interrupted billable developer**, currently the author alone.

- *"Values speed and low friction above all."* Pause is a **capture-path** action, not a review action: it happens at the moment the user stands up. Its timestamp is the moment it is invoked, so anything that delays invocation makes the record wrong. This is what drives decision 6.
- *"Comfortable with hotkeys,"* explicitly not needing onboarding.

## Decisions taken (2026-08-07)

**1. `Pause` requires an active Time Block, and that single precondition answers "what happens if I pause twice".**

ADR 0005:85 gives Pause the precondition *"active block"*. Therefore a second Pause while already paused is **rejected** with `NoActiveTask`, by the same rule that rejects any other operation needing an active task. **No repeat-pause rule is written, because none is needed** — this is the state machine's existing shape doing the work, which is what ADR 0005 meant by reusing the interruption machinery entirely.

**2. The paused block records exactly what an interrupted block records.** Its end is the moment Pause was invoked, `EndDetermination::UserDetermined` — the user chose it. `interruption_outcome` stays `None`, so `derived_status` reports **`Pending`** while the frame is open, then `Resumed` or `Skipped` when the frame is resolved. `CaptureOrigin` is untouched (`LiveCapture`). **Nothing distinguishes a paused block from an interrupted one in the record, and that is a decision**, not an omission: ADR 0005 made Pause *a specialised Interrupt*, and the domain fact is identical — *this work stopped and the user intends to return*. It is the same reasoning that rejected an `Abandoned` value for dismissal.

**3. "Paused" is a derived UI state, not a stored one, and it is distinguishable from lost-track by evidence that already exists.**

The state `active == None && !stack.is_empty()` has two causes, and ADR 0005:110–112 records that the second one **already occurs in shipped code**: `RecoverGap` closes a block without pushing a frame (ADR 0005:168), so a crash inside an interruption leaves frames waiting that the user never chose to leave.

**Anchor asserts "Paused" only when the evidence is unambiguous, and otherwise states the facts without claiming intent.** The rule:

> **Paused** ⟺ nothing is active, the stack is non-empty, the top frame's paused block has `end_determination == UserDetermined`, and **no closed block ends later than it**.

- The last clause is what excludes the lost-track case: there, the `RecoverGap`-closed block is `SystemInferred` and ends *after* the top frame's block, because the crash happened later than the interruption.
- **The comparison is by `end` time, never by position in `closed`.** *(Corrected on this document's review, which had used "the most recently closed block".)* `closed` is push-ordered, and `Add` pushes a reconstructed block onto the end (`stack.rs:256`) — so a manual entry made while paused would have moved the "most recent" block and flipped the reading. Position is an artefact of when a record was written; `end` is the fact being asked about.

**When the rule does not hold, Anchor does not guess.** It reports what it can see — nothing is being tracked, *n* tasks are waiting — and, where the latest end is `SystemInferred`, that it closed that block itself and the end may be wrong, with the correction route. It does **not** call that "Paused", because the user never said so, and inventing intent is [`principles.md`](../principles.md) #3.

This degrades in the safe direction: an unusual sequence costs a *less specific* message, never a false one. Both inputs are already serialised in `StackView`, so **no new field and no new transition is needed to answer "why is nothing running?"**

**4. Paused is shown as a first-class state on the primary surfaces, not as an absence.** "No active task" is what Anchor says today when nothing has ever started; it must not also be what it says after a deliberate stop. The dashboard and the widget both show a **Paused** state naming the task that is waiting and how long it has been waiting — the latter derived as `now − pausedBlock.end`, the same computation `interruption-history.md` uses for a frame, and never stored.

**5. Continue is Return to Previous, relabelled in the paused state — not a new action.** ADR 0005 fixed that Continue Session is a UI action only; this decides which existing command it maps to, which that ADR left open. While paused, the control that resumes the top frame reads **Continue** rather than *Return to Previous*, because after a lunch break "return to previous" describes a stack operation and "continue" describes what the user is doing.

The relabel is **contextual and total**: it applies exactly when the paused shape in decision 3 holds, and the underlying command is unchanged, so this creates no third return path (`mvp.md:25`). *Return to Original* keeps its name in both states — it is not "continuing" anything when the top frame is not the target.

**6. Pause gets a global hotkey — a sixth binding.** Pause is a capture-path action: its whole value is that the recorded end is the moment the user stopped. Requiring the dashboard would put a window-focus and a mouse trip between the decision and the timestamp, making every paused block's end late by exactly the amount of friction — the argument `interruption-stack.md` already uses for Switch and Interrupt. Under `visual-redesign.md` E.3 this is a **frequent** action and must therefore be *memorable*, which is what a binding provides.

This changes `HotkeyBindings` from five fields to six, and that struct is applied atomically (`hotkeys::apply_remap`) and persisted — see Technical Constraints.

**7. Pause is legal while interrupted, and the consequence is surfaced rather than prevented.** Pausing at depth *n* yields depth *n + 1*, with the paused task on top. This creates one genuine hazard: **Return to Original will mark the just-paused task `Skipped`**, because it resolves every non-root frame that way. That is correct domain behaviour and is not changed here. It is made visible by `interruption-history.md`'s **return preview**, which already states which blocks *Return to Original* writes off — the paused one now among them. Prohibiting Pause at depth > 0 was considered and rejected (Alternative D): the deepest interruption is exactly when stopping for the day is most likely.

**8. Starting a task while paused is legal and needs no special case.** `Start` requires only that nothing is active, which holds. The paused frame stays where it is; the new task is not its successor and does not resume it. This is the same shape as an interruption whose successor arrives late, and the stack expresses it correctly without help.

**9. Nothing about Pause touches gap recovery, the heartbeat, or replay — verified, not assumed.**

- `gap::resolve` returns `Continue` when `active` is `None`, with a test named `nothing_active_means_nothing_to_do`. So **no gap transition is ever produced while paused**, on either the startup or the wake path.
- This also means `RecoverGap`'s `NoActiveTask` error is unreachable from the gap path while paused — the guard is in `resolve`, before any transition is built.
- `heartbeat::should_beat` returns the `active.is_some()` flag, so **heartbeats stop on their own while paused**. Nothing is logged during a break, and `last_activity_at` stops advancing — which is correct, because there is nothing whose end would need bounding.
- Replay reconstructs the paused state from the persisted stack like any other state, and the snapshot carries `InterruptionStack` whole. **This is why Continue Session needs no transition** (ADR 0005:70) — verified rather than trusted.

**10. `Complete` stays rejected while paused, and that is the right answer.** The stack is non-empty, so `CannotCompleteWithOpenStack` applies. Completing *what*? The paused task is not finished — if it were, the user would have completed it instead of pausing. To finish a paused day the user **continues, then completes**. Dismissing the remaining frames from the Interruption History panel empties the stack but does **not** on its own make `Complete` available: the arm checks the stack first and then requires an active task, and while paused there is none. *(Clarified 2026-08-07 against the arm itself. This previously read "Continue then Complete, or dismiss them", which offered dismissal as an equivalent route to finishing; it is not one, and it is the only route a user would find by following the panel.)* **Pause is not "stop tracking forever"; it is "stop tracking, intending to return."**

**11. A frame created by Pause is dismissible exactly like any other**, and dismissal records `Skipped` — meaning *I paused and never went back*, which is true. `interruption-history.md` decision 5 owns this; nothing is added here.

## Alternatives

### A. How Pause is expressed in the event model

1. **A new persisted `Paused` state or `PauseSpan` entity** — **already rejected by ADR 0005**, which found it had no problem statement at MVP scope. Not reopened; recorded so the option is visibly closed.
2. **Complete + Start** — the historical answer, and **impossible**: `Complete` is rejected with a non-empty stack. This is R11's headline instance and is recorded in `principles.md` and `risks.md`.
3. **A specialised Interrupt with no successor.** **Chosen by ADR 0005**, restated here for completeness, not re-decided.

### B. What "Continue" maps to

1. **A dedicated `Continue` transition** — rejected by ADR 0005: it would change no state and have nothing to replay.
2. **Leave the controls unchanged**, so the user presses *Return to Previous* after lunch. Honest and free, and rejected: it describes the mechanism rather than the intent, and at depth 1 the two return buttons are already identical, so the user is asked to choose between two labels that do the same thing at the moment they least want to think.
3. **Relabel Return to Previous as `Continue` while paused.** **Chosen.** No new command, no new path, and the label matches the user's actual intent at the only moment it appears.

### C. Distinguishing "paused" from "Anchor lost track"

1. **A new persisted field on the block or frame** — unambiguous, and rejected: it duplicates information the record already carries, and ADR 0005 explicitly refused a second mechanism for preserving return intent.
2. **Do not distinguish them** — cheapest, and rejected: they demand opposite responses. A pause is continued; a lost-track block has a **wrong inferred end** that wants correcting (**R4**), and conflating them buries the case that costs money.
3. **Derive it from `end_determination` plus end-time ordering, and assert "Paused" only when that is unambiguous.** **Chosen** (decision 3). No new state, and it fails safe: where the evidence does not settle it, Anchor reports observable facts instead of a guess. An earlier draft of this option derived it from position in `closed` and was wrong — `Add` pushes onto that list, so reconstructing a forgotten block during a break would have made Anchor report that it had lost track.

### D. Whether Pause is allowed while interrupted

1. **Reject Pause at depth > 0**, forcing the user to unwind first — and rejected outright: unwinding is what marks work `Skipped`, so this reintroduces the exact record corruption Pause exists to prevent, at the depth where stopping is most likely.
2. **Allow it, and prevent Return to Original afterwards** — rejected: it makes a shipped action conditionally illegal for a reason the user cannot see, and `mvp.md:25` forbids adding return-path complexity.
3. **Allow it, and surface the consequence.** **Chosen** (decision 7), using the return preview that already exists.

### E. Whether Pause gets a hotkey

1. **No hotkey; dashboard and widget only** — smallest change, and rejected: it makes the recorded end late by the cost of reaching the window, which is the one thing Pause's timestamp must be right about.
2. **Reuse an existing binding contextually** — rejected: a key whose meaning depends on state is exactly the "memorable, not findable" failure `visual-redesign.md` E.3 warns about, and a mispress would write a transition.
3. **A sixth global binding.** **Chosen** (decision 6).

## Trade-offs

| | Event model | Continue | Paused vs lost-track | Depth > 0 | Hotkey |
|---|---|---|---|---|---|
| **Chosen** | ADR 0005's specialised Interrupt | Relabelled Return to Previous | Derived from existing fields | Allowed, consequence shown | Sixth binding |
| Complexity | **Low — one transition** | None — a label | Low — a projection read | None — no new rule | **Moderate — a persisted struct grows** |
| Reversibility | Low — a transition is an on-disk fact | High | High | High | Moderate — a settings shape change |
| UX impact | Stopping stops falsifying the record | The label matches the intent | "Why is nothing running?" is answerable | Stopping works at any depth | The end time is the real one |
| Risk if wrong | — (ADR-settled) | "Continue" hides that a stack operation occurred | The derivation is subtle and drifts | A paused task is skipped by Return to Original (**R7 preview mitigates**) | A mispressed key writes a transition |

**Not a trade-off, recorded so it is not mistaken for one:** everything except the `Pause` transition itself already exists and was verified. This feature adds no field, no projection, and no second mechanism.

## UX

Owned by the ux-designer. Stack semantics are `interruption-stack.md`'s and are not restated.

### The paused state

- **Dashboard**: where the active task normally appears, a **Paused** state naming the waiting task, the time it was paused, and how long ago — updating on the existing one-second tick. Not styled as an error or a warning; pausing is a normal thing to do.
- **Widget**: `state` reads **Paused** and the task name is the paused task's, so a glance from across the desk distinguishes paused from stopped without reading. Stack depth continues to display, unchanged.

  **This reinterprets two existing fields; it does not add a fifth.** `visual-redesign.md` fixes the widget's contents at *current task name, elapsed time, stack depth and state, and nothing else*. In the paused state "current task" reads as the **waiting** task and "elapsed" as **how long it has been paused** — same four slots, same 260×90.

  **The widget gets no Pause control.** `visual-redesign.md` D.2 exempts the widget from the 24×24 target floor on the stated grounds that *"the widget has no controls that require aiming"*, and that exemption is load-bearing for its `compact` density. Adding a button would invalidate it. *(Caught on this document's second review pass, which had listed the widget among Pause's control surfaces.)* This is not a limitation in practice and is the strongest argument for decision 6: **the hotkey is how you pause without the dashboard**, which is exactly the moment the widget is the only thing on screen.
- **The distinction from lost-track is stated, not implied.** Where the shape is lost-track rather than paused (decision 3), the surface says Anchor stopped tracking and the block's end is **inferred** — with the correction route, since that block's end is exactly what **R4** is about. The two states never share wording.
- **Empty and paused are never the same words.** "No active task" remains reserved for a genuinely empty stack with nothing waiting.

### Controls

- **Pause** is available whenever a task is active, on the dashboard and via its hotkey — **not on the widget**, for the reason above. It is unavailable, not merely inert, when nothing is active, since its precondition cannot hold.
- While paused, the resume control reads **Continue** and invokes Return to Previous (decision 5). *Return to Original* keeps its name and remains available whenever the stack is non-empty.
- **Complete stays visibly unavailable while paused**, and the reason is stated rather than left to a disabled control. The reason **changes** as frames are resolved: while frames remain it is the non-empty stack; once they are all dismissed it is that nothing is active to complete. Both are stated as they apply, because a control that stays greyed out for a reason that silently changed is worse than one that never moved.
- **Nothing prompts on pause.** No "are you still there", no idle detection, no suggestion to resume — ADR 0001.

### In Interruption History

- A Pause-created frame appears in **Now** like any other frame, and is dismissible. It is **not** labelled as originating from Pause, because the record does not carry that and `interruption-history.md` forbids implying distinctions the record lacks.
- The **return preview** covers decision 7's hazard without new wording: when a paused task is not the root, it is counted among the blocks *Return to Original* will mark `Skipped`.

### Motion

Entering the paused state uses `visual-redesign.md`'s 150–200 ms ease-out. Nothing pulses, blinks, or otherwise nags.

## Technical Constraints

**Inherited, not open:**

- **Pause is a specialised Interrupt with no successor**, leaving `active == None` — ADR 0005, not reopened.
- **`active == None` with a non-empty stack is a supported state** and already reachable in shipped code via the ≥ 1 h gap zone.
- **`Complete` is rejected while the stack is non-empty.**
- **Both returns are already legal with no active task** — verified above; **no change to either arm is in scope**, and a change made "for Pause" would be a regression risk with no requirement behind it.
- **`derived_interruption_status` is the only permitted reading** of interruption state (ADR 0005, risk **R1**).
- **No inference** — ADR 0001. Pause is never automatic.

**This surface's own:**

- **The `Pause` transition and its command must be built.** `TransitionPayload` has no Pause variant. It must close `active` with `UserDetermined`, push its `StackFrame`, and leave `active` as `None` — reusing the `Interrupt` arm's frame construction so the two cannot drift.
- **The transition must be exercised by replay tests, not only live tests.** `log::reader` calls `apply` directly with no dry-run guard, so a Pause arm that mutates before it can fail would corrupt replay rather than return an error — the hazard the `ReturnPrevious` arm's comment already documents.
- **`HotkeyBindings` grows from five fields to six**, and `hotkeys::apply_remap` applies all bindings atomically — either every accelerator registers and persists or none do. A sixth field changes a **persisted settings shape**, so a settings file written by the five-field build must still load. This is the only durable-format change in the feature and it is not in the transition log.
- **Nothing may be logged while paused.** The heartbeat already stops on its own (`should_beat`), and no gap transition is produced (`gap::resolve`). An implementation that beats while paused would append records asserting activity that is not happening.
- **Pause's timestamp is the transition's own**, never a wall-clock read inside the state machine — replay must not depend on when it runs.
- **Paused-for duration is computed in the UI** from the paused block's `end`, never persisted and never sent to the domain.
- **Implementation is blocked on `visual-redesign.md`'s inputs** for its *visual* surface only — the spacing scale, palette and font weights are not in this repository. **The transition, the command and their tests are not so blocked**, being domain work with no design-system dependency.

- **The hotkey binding, however, cannot ship before the paused-state display.** *(Established 2026-08-07 during M3 planning, from the code rather than from preference; it amends the previous sentence, which grouped the hotkey with the unblocked work.)* `hotkeys::register_bindings` iterates the actions and calls `global_shortcut().register` on each **unconditionally** — no enabled flag, no skip list, no dormant-registration path. A new `HotkeyAction` variant is therefore live the day it ships. A live Pause key with no visible paused state realises risk **R19** directly, and assumption **A18** names that display as its only mitigation. The two are one shippable unit. **This does not change decision 6** — Pause still warrants a binding, and `visual-redesign.md`'s frequent-action list was extended to say so; it changes only *when* the binding can land.

## Acceptance Criteria

**The transition**

- Invoking Pause with an active task closes that block with an end equal to the transition's own timestamp and `EndDetermination::UserDetermined`, pushes a frame naming it, and leaves nothing active.
- Stack depth increases by exactly one. The paused block's `derived_interruption_status` reads `Pending`.
- `CaptureOrigin` is unchanged by Pause — a `LiveCapture` block stays `LiveCapture`, not `*Adjusted` and not `ManualEntry`.
- Invoking Pause with **nothing active is rejected** with the domain's own error, and appends no transition. This covers pausing twice.
- Pause at depth 0 yields depth 1; Pause at depth *n* yields depth *n + 1*, with the paused task on top.
- Exactly **one** transition is appended per successful Pause, and none per rejected one.

**Getting out of the paused state**

- Return to Previous while paused resumes the paused task, resolves its block `Resumed`, and leaves depth *n − 1*. No task needs to be started first.
- Return to Original while paused resumes the root and marks every non-root frame `Skipped`, **including the paused task when it is not the root** — and the return preview stated that count beforehand.
- `Start` while paused begins a new task, leaves every frame in place, and resolves nothing.
- Dismissing a Pause-created frame records `Skipped` and does not change what is active.
- `Complete` while paused is rejected. Emptying the stack — by dismissing every frame — does **not** make it available while paused, because the active task is still absent; it becomes available only after work is started or continued.

**Durability**

- Pausing, then quitting cleanly and relaunching, restores the paused state exactly: nothing active, same depth, same frames, same paused-at times — with **no** transition appended by the restart.
- Pausing, then killing the app ungracefully and relaunching, does the same. **No `RecoverGap` is appended**, at any elapsed time, including beyond one hour.
- Sleeping and waking while paused appends no transition, at any elapsed time.
- **No heartbeat is written while paused**, and the log does not grow during a break.
- Replaying a log containing Pause reproduces the same projection as the live run that wrote it, and a snapshot taken while paused restores it identically.

**Display**

- While paused, the dashboard and the widget both show a **Paused** state naming the waiting task and how long it has been paused, advancing once per second.
- The paused state is worded differently from both the empty state and the lost-track state; no two of the three share a message.
- After a crash inside an interruption (frames waiting, latest end `SystemInferred`), the surface does **not** read "Paused". It states that Anchor closed the block itself, that the end may be wrong, and offers the correction route.
- Adding a reconstructed block while paused **does not** change the state from Paused to lost-track, at any position in the day — the test is by `end` time, not by insertion order.
- Where the evidence is ambiguous, the surface states only what is observable — nothing tracked, *n* waiting — and never asserts either "Paused" or a lost-track claim.
- While paused the resume control reads **Continue** and invokes Return to Previous; *Return to Original* keeps its label.
- When the Paused state does **not** hold — including the lost-track and ambiguous shapes — that control keeps its *Return to Previous* label, so "Continue" is never shown for a stop the user did not choose.
- Pause is unavailable when nothing is active, and **absent from the mini widget entirely**, which carries no controls.
- The widget shows the paused task in its existing name slot and the paused-for duration in its existing elapsed slot — four fields in the paused state, as in every other state.
- A Pause-created frame is indistinguishable from an Interrupt-created one in the Interruption History panel.

**Non-regression**

- No existing transition's behaviour changes; in particular neither return arm is modified, and a full Switch/Interrupt/Return cycle is byte-identical in the log before and after this feature.
- Export output is unchanged for any timeline containing no Pause, and a paused block exports exactly as an interrupted one does — the break itself is never billed, because it is not a block.
- A settings file written before the sixth hotkey binding existed still loads, and the app starts with a usable binding set.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
