---
status: draft
date: 2026-07-29
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/principles.md, docs/risks.md, docs/glossary.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/decisions/0004-transition-log-format-and-torn-write-scheme.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, ideas/manual-time-block-entry.md]
---

# Timeline Reconstruction

> Design pass for GitHub issue #15. Follows `.claude/workflows/design.md`. Resolves ADR 0005 open items 1–4. **Not yet reviewed** — `status: draft` until an independent reviewer pass finds no must-fix items.
>
> Depends on the **Timeline Editor** (#14) for the surface these operations happen on, and that in turn on the visual redesign as enabling work. This doc specifies *what the operations mean*, not what the editor looks like.

## Problem

Anchor's record is only as good as the user's capture discipline, and capture has three failure modes it currently cannot recover from:

1. **Work that was never captured.** Risk **R3** (med-high likelihood, high impact) — nothing prevents forgetting to press the hotkey. Until now this risk had *no mitigation at all*: forgotten work was simply lost.
2. **An inferred end time that is wrong.** When Anchor recovers a gap (crash or sleep/hibernate) it infers an end from the last durable write. `interruption-stack.md` has promised since 2026-07-23 that this is "user-correctable… whenever the user next opens the dashboard" — and **no mechanism has ever existed**. That is risk **R9**. Risk **R4** — that a wrong inferred end reaches an export and is billed — rests on the same missing mechanism, since its only mitigation was the correction that does not exist.
3. **Mis-attributed work.** A task tracked under the wrong name, project, or client — or left under an auto-assigned `Anchor N` name (risk **R8**) — fragments export totals under a meaningless label.

`docs/vision/vision.md` requires that a workday be reconstructable "with minimal manual effort and **entirely inside Anchor**." Today, closing any of these three gaps requires editing raw log data or accepting a wrong record. Both are exactly what this project exists to eliminate.

## Goals

- **Every gap between the record and reality can be closed inside Anchor.** No scenario should require hand-editing a log file or shrugging at a wrong entry.
- **The record stays honest about itself.** Reconstructed work is permanently distinguishable from live-captured work, and adjusted work from untouched work — via `CaptureOrigin`, already in the model. A reader can always tell how much of a day was captured versus reconstructed.
- **Reconstruction cannot express something capture could not have produced.** No overlapping blocks, no future-dated work. This is a correction mechanism, not a richer parallel model.
- **Capture discipline stays measurable.** Risk **R10** is that an always-available editing surface erodes the in-the-moment capture this product depends on — reconstruction quietly becoming the default path. This design does not remove that risk, but it makes it *falsifiable*: reconstructed minutes carry `CaptureOrigin::ManualEntry`, so erosion shows up as a falling Capture Rate rather than as invisible drift.
- Ties to `docs/vision/vision.md`'s "minimal manual effort, entirely inside Anchor" and closes the mechanism gap behind **R9**, **R4**, and the only mitigation **R3** has.

## Users

Serves the single primary persona in `docs/product/users.md` — the interrupted billable developer — with no new segment. Two persona properties constrain the design directly:

- *"Values speed and low friction above all."* Reconstruction is the **exception path**. It may be deliberate and slower than capture, but it must never become the fast path, or the product stops being capture-first.
- *Comfortable with hotkeys, not needing onboarding.* No wizards, no explanatory chrome, no first-run tour. Confirmation dialogs are reserved for actions that are **not** reversible — which is why Delete's status depends on undo existing (see UX).

## Alternatives

### A. Ordering authority, once log order stops matching chronological order

A block entered at 16:00 covering a 09:00 span is logged *after* work that happened later. Log order and chronological order stop agreeing.

1. **Re-sort the log so it stays chronological** — keeps one ordering, but destroys the append-only property that all of [ADR 0004](../../decisions/0004-transition-log-format-and-torn-write-scheme.md)'s crash-safety rests on. Rejected outright.
2. **Insert reconstruction transitions with a synthetic `seq`** placing them "where they belong" — preserves apparent chronology in the log, but breaks the monotonic sequence the watermark-based replay filter depends on. Rejected.
3. **Two orderings, each authoritative in its own domain.** **Chosen.** Log order (`seq`) remains authoritative for **replay** — unchanged, ADR 0004 untouched. Block `start` is authoritative for **display and export**. Reconstruction breaks only the *coincidence* that the two used to agree; neither ordering changes meaning.

### B. Overlap policy

Capture can never produce overlapping blocks — Anchor tracks one active task at a time. The question is whether reconstruction may.

1. **Permit overlaps with a visible warning** — more expressive, but export sums durations per task, so an overlap silently inflates a billed total. That is risk **R2**'s failure mode with a new cause, and it lets reconstruction express something the model denies exists. Rejected.
2. **Reject the operation when it would overlap** — keeps the invariant, but forces the user into an error state for what is, on a direct-manipulation surface, an ordinary gesture. Rejected on [`principles.md`](../principles.md) #3 grounds: the model should not make the user fail to satisfy it.
3. **Collision clamping.** **Chosen.** The persisted timeline may never contain overlapping blocks, and the user is never told "no." A drag simply **stops at the neighbouring boundary**: resizing A rightward toward a B that starts at 10:30 clamps exactly at 10:30 and goes no further; moving a block clamps when its leading edge would collide. Neighbours are never pushed, truncated, or rewritten.

   This was preferred over a "split-and-fill" variant that would offer to truncate the neighbour: that mutates a block the user did not select, which is a surprising side effect on a billing record.

### C. Editing blocks that a live interruption stack frame still references

A stack frame is *an unresolved obligation to record the outcome of interrupted work*. It holds `paused_time_block_id` — the only link back — plus its own copy of the paused task's name/project/client for the return path.

1. **Allow all operations, cascading changes into the frame** — most permissive, but deleting a block orphans its frame, and replay then fails with `PausedBlockNotFound`. Corrupting replay to allow an edit is not a trade worth making. Rejected.
2. **Forbid all editing until the frame is resolved** — safest, but a task interrupted this morning and not yet returned to cannot have its `Anchor N` name corrected all day. That is the R8 case, made worse. Rejected.
3. **Three tiers, on one rule: you cannot rewrite history that hasn't finished happening.** **Chosen.**

   | Block state | Permitted |
   |---|---|
   | **Active** | Identity only — that is `Rename`, already shipped |
   | **Open frame** (interrupted, unresolved) | Identity only, **propagated to the frame** so the return path cannot desync |
   | **Resolved** (`Resumed`/`Skipped`) or never interrupted | All five operations |

### D. Rename versus Edit Identity

Both change name/project/client on a Time Block.

1. **Merge into one transition** carrying a target block id, with `Rename` as the case where the target is the active block — one domain fact, one transition. Rejected: it supersedes a shipped, accepted transition and `interruption-stack.md`'s "requires an active task" rule, for a modelling tidiness gain.
2. **Keep two transitions.** **Chosen.** They produce identical state but answer different questions about *how that state came to be*: `Rename` changes the identity of work that **is still happening**; `EditIdentity` corrects the identity of work that **has already happened**. That is [`principles.md`](../principles.md) #6 exactly — persistence captures what became true, the event model captures how. Implementation may be shared; the domain concepts and the transitions stay separate.

### E. The operation set

Each operation must survive [`principles.md`](../principles.md) #1 — a stated problem, not a natural-feeling gesture. This is the test that removed split and merge.

| Operation | Problem it solves | Verdict |
|---|---|---|
| **Add** | Work happened and was never captured (**R3**) | Keep |
| **Resize** | The recorded **timing** was wrong — one boundary moves, duration changes. This is the mechanism **R9**/**R4** have been promised | Keep |
| **Move** | The recorded **placement** was wrong — both boundaries translate, **duration is preserved**. A 60-minute meeting recorded 30 minutes early has the right duration in the wrong place | Keep |
| **Edit Identity** | Work was attributed to the wrong name/project/client, including an uncorrected `Anchor N` (**R8**) | Keep |
| **Delete** | A block records something that never happened — tracking started by mistake | Keep |

**Move exists to correct temporal placement while preserving duration.** That is its problem statement, and it is the whole of it.

**Move was scrutinised specifically**, since two Resize operations can reach the same end state. It survives because it expresses a different intent and a different problem: Resize says *the timing was wrong*, Move says *the duration was right and the position was wrong*. Reaching it by resizing both edges would transiently change the duration to something the user never claims — the operation would misrepresent itself mid-gesture. "Dragging a block feels natural on a timeline" was explicitly **not** accepted as justification.

## Trade-offs

| | Ordering | Overlap | Live-frame editing | Rename vs. Edit Identity | Operation set |
|---|---|---|---|---|---|
| **Chosen** | Two orderings, each authoritative in its domain | Collision clamping; no overlaps ever persisted | Three tiers by block state | Two transitions, shared implementation | Add, Move, Resize, Edit Identity, Delete |
| Complexity | Low — no format change; one display defect to fix | Moderate — clamping must be computed in the domain and mirrored in the editor | Moderate — identity edits must propagate to the frame | Low — second transition, same validation core | Low — five bounded operations |
| Reversibility | High — display ordering can change without touching stored data | Moderate — permitting overlaps later is additive; *forbidding* them later would not be | High — tiers can loosen without a data change | Low — merging them later supersedes a shipped transition | High — an operation can be added later with its own problem statement |
| Risk if wrong | Full-fidelity JSON emits non-chronologically, quietly | An overlap that reaches export inflates a billed total invisibly (**R2** shape) | A deleted open-frame block breaks replay outright (`PausedBlockNotFound`) | Two transitions for one fact reads as redundancy to a newcomer | An operation without a real problem accretes semantics forever (the split/merge lesson) |

## UX

Owned by the ux-designer; the Timeline Editor's visual form belongs to #14, not here.

- **Add** — draw a span on the Timeline Editor. Opens naming with the same autocomplete Rename uses (Task Templates plus past task history, source-tagged). The new block is `CaptureOrigin::ManualEntry`.

  **An added block is a single independent Time Block. It never touches the interruption stack**, carries no `InterruptionOutcome` (so its `DerivedInterruptionStatus` is `NeverInterrupted`), and pushes no frame. There is deliberately **no way to reconstruct an interruption *relationship*** after the fact — you can add the two blocks that a real interruption would have produced, but not the stack semantics between them. The stack is a live structure recording what the user actually did in the moment; inventing one retroactively would fabricate provenance, which is the opposite of what `CaptureOrigin` exists to prevent.
- **Move / Resize** — direct manipulation, **clamped at neighbouring boundaries**. The clamp must be *visible*: the block stops at the boundary and the boundary itself indicates it is the limit. A user who drags harder must understand why nothing more is happening — a silent clamp reads as a broken UI.
- **Edit Identity** — same fields and autocomplete as Rename, on a historical block.
- **Delete** — no confirmation dialog *provided undo exists*. As written that is a circular dependency: this doc justified skipping confirmation by calling delete reversible, while deferring undo to the Timeline Editor (#14). **Resolved here as a hard prerequisite** — #14 must provide undo, or delete gains a confirmation step. Deleting a Time Block is destroying a billing record; it may not be both unconfirmed and irreversible.
- **Reconstructed and adjusted blocks are visually marked**, permanently, from `CaptureOrigin`. This is the same principle as surfacing `SystemInferred` ends distinctly: the record must never let reconstructed work pass as captured work.
- **Nothing prompts.** Reconstruction is always user-initiated. Anchor never suggests that a gap "looks like" untracked work — that would be idle detection, which `docs/vision/vision.md` puts out of scope and [ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md) rules out.

## Technical Constraints

Owned by technical-architect / senior-software-engineer. Implementation is gated on ADR 0005's remaining items being specified there — this section states what the design requires of them.

- **New transitions carrying explicit times.** Every existing `TransitionPayload` variant derives its block boundaries from *when the transition was logged*; reconstruction needs variants carrying an author-chosen start and end. Additive to ADR 0004's schema, in the same way `Rename` was.
- **Log order stays authoritative for replay.** No reordering, no synthetic sequence numbers. ADR 0004 is unaffected. What must change is any code assuming `closed` is chronological.
- **Full-fidelity JSON export must sort by `start`.** It currently emits in `closed` order, which stops being chronological the moment a block is inserted. The History View already sorts by `start` and needs no change. *(Defect found during this design pass; not previously recorded.)*
- **The editor clamps; the domain rejects.** These are deliberately different mechanisms for one invariant. Clamping is a *usability* device: it keeps the gesture from ever producing an overlap, so the user is never told "no." Rejection is the *guarantee*: the state machine refuses an overlapping result regardless of caller, because a UI-only invariant is one bug — or one non-editor caller — away from an overlapping billing record. A correctly working editor should never trigger the rejection.
- **Identity edits on an open-frame block must update the frame's copy** of name/project/client atomically with the block, or the return path resumes under a stale identity.
- **Delete is rejected while a frame references the block.** Non-negotiable: it would orphan `paused_time_block_id` and break replay.
- **No block may end in the future**, and no block may overlap the currently-active block's live span (start → now). Both follow from "every block represents work that actually happened."
- **`CaptureOrigin` transitions**: Add produces `ManualEntry`. Move, Resize, and Edit Identity move a block to the `*Adjusted` variant of its existing origin, never rewriting origin — a manually entered block nudged once must stay distinguishable from a live capture that needed correcting. Delete removes the block, so neither applies. Already expressible — no model change.
- **`EndDetermination` after an edit.** Any operation that sets a block's **end** makes it `UserDetermined` — Add, Resize of the end boundary, and Move (which sets both). Editing only the *start* leaves it untouched, because it says nothing about how the end was established.

  **This is the R9 case working correctly, and it must not be left implicit.** Correcting a wrongly inferred end is the headline reason this feature exists; if the block stayed `SystemInferred` afterwards, the record would permanently claim Anchor inferred a time the user actually determined — and the distinct visual treatment `interruption-stack.md` requires for inferred ends would keep flagging a block the user has already fixed.
- **Collision rules apply to Add, not only to Move and Resize.** A new block's **start point must fall in free space**; its end clamps at the next occupied boundary. Drawing a span across an existing block therefore yields a block ending where that block begins, rather than an overlap or a rejection. Beginning a draw *inside* an existing block has no free space to occupy and is not a valid start.
- **The active block is a collision boundary whose end is `now`.** Unlike every other neighbour it has no fixed end, so its occupied span grows as the drag proceeds. Two consequences: the editor must render it as occupied up to the present moment, and **the domain must re-evaluate the collision at commit time, not trust a boundary computed when the gesture began.** In practice this means reconstruction can never place work inside the span Anchor believes is currently being tracked — which is correct: per the record, the user was doing the active task.
- **Snapshot compatibility**: reconstruction transitions are ordinary log records and must survive compaction's watermark replay like any other. No new requirement beyond ADR 0005's existing snapshot-payload guarantee.

## Acceptance Criteria

- Adding a block for a past span creates a Time Block with `CaptureOrigin::ManualEntry`, and it appears in the History View ordered by its `start`, not by when it was entered.
- After adding a block covering an earlier span than existing blocks, replaying the log from disk reproduces the identical timeline — proving log order remained authoritative for replay while `start` governs display.
- Full-fidelity JSON export emits records in ascending `start` order, including after a block has been inserted covering an earlier span.
- Resizing a block toward a neighbour stops exactly at the neighbour's boundary; continuing the gesture produces no further change, no overlap, and no modification to the neighbour.
- Moving a block preserves its duration exactly, and clamps at a neighbouring boundary without altering that neighbour.
- An attempt to persist an overlapping block is rejected by the domain even when submitted directly, not only by the editor.
- Editing the identity of a block whose frame is still open updates both the block and the frame; returning to that task afterwards resumes under the corrected identity.
- Deleting a block whose frame is still open is rejected; the frame and its `paused_time_block_id` remain intact and replay succeeds.
- Deleting a resolved or never-interrupted block removes it from the timeline and from subsequent exports, leaves neighbouring blocks untouched, and leaves a gap rather than closing one.
- Editing the identity of a resolved block changes its name/project/client without altering its start, end, or `InterruptionOutcome`; a subsequent export groups it under the corrected identity.
- Correcting the end of a block whose `EndDetermination` was `SystemInferred` leaves it `UserDetermined`, and it stops being rendered as an inferred end — the R9 correction path, end to end.
- Drawing a new block whose span crosses an existing one produces a block ending at that block's start; beginning a draw inside an existing block is not a valid start.
- Reconstruction cannot place a block inside the currently-active block's span, and that check is evaluated against the present moment at commit time rather than against a boundary captured when the gesture began.
- Move, Resize, and Edit Identity each move the affected block to the `*Adjusted` variant of its existing `CaptureOrigin`, and never change whether its origin is live or manual. (Add creates a block at `ManualEntry`; Delete removes one, so neither applies.)
- No operation can produce a block ending in the future, or one overlapping the currently-active block's span.
- Capture Rate (`docs/vision/vision.md`) falls measurably when a day's minutes are reconstructed rather than captured — the metric reflects reconstruction rather than hiding it.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
