---
status: accepted
date: 2026-08-01
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/principles.md, docs/risks.md, docs/glossary.md, docs/architecture/constraints.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/decisions/0004-transition-log-format-and-torn-write-scheme.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0006-stable-persistent-time-block-identity.md, docs/assumptions.md, ideas/manual-time-block-entry.md]
---

# Timeline Reconstruction

> Design pass for GitHub issue #15. Follows `.claude/workflows/design.md`. Resolves ADR 0005 open items 1–4, and the reconstruction-payload question [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md) delegates to this doc (alternative F). **`status: accepted` 2026-08-01**, after an independent review that found no architectural blockers and six must-fixes, all applied.
>
> **Accepted is not ready to implement.** Three things gated that, none of them design questions. **One is now closed:** risk **R14** — a `seq` consumed by an append that did not durably complete — was fixed in the writer on 2026-08-02, so `seq` uniqueness is now an invariant the writer enforces rather than one ADR 0004 assumed. **Two remain:** the Timeline Editor (#14) owes Add/Move/Resize a surface, and the compaction snapshot (#8) must carry block ids or reconstruction loses reach over pre-watermark history. *(A third — undo's identity question — was closed on 2026-08-02: Delete is confirmed and has no undo, so nothing is ever re-created and ADR 0006's revisit trigger did not fire. See the Delete bullet under UX.)*
>
> Depends on the **Timeline Editor** (#14) for **Add, Move and Resize** — the operations that need a spatial surface — and that in turn on the visual redesign as enabling work. This doc specifies *what the operations mean*, not what the editor looks like.
>
> **Edit Identity and Delete are not Editor-gated.** They need a selected row, not direct manipulation, and both accepted docs that speak to this already scope the dependency that way: `docs/product/mvp.md` makes the Editor a hard prerequisite because "move and resize are meaningless against the tabular History View" — naming two operations, not five — and `docs/glossary.md` defines the History View as being for "reading **and editing** work as a structured list." An earlier draft of this header gated all five on #14, which would have sequenced **R8**'s only mitigation (correcting an uncorrected `Anchor N` on a closed block) behind the visual redesign *and* the Editor for no reason either doc supports. The two operations that unblock **R8** and **R9** can ship on the surface that already exists.
>
> **Depends on [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md) for persistent Time Block identity.** Four of the five operations name a block that already exists, from a session that may be days later — the first requirement in Anchor for a reference that survives a restart. That ADR decides how identity is derived and what it guarantees; this doc states only what reconstruction *needs* from it and does not restate the derivation.

## Problem

Anchor's record is only as good as the user's capture discipline, and capture has three failure modes it currently cannot recover from:

1. **Work that was never captured.** Risk **R3** (med-high likelihood, high impact) — nothing prevents forgetting to press the hotkey. Until now this risk had *no mitigation at all*: forgotten work was simply lost.
2. **An inferred end time that is wrong.** When Anchor recovers a gap (crash or sleep/hibernate) it infers an end from the last durable write. `interruption-stack.md` has promised since 2026-07-23 that this is "user-correctable… whenever the user next opens the dashboard" — and **no mechanism has ever existed**. That is risk **R9**. Risk **R4** — that a wrong inferred end reaches an export and is billed — is **partly** mitigated already: the 60-second heartbeat bounds the inference error to roughly a minute rather than hours, and `recovered-gap` entries are surfaced distinctly. What does not hold is the third mitigation, "inviting correction," because the correction has no mechanism. This feature supplies it.
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
- *Comfortable with hotkeys, not needing onboarding.* No wizards, no explanatory chrome, no first-run tour. Confirmation dialogs are reserved for actions that are **not** reversible — which is exactly why Delete gets one (see UX). This rule is what *permits* that confirmation rather than what argues against it: Delete is the only operation of the five that destroys a billing record, and MVP gives it no undo.

## Alternatives

### A. Ordering authority, once log order stops matching chronological order

A block entered at 16:00 covering a 09:00 span is logged *after* work that happened later. Log order and chronological order stop agreeing.

1. **Re-sort the log so it stays chronological** — keeps one ordering, but destroys the append-only property that all of [ADR 0004](../../decisions/0004-transition-log-format-and-torn-write-scheme.md)'s crash-safety rests on. Rejected outright.
2. **Insert reconstruction transitions with a synthetic `seq`** placing them "where they belong" — preserves apparent chronology in the log, but breaks the monotonic sequence the watermark-based replay filter depends on. Rejected. *(Since [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md), this option is worse than it was when rejected: `seq` is now also the input to block identity, so a synthetic `seq` would collide with an existing block's id. The original reason stands on its own; this simply removes any temptation to revisit it.)*
3. **Two orderings, each authoritative in its own domain.** **Chosen.** Log order (`seq`) remains authoritative for **replay** — unchanged, ADR 0004 untouched. Block `start` is authoritative for **display and export**. Reconstruction breaks only the *coincidence* that the two used to agree; neither ordering changes meaning.

### B. Overlap policy

Capture can never produce overlapping blocks — Anchor tracks one active task at a time. The question is whether reconstruction may.

1. **Permit overlaps with a visible warning** — more expressive, but export sums durations per task, so an overlap silently inflates a billed total. That is risk **R2**'s failure mode with a new cause, and it lets reconstruction express something the model denies exists. Rejected.
2. **Reject the operation when it would overlap** — keeps the invariant, but forces the user into an error state for what is, on a direct-manipulation surface, an ordinary gesture. Rejected on [`principles.md`](../principles.md) #3 grounds: the model should not make the user fail to satisfy it.
3. **Collision clamping.** **Chosen.** The persisted timeline may never contain overlapping blocks, and the user is never told "no." A drag simply **stops at the neighbouring boundary**: resizing A rightward toward a B that starts at 10:30 clamps exactly at 10:30 and goes no further; moving a block clamps when its leading edge would collide. Neighbours are never pushed, truncated, or rewritten.

   This was preferred over a "split-and-fill" variant that would offer to truncate the neighbour: that mutates a block the user did not select, which is a surprising side effect on a billing record.

### C. Editing blocks that a live interruption stack frame still references

A stack frame is *an unresolved obligation to record the outcome of interrupted work*. It holds `paused_time_block_id` — the only link back — plus its own copy of the paused task's name/project/client for the return path.

   **That reference is not the same kind of reference reconstruction needs**, and conflating them would misread what [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md) changed. `paused_time_block_id` is created and consumed *within one replay*: the frame and the block it points at are both rebuilt from the same log in the same pass, so the link holds even when ids are regenerated per replay. A reconstruction transition is the opposite — it is **written** in one session naming a block **built** in another. Stable identity is what makes the second kind possible; the first never needed it.

1. **Allow all operations, cascading changes into the frame** — most permissive, but deleting a block orphans its frame, and replay then fails with `PausedBlockNotFound`. Corrupting replay to allow an edit is not a trade worth making. Rejected.
2. **Forbid all editing until the frame is resolved** — safest, but a task interrupted this morning and not yet returned to cannot have its `Anchor N` name corrected all day. That is the R8 case, made worse. Rejected.
3. **Three tiers, on one rule: you cannot reshape a block whose span is not yet fixed.** **Chosen.**

   | Block state | Permitted |
   |---|---|
   | **Active** | Identity only — that is `Rename`, already shipped |
   | **Open frame** (interrupted, unresolved) | Identity only, **propagated to the frame** so the return path cannot desync |
   | **Resolved** (`Resumed`/`Skipped`) or never interrupted | All five operations |

   **Dragging the active block's *start* is forbidden, and that needs saying explicitly** — ADR 0005's open item 3 names it as its own undefined case, and the tier table above would otherwise answer it only by implication. It is also the case where the rule needs stating carefully: the active block's *end* has not finished happening, but its *start* has, so "you cannot rewrite history that hasn't finished happening" would on its own permit a start-drag. The rule is deliberately the stronger one — **a block whose span is still being determined is not a reconstruction target at all** — because a block with a moving end and a moved start has no stable span for the domain to validate an overlap against, and the collision check for every *other* operation already treats the active block as an occupied region growing to `now`.

   This closes a real and common case: the user starts work, then realises twenty minutes in that they never pressed the hotkey. **The path is Add, not a start-drag** — draw the missing span, which clamps at the active block's start (see Technical Constraints), and the result is two blocks: a `ManualEntry` block for the unrecorded twenty minutes and the untouched live capture. That is *better* than stretching one block backwards, not merely an acceptable substitute — stretching would relabel twenty reconstructed minutes as live-captured, which is exactly the provenance erosion `CaptureOrigin` exists to prevent, and would make Capture Rate (**R10**'s falsifiability test) read the day as fully captured.

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

### F. What a reconstruction payload names its target block by

Move, Resize, Edit Identity and Delete all act on a block that already exists. [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md) makes a block referenceable across restarts by deriving `id` from the `seq` of the transition that created it — and then explicitly **declines to decide which of the two the payload should carry**, on the grounds that the choice belongs to whichever design specifies those payloads. This is that design, so it is decided here.

1. **Carry the derived `Uuid`.** **Chosen.** `TimeBlock.id` is already a `Uuid`, the frontend already types it, and the editor selects *a block* — so the value it has in hand is the id. Nothing has to be resolved at write time, and a reference means the same thing whether it is read by replay, by a test, or by a human.
2. **Carry the `seq` instead**, deriving the id in memory only. Genuinely attractive, and the cost is *not* that anything has to be computed at write time — `TimeBlock` would simply carry its creating `seq` alongside the derived id, `StackView` would serialise it, and the editor would send a value it already holds, exactly as symmetric as option 1. (A UUIDv5 cannot be inverted back to its input; any design that required that would be broken, not merely awkward.) What it buys is real: the derivation stays a private in-memory detail that could still be changed, and a reconstruction line stays legible by eye against the line it targets — one of the two axes ADR 0004 chose JSON Lines on.

   **Rejected on one ground: it puts two spellings of the same reference into the system** — a `Uuid` in memory, in `StackView` and in the frontend, which already types `TimeBlock.id` as one, against a `seq` on disk. Every reader then has to know which side of the boundary it is on to compare two references, and the id field that exists today would become a value with no persisted counterpart. The legibility gain is also narrower than it first appears: the id is ADR 0006's UUIDv5 over the decimal `seq`, so a reader *with the namespace to hand* can reproduce it and check the match — only a reader without it loses. (Restated here rather than merely cited because the trade-off cannot be weighed without it; ADR 0006 remains the authority, and changing the derivation is one of its own revisit triggers.)

**The cost of choosing 1 is stated plainly:** it makes ADR 0006's namespace and encoding a permanent on-disk contract from the first reconstruction transition ever written, rather than a changeable in-memory detail. That is a real loss of future freedom, accepted deliberately — a reference that has to be computed before it can be compared is the kind of indirection that is cheap to add and expensive to live with.

## Trade-offs

| | Ordering | Overlap | Live-frame editing | Rename vs. Edit Identity | Operation set | Target reference |
|---|---|---|---|---|---|---|
| **Chosen** | Two orderings, each authoritative in its domain | Collision clamping; no overlaps ever persisted | Three tiers by block state | Two transitions, shared implementation | Add, Move, Resize, Edit Identity, Delete | Payloads carry the derived `Uuid`, not the `seq` |
| Complexity | Low — no format change; one display defect to fix | Moderate — clamping must be computed in the domain and mirrored in the editor | Moderate — identity edits must propagate to the frame | Low — second transition, same validation core | Low — five bounded operations | Low — the value is already in hand on both sides |
| Reversibility | High — display ordering can change without touching stored data | Moderate — permitting overlaps later is additive; *forbidding* them later would not be | High — tiers can loosen without a data change | Low — merging them later supersedes a shipped transition | High — an operation can be added later with its own problem statement | **Lowest of the six** — it makes ADR 0006's namespace and encoding a permanent on-disk contract from the first such line written |
| Risk if wrong | Full-fidelity JSON emits non-chronologically, quietly | An overlap that reaches export inflates a billed total invisibly (**R2** shape) | A deleted open-frame block breaks replay outright (`PausedBlockNotFound`) | Two transitions for one fact reads as redundancy to a newcomer | An operation without a real problem accretes semantics forever (the split/merge lesson) | Changing the derivation afterwards **orphans** every stored reference — it would not re-point them, since no block matches the old value, and replay escalates an unresolvable reference to `ReplayError::Inconsistent`, so **the app does not start** until a backup is restored |

## UX

Owned by the ux-designer; the Timeline Editor's visual form belongs to #14, not here.

- **Add** — draw a span on the Timeline Editor. Opens naming with the same autocomplete Rename uses (Task Templates plus past task history, source-tagged). The new block is `CaptureOrigin::ManualEntry`.

  **An added block is a single independent Time Block. It never touches the interruption stack**, carries no `InterruptionOutcome` (so its `DerivedInterruptionStatus` is `NeverInterrupted`), and pushes no frame. There is deliberately **no way to reconstruct an interruption *relationship*** after the fact — you can add the two blocks that a real interruption would have produced, but not the stack semantics between them. The stack is a live structure recording what the user actually did in the moment; inventing one retroactively would fabricate provenance, which is the opposite of what `CaptureOrigin` exists to prevent.
- **Move / Resize** — direct manipulation, **clamped at neighbouring boundaries**. The clamp must be *visible*: the block stops at the boundary and the boundary itself indicates it is the limit. A user who drags harder must understand why nothing more is happening — a silent clamp reads as a broken UI.
- **Edit Identity** — same fields and autocomplete as Rename, on a historical block. **Reachable from the History View**, not only the Timeline Editor: it changes a label, not a span, so it needs a selected row rather than a spatial gesture.
- **Delete** — **also reachable from the History View**, for the same reason as Edit Identity: removing a block is a row-level action. **Delete is confirmed, and there is no undo.** *(Decided 2026-08-02 by a dedicated undo design pass and its independent review; supersedes this bullet's earlier "no confirmation dialog provided undo exists," which was circular — it justified skipping confirmation by calling delete reversible while deferring reversibility to #14.)*

  The standing rule is unchanged and is what forces the choice: deleting a Time Block destroys a billing record, so it may not be both unconfirmed and irreversible. This doc's own persona rule reserves confirmations **for** actions that are not reversible, which makes this the case they exist for rather than an exception to them.

  **Why confirmation rather than undo, stated as the smallest thing that meets the requirement.** The accepted doc set requires that a delete not be silently irrecoverable. It does **not** require multi-level undo, undo surviving a restart, durable deletion history, or an audit of deletion events — that was checked against every accepted document, not assumed. Confirmation satisfies the requirement with no new domain concept, no new persisted state, and no consequence for export, compaction, collision detection, replay or identity.

  **A mistaken delete is not unrecoverable — the recovery path is Add**, and what it costs is exactly what it should. The re-added block gets a new id and `CaptureOrigin::ManualEntry`, and both are *truthful*: what is now on the timeline genuinely is a manual entry, and Capture Rate correctly dips to reflect that the user had to re-enter it by hand. An undo that restored the original id and origin would make the record claim a continuity that did not happen.

  **The preferred evolution, if confirmation proves insufficient in real use: deferred commit.** The Delete gesture removes the block from the display immediately and offers an undo affordance; the transition is appended at a boundary (a timer, focus loss, or the next user action). Undo inside that window writes nothing at all. What makes this the right next step rather than durable undo is that it treats **undo as an editing affordance, not a historical event** — cancelling an edit before it becomes history, which is what the rest of this product already does with Continue Session ([ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md): a UI action, deliberately not a transition). Nothing outside #14 would need to know, so it needs no ADR either. Its one real cost, to be weighed when the time comes: it opens a bounded window in which a user action is not yet durable, in an architecture whose posture is otherwise durable-before-committed.

  **Durable undo was designed, reviewed, and deliberately not adopted.** The design (Delete as a tombstone rather than a removal, with a `Restore` transition) is architecturally sound — it preserves [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md)'s derivation intact, keeps replay a pure fold, and keeps the one-block-per-transition invariant. It was rejected because it buys durable, restart-surviving undo that **no accepted requirement asks for**, and charges for it across six subsystems: export and full-fidelity JSON must filter tombstones, Capture Rate must exclude them, the compaction snapshot gains a third payload requirement after A10 and A14, the reconstruction domain must reject operations against them, and `next_default_name` must deliberately *not* filter them. That spread is itself the signal — an abstraction whose obligations reach that far while its stated problem is a keystroke's reversibility is premature. Recorded rather than discarded: if deletion-as-a-durable-fact ever acquires a real consumer, the design exists and the argument for it will be a requirement rather than an anticipation.
- **Reconstructed and adjusted blocks are visually marked**, permanently, from `CaptureOrigin`. This is the same principle as surfacing `SystemInferred` ends distinctly: the record must never let reconstructed work pass as captured work.
- **Nothing prompts.** Reconstruction is always user-initiated. Anchor never suggests that a gap "looks like" untracked work — that would be idle detection, which `docs/vision/vision.md` puts out of scope and [ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md) rules out.

## Technical Constraints

Owned by technical-architect / senior-software-engineer. Implementation is gated on ADR 0005's remaining items being specified there — this section states what the design requires of them.

- **New transitions carrying explicit times.** Every existing `TransitionPayload` variant derives its block boundaries from *when the transition was logged*; reconstruction needs variants carrying an author-chosen start and end. Additive to ADR 0004's schema, in the same way `Rename` was.
- **A block reference that survives a restart** — supplied by [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md), and the single hardest requirement this feature places on the architecture. Move, Resize, Edit Identity and Delete each name a block created in an earlier session, so a `Uuid` regenerated on every replay cannot serve; the payload carries the derived id per alternative F. What reconstruction requires of that identity, and nothing more:

  - it is **the same value on every replay of the same log**, so a reference written on Tuesday resolves on Friday;
  - it is **unique within the log**, so a reference resolves to exactly one block;
  - it is **assigned to every block the log has ever produced**, so history is addressable back to the first line rather than from some cut-over point onward.

  How those hold is ADR 0006's to state, not this doc's. Two of its consequences are worth naming here because they land on reconstruction directly: the guarantee is scoped to **one log lineage** (assumption **A13** — a restored backup or a second install reuses ids for unrelated work, which is accepted because nothing exports or consumes an id), and it rests on the invariant that **a transition creates at most one Time Block**. Reconstruction satisfies that invariant as designed — Add creates one block, the other four create none, and the overlap policy is clamping rather than splitting — but it is a constraint on any *future* operation added here, not a fact to be assumed.
- **Deleting a block does not free its identity.** The transition that created it stays in the log, so its `seq` is never reissued and its id is never reused by a later block. Nothing in reconstruction may depend on the reverse.
- **Log order stays authoritative for replay.** No reordering, no synthetic sequence numbers. ADR 0004's record format, checksum framing and watermark are unaffected (its *snapshot* payload is not — see below). What must change is any code assuming `closed` is chronological.
- **Full-fidelity JSON export must sort by `start`.** It currently emits in `closed` order, which stops being chronological the moment a block is inserted. The History View already sorts by `start` and needs no change. *(Latent defect found during this design pass — `closed` order coincides with `start` order today, because the state machine closes the active block at the instant the next one starts. Reconstruction is what breaks the coincidence.)* **Now carried in [`export.md`](export.md)**, which owns export behaviour, with its own acceptance criterion — an implementer working from that doc would otherwise never learn the requirement exists.
- **The editor clamps; the domain rejects.** These are deliberately different mechanisms for one invariant. Clamping is a *usability* device: it keeps the gesture from ever producing an overlap, so the user is never told "no." Rejection is the *guarantee*: the state machine refuses an overlapping result regardless of caller, because a UI-only invariant is one bug — or one non-editor caller — away from an overlapping billing record. A correctly working editor should never trigger the rejection.
- **Identity edits on an open-frame block must update the frame's copy** of name/project/client atomically with the block, or the return path resumes under a stale identity.
- **Delete is rejected while a frame references the block.** Non-negotiable: it would orphan `paused_time_block_id` and break replay. *(This justification depends on Delete physically removing the block, which the 2026-08-02 undo pass confirmed it does. Had Delete become a tombstone, the id would have survived and this reasoning would have been false while the rule stayed right — noted so a future change to Delete's semantics revisits the reason and not only the rule.)*
- **Delete must not let an `Anchor N` name be reissued.** `stack.rs`'s `next_default_name` takes the maximum `N` among *today's* blocks, so physically deleting today's highest-numbered auto-named block makes the next unnamed task reuse that name — two unrelated pieces of work under one name on one day, which export then groups into a single row. That is **R8** arriving through the delete path. Found during the undo design pass; it is a property of Delete itself, not of undo, so it lands here rather than in #14.
- **No block may end in the future**, and no block may overlap the currently-active block's live span (start → now). Both follow from "every block represents work that actually happened."
- **`CaptureOrigin` transitions**: Add produces `ManualEntry`. Move, Resize, and Edit Identity move a block to the `*Adjusted` variant of its existing origin, never rewriting origin — a manually entered block nudged once must stay distinguishable from a live capture that needed correcting. Delete removes the block, so neither applies. Already expressible — no model change.
- **`EndDetermination` after an edit.** Any operation that sets a block's **end** makes it `UserDetermined` — Add, Resize of the end boundary, and Move (which sets both). Editing only the *start* leaves it untouched, because it says nothing about how the end was established.

  **This is the R9 case working correctly, and it must not be left implicit.** Correcting a wrongly inferred end is the headline reason this feature exists; if the block stayed `SystemInferred` afterwards, the record would permanently claim Anchor inferred a time the user actually determined — and the distinct visual treatment `interruption-stack.md` requires for inferred ends would keep flagging a block the user has already fixed.

  **`EndDetermination` travels with the end value, in both directions.** *(Added 2026-08-02, from the undo design pass. An earlier formulation of that pass claimed all three metadata fields are monotonic — that is false for this one, and the error is worth recording because the mirror of the paragraph above is what exposed it.)* If an operation ever sets a block's end **back** to a value Anchor inferred, `EndDetermination` returns to `SystemInferred` with it. Otherwise the record claims the *user* determined a time Anchor actually inferred — the exact inversion of the failure above — and per this doc's own acceptance criterion the block has already stopped rendering as an inferred end, so **R4**'s one surviving mitigation ("surfaces `recovered-gap` entries distinctly, inviting correction") is silently switched off for precisely the block that still needs it.

  The general rule, since the three fields are not the same kind of thing and treating them alike is what produced the error:

  | Field | Describes | Behaviour |
  |---|---|---|
  | `CaptureOrigin`'s adjusted flag | **How the record has been handled** — "has this ever been touched?" | **Monotonic.** Never reverts; `adjusted()` is idempotent and one-way by construction (`model.rs`). A block edited twice to a net-zero change was still edited. |
  | `EndDetermination` | **How the current end value was arrived at** | **Travels with the value.** It is a property *of that end*, not of the block's history. |
  | `InterruptionOutcome` | The outcome of an interruption | **Untouched by reconstruction** — resolved only by the interruption stack. |
- **Collision rules apply to Add, not only to Move and Resize.** A new block's **start point must fall in free space**; its end clamps at the next occupied boundary. Drawing a span across an existing block therefore yields a block ending where that block begins, rather than an overlap or a rejection. Beginning a draw *inside* an existing block has no free space to occupy and is not a valid start.
- **The active block is a collision boundary whose end is `now`.** Unlike every other neighbour it has no fixed end, so its occupied span grows as the drag proceeds. Two consequences: the editor must render it as occupied up to the present moment, and **the domain must re-evaluate the collision at commit time, not trust a boundary computed when the gesture began.**

  **"Commit time" means the transition's own timestamp, never a wall-clock read inside the domain.** The state machine is the one place live commands and replay share, and `reader.rs` calls it on every historical line — so a `Utc::now()` inside it would make replay depend on when it runs, breaking the reproducibility everything else here rests on. The live path supplies `now` as the record's timestamp; replay supplies the recorded one; the validation code cannot tell them apart and must not try. *(The rules as written appear to be safe either way — no block may end in the future, so a reconstructed block lies entirely before the active block's start and a later `now` cannot create a new conflict. That is an argument, not a test: it should be asserted, not relied on.)* In practice this means reconstruction can never place work inside the span Anchor believes is currently being tracked — which is correct: per the record, the user was doing the active task.
- **Snapshot compatibility**: reconstruction transitions are ordinary log records and must survive compaction's watermark replay like any other.

  **This does place a new requirement on the snapshot**, and an earlier draft of this doc wrongly said it did not — it claimed nothing was needed beyond ADR 0005's payload guarantee, which covers unresolved stack frames (assumption **A10**) and says nothing about identity. Blocks below the watermark are never replayed, so after a compaction their ids can come only from the snapshot. If it omits them, every block older than the watermark becomes unaddressable and reconstruction silently loses reach over exactly the history most likely to need correcting. [ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md) makes persisting each block's `id` (or the `seq` it derives from) a normative snapshot requirement; assumption **A14** records it alongside A10 so a future snapshot implementer finds the whole payload contract in one place. Compaction is unimplemented (#8), so this is a constraint on work not yet done rather than a defect in shipped code.

## Acceptance Criteria

- Adding a block for a past span creates a Time Block with `CaptureOrigin::ManualEntry`, and it appears in the History View ordered by its `start`, not by when it was entered.
- After adding a block covering an earlier span than existing blocks, replaying the log from disk reproduces the identical timeline — **including every block's `id`** — proving log order remained authoritative for replay while `start` governs display.
- A reference to a block, written by a reconstruction transition in one session, still resolves to that same block after the app is restarted and the log replayed from scratch. This is the criterion ADR 0006 exists to satisfy; before it, the reference resolved only within the session that wrote it.
- Editing a block that was created **before** the identity scheme was in place succeeds — history is addressable to the first line of the log, with no cut-over generation of blocks that cannot be reconstruction targets.
- No two blocks in a replayed timeline share an `id`. Asserted explicitly rather than assumed: `resolve_paused` and `derived_status` use `id` as a lookup key, so a collision resolves the wrong block silently instead of failing.
- Full-fidelity JSON export emits records in ascending `start` order, including after a block has been inserted covering an earlier span.
- Resizing a block toward a neighbour stops exactly at the neighbour's boundary; continuing the gesture produces no further change, no overlap, and no modification to the neighbour.
- Moving a block preserves its duration exactly, and clamps at a neighbouring boundary without altering that neighbour.
- An attempt to persist an overlapping block is rejected by the domain even when submitted directly, not only by the editor.
- Editing the identity of a block whose frame is still open updates both the block and the frame; returning to that task afterwards resumes under the corrected identity.
- Move, Resize and Delete are each rejected against the **currently active** block, including a Resize of its start — the tier table's "identity only" rule, asserted rather than left implicit. `Rename` on that block still succeeds.
- Move and Resize are each rejected against a block whose interruption frame is still **open**, while Edit Identity on the same block succeeds — the open-frame tier, tested on the operations it restricts and not only on the ones it permits.
- Deleting a block whose frame is still open is rejected; the frame and its `paused_time_block_id` remain intact and replay succeeds.
- After a compaction, a block whose creating transition now sits **below the watermark** is still a valid reconstruction target — editing it succeeds and resolves to the same block. This is the criterion that fails if the snapshot omits ids, and it is the only one that catches it. *(Untestable until compaction exists (#8); recorded now because the criterion is the contract, not the test schedule.)*
- Deleting a block requires an explicit confirmation; dismissing or cancelling it leaves the block and the log untouched. *(The standing rule is "never both unconfirmed and irreversible" — MVP satisfies it by confirming. A future build that adds undo may drop the confirmation, but not both.)*
- Correcting a block's end **back** to the value Anchor originally inferred returns its `EndDetermination` to `SystemInferred`, and it renders as an inferred end again — the R9 path in reverse, which is what stops a reverted correction from claiming the user determined an inferred time.
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
