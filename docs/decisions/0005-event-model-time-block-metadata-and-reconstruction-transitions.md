---
status: accepted
date: 2026-07-28
owner: erich
related: [docs/decisions/0004-transition-log-format-and-torn-write-scheme.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/concept/concept.md, docs/vision/vision.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/glossary.md, docs/principles.md, docs/risks.md, docs/architecture/constraints.md]
---

# 0005: Event Model — Time Block Metadata and Reconstruction Transitions

> **Accepted 2026-07-28**, together with `vision.md` and `concept.md`. The decisions below came out of a `grill-with-docs` session (14 branches) and survived three independent review passes returning 15, 14, and 12 findings respectively — each round's fixes introduced new problems the next round caught, which is why three ran. Every reported blocker was addressed and the author reviewed the full diff.
>
> **Accepted with nine open items** (below). That is deliberate: they are implementation-level decisions — ordering, overlap rules, wire formats — that need code in hand to settle well, not design uncertainty about the model. **They must be resolved before implementation begins**, per the process gate in `CLAUDE.md`. Anything implementation uncovers that contradicts this ADR gets a new ADR, not an edit to this one.

## Context

The 2026-07-28 Concept revision established Anchor as **capture-first, timeline-assisted** and brought timeline reconstruction into MVP scope (`docs/product/mvp.md`). That created four requirements the current event model cannot meet:

1. **Reconstruction needs transitions that carry explicit times.** Every existing `TransitionPayload` variant derives its Time Block boundaries from *when the transition was logged*. Only `RecoverGap` carries an explicit time (`inferred_end`), and it only closes an entry. Nothing can create or reshape a block with author-chosen start and end.
2. **`completion_reason` conflates three independent questions.** `auto-completed-on-skip` is the clearest symptom: it describes the fate of a stack frame, not a reason a block ended. Separately, `explicit` is written at six sites in `app/src-tauri/src/stack.rs` for four different meanings, while `docs/glossary.md` defined it as "user-finished."
3. **There is no legal way to stop tracking with an open interruption stack.** `Complete` is rejected when the stack is non-empty (`stack.rs`), so stopping mid-stack requires leaving the clock running or fabricating returns — both violate [`principles.md`](../principles.md) #3.
4. **A stack frame can only be resolved by returning to it**, so declining to return requires fabricating a resumption. Same principle, second instance.

[ADR 0004](0004-transition-log-format-and-torn-write-scheme.md) calls the on-disk JSONL schema a stable contract. This ADR **amends** it rather than editing it, per `CLAUDE.md`'s append-only rule.

## Options Considered

1. **Extend `CompletionReason` with new variants** (`SwitchedAway`, `Interrupted`, `Abandoned`) — smallest diff, no new fields.
2. **Three orthogonal fields, one question each** — `EndDetermination`, `CaptureOrigin`, `InterruptionOutcome`.
3. **Three fields plus a persisted `PauseSpan` entity and a persisted `Pending` state** — maximally explicit.

## Trade-offs

| | 1. Extend the enum | 2. Three fields | 3. Three fields + PauseSpan + Pending |
|---|---|---|---|
| Complexity | Lowest diff, highest ongoing confusion | One field per question | Second persisted entity type; transient state duplicated |
| Information preserved | Loses one axis whenever two apply (e.g. correcting a `recovered-gap` block) | All three axes independently | All three, plus intentional-vs-unknown gaps and pending-ness |
| Reversibility | Cheap now, compounding later — every future ADR inherits the conflation | Additive fields; values can be added later | Hard to remove an entity once exports and UI depend on it |
| Problem statement | — | Each field has one | `PauseSpan` had none at MVP scope |

## Decision

**Option 2.** Three orthogonal fields on `TimeBlock`, each answering exactly one question:

- **`EndDetermination`** — *how was this block's end time established?* `UserDetermined` | `SystemInferred`. Renamed from `CompletionReason`, which described neither completion nor a reason: a Switch-ended block's end time is exactly as user-determined as a completed one's.
- **`CaptureOrigin`** — *how did this block enter the system, and how much do we trust it?* Distinguishes live capture from after-the-fact reconstruction, and records whether a block has since been adjusted. Origin and adjusted-ness are preserved independently, so a manually entered block that received a one-second nudge stays distinguishable from a live capture that needed correcting.
- **`InterruptionOutcome`** — *what ultimately happened to this interrupted work?* Optional: **absent** means never interrupted (the common case); `Resumed`; `Skipped`. Absorbs `auto-completed-on-skip`.

**No persisted `Pending` value.** An unresolved obligation is represented by the interruption stack frame itself, which the snapshot persists. Persist stable facts, derive transient process state.

**Consequence — a canonical projection is mandatory, not optional.** Because `InterruptionOutcome` is absent both when a block was never interrupted *and* while it is interrupted-but-unresolved, a `TimeBlock` read in isolation is ambiguous. `stack.rs:98-113` pushes an interrupted block to `closed` with no outcome; only `resolve_paused` (`stack.rs:184`) fills it in. Every consumer therefore reads a **derived** status, and no consumer may implement its own:

> **`DerivedInterruptionStatus`** — not persisted. Computed from the `TimeBlock` plus the current interruption stack:
> - `InterruptionOutcome` is `Some(...)` → that outcome (`Resumed` / `Skipped`)
> - else the block's `id` appears as a `paused_time_block_id` in the current stack → `Pending`
> - else → `NeverInterrupted`

The History View, Timeline Editor, full-fidelity export, and any future diagnostics all consume this single projection. This keeps the persisted model minimal without leaving derived views free to disagree — the second-pass review found that without it, an interrupted-and-pending block is indistinguishable from a completed one in every derived view, which is the exact confusion R1 exists to prevent.

**No `Abandoned` value.** Explicitly dismissing a frame writes the existing `Skipped`, because the domain fact is identical — *this work was interrupted and never resumed*. The route is the event model's business, not the persisted state's ([`principles.md`](../principles.md) #6).

**New transitions:** reconstruction edits (add, move, resize, edit identity, delete), **Pause**, and frame dismissal.

**Pause is a specialised Interrupt: it creates an interruption frame without creating a successor task.** It closes the active Time Block, pushes its frame onto the interruption stack, and leaves `active == None`. Nothing is started.

This deliberately collapses "paused work" and "interrupted work" into one state — *work stopped with the intention of returning later* — because whether the reason was a phone call, lunch, or the end of the day does not change the lifecycle of the interrupted work. Consequences worth stating, all of which fell out of the second- and third-pass reviews:

- **The paused task keeps its return path.** It is simply the top frame, and Return to Previous lands on it. An earlier draft left Pause not pushing a frame, which silently lost "where was I" for the one task the user had just stopped.
- **Continue Session is not a transition.** It is a UI action only. After Pause the state is already correct, and after a restart replay reconstructs it from the persisted stack — so there is nothing to restore and nothing to materialise. An earlier draft listed it as a transition that changed no state, which would have had nothing to replay.
- **No second mechanism for preserving return intent.** Pause needs no persisted concept of its own; it reuses the interruption machinery entirely.

**`active == None` with a non-empty stack is a legal state.** The stack machine must therefore change:

- **`ReturnPrevious` and `ReturnOriginal` become valid with no active task**, provided at least one frame exists. They pop the frame, start the returned task, and resolve that frame's `InterruptionOutcome` — with no active block to close, since none exists. Today both fail on `self.active.take().ok_or(StackError::NoActiveTask)?` (`stack.rs:123`, `stack.rs:137`).
- **Without this change Pause does not close its hole — it moves it.** The user would have to start a task they did not do purely to make an unwinding transition legal: [`principles.md`](../principles.md) #3's failure mode, reintroduced by the change made to satisfy it.

**`Start` becomes a first-class action**, legal only when nothing is active. `TransitionPayload::Start` already exists in `model.rs` but no command exposes it; Start is currently reachable only implicitly through `switch`. `Switch` is **not** overloaded to mean Start when nothing is active — that would make one command describe two transitions depending on state. The vocabulary is five distinct intents with distinct preconditions:

| Action | Precondition | Meaning |
|---|---|---|
| `Start` | nothing active | Begin tracking |
| `Switch` | active block | Stop this, immediately begin something else |
| `Interrupt` | active block | Stop this, begin something else, intend to return |
| `Pause` | active block | Stop this, begin nothing, intend to return |
| `Return*` | ≥1 frame | Resume previously interrupted work |

The UI stays simple by being context-sensitive — Start where nothing is active, Switch where something is — without the domain model becoming state-dependent.

**Snapshot payload becomes a specified guarantee:** the snapshot MUST persist unresolved interruption stack frames, not only closed Time Blocks. ADR 0004 specified compaction's mechanism and never its payload; the no-`Pending` decision depends entirely on this, so it stops being an assumption.

**Explicitly rejected:** split and merge, for lacking problem statements ([`principles.md`](../principles.md) #1). Merge duplicates what `export.md` already does — grouping and summing equivalent work — while introducing adjacency rules, `duration ≠ end − start`, and provenance laundering on two axes.

## Consequences

**Makes easier.** Reconstructed work stops being indistinguishable from captured work, which makes `vision.md`'s Capture Rate metric computable and risk **R10** falsifiable rather than a matter of self-report. Interruption fragmentation becomes visible in the record permanently instead of evaporating with live state. R1's audit trail strengthens: `Skipped` now describes work rather than doubling as a navigation side effect.

**Makes harder.** Three fields instead of one is more to hold in mind, and every new transition must decide what each field means for the blocks it touches. Export must continue to ignore all three — `export.md`'s grouped output carries no per-block metadata, and none of these fields may become an aggregation key.

**Breaking change, taken deliberately now.** Moving `auto-completed-on-skip` out of the enum is not additive, against an ADR 0004 contract meant to be stable. Accepted because that contract currently protects exactly one log file, belonging to the person deciding, on an unreleased tool with no other users — this is the cheapest moment the change will ever be. It needs a replay shim or a discarded dev log, not a migration strategy.

**Unaffected.** [ADR 0001](0001-manual-assisted-tracking-for-mvp.md) holds: reconstruction is the user stating what happened, never activity inference. [ADR 0003](0003-billable-classification-out-of-scope.md) is untouched. ADR 0004's format, checksum framing, and compaction triggers are unchanged — only the payload guarantee is added.

## Open — must be resolved before implementation

This ADR does not decide these. They were identified but never worked through, and recording them as open is more honest than inventing answers:

1. **Log order vs. chronological order.** Replay is a sequential fold over `seq`-ordered lines, but a block entered at 16:00 for a 09:00 span is logged after blocks that happened later. Whatever replays the log must stop assuming append order implies timeline order.
2. **Overlap rules.** What happens when an added or resized block overlaps an existing one — reject, truncate the neighbour, or permit? Export sums durations per task, so permitting overlap silently inflates billed totals.
3. **Editing blocks bound to live stack frames.** Deleting or resizing a block whose task is currently on the stack, or dragging the active block's start, has no defined effect on replay.
4. **`Rename` versus edit identity.** `Rename` is defined as acting on the *currently active* block and is rejected otherwise (`interruption-stack.md`). Editing a historical block's name/project/client either extends that or creates a second naming path.
5. **Exact enum value names and serialised forms** for all three fields, including the kebab-case wire representation and the replay shim for existing `auto-completed-on-skip` lines.
6. ~~What rounding-off JSON export emits in place of `completion_reason`.~~ **Decided 2026-07-28** (moved out of open items): full-fidelity JSON carries **the three persisted fields**, and deliberately **not** `DerivedInterruptionStatus`. The projection is computed against the *current* interruption stack, so embedding it would make an export of last Tuesday produce different values depending on when it was run — unacceptable in the artifact that has to be reproducible for billing and analysis. It is a **display-surface** projection (History View, Timeline Editor, diagnostics); exports carry raw facts. Capture Rate needs only `CaptureOrigin` and is unaffected. This also resolves the apparent conflict with the glossary's "no view may read `InterruptionOutcome` directly": that rule governs surfaces that *display interruption state*, not an export serialising stored fields. Rationale — `docs/vision/vision.md` requires Capture Rate to be *computable*, and this is the only specified artifact that can carry the per-block metadata to compute it; adding an in-app analytics view was explicitly rejected as expanding MVP surface to validate the product rather than serve the user. Each export mode now has one job: **grouped output is intentionally lossy and billing-oriented; full-fidelity output is the analysis artifact.** Hard constraint unchanged: **no field here may become an aggregation key**, or export totals change silently (see R2). Remaining detail for implementation: exact key names and nesting.
7. **Two `export.md` defects with no other owner**, inherited here so they are not lost: (a) its Technical Constraints still describe "the underlying stored timeline (the append-only log...)" — the log/timeline conflation the source-of-truth correction existed to remove, and it cites ADR 0002 rather than 0004; (b) its acceptance criterion that the stored timeline is *"unchanged, byte-for-byte, after any export"* is **false whenever a task is active**, since the 60-second heartbeat appends to that file during the export. The existing test passes only because no heartbeat timer runs in it. The criterion needs restating as "no export writes to the timeline," which is what it meant.
8. **Restating `interruption-stack.md`'s acceptance criteria** against the new fields.
9. **`power.rs`'s auto-`Start` after `RecoverGap` on sleep/wake.** `app/src-tauri/src/power.rs` currently emits `RecoverGap` and then `Start` with the same identity at wake time — Anchor asserting both *that* work resumed and *when*. That is exactly what this ADR's Pause rationale refuses to claim, and [`principles.md`](../principles.md) #3's failure mode in shipped code. Three ways out: exempt it with a stated reason (the user was demonstrably at the machine), stop auto-starting and leave `active == None` with the frame on the stack (now a legal state, and consistent with Pause), or rename it so the two behaviours stop sharing a word. Not decided here. The asserted behaviour is unchanged; only the vocabulary is stale. Both docs are `accepted`, so this needs its own pass rather than an edit in passing — precisely the discipline R11 exists to enforce.
