---
status: accepted
date: 2026-07-28
owner: erich
related: [docs/decisions/0004-transition-log-format-and-torn-write-scheme.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/concept/concept.md, docs/vision/vision.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/glossary.md, docs/principles.md, docs/risks.md, docs/architecture/constraints.md]
---

# 0005: Event Model — Time Block Metadata and Reconstruction Transitions

> **Accepted 2026-07-28**, together with `vision.md` and `concept.md`. The decisions below came out of a `grill-with-docs` session (14 branches) and survived three independent review passes returning 15, 14, and 12 findings respectively — each round's fixes introduced new problems the next round caught, which is why three ran. Every reported blocker was addressed and the author reviewed the full diff.
>
> **All nine open items are now resolved** (2026-08-01). Items 5–9 were settled during implementation and follow-up passes; items 1–4 by [`timeline-reconstruction.md`](../product/features/timeline-reconstruction.md)'s design pass, which also required a new decision this ADR did not anticipate — persistent Time Block identity, [ADR 0006](0006-stable-persistent-time-block-identity.md). The process gate below is therefore met for the model; implementation of reconstruction remains gated on risk **R14** and on #14/#8, which are not model questions. Original note follows.
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

## Amendment (2026-07-29): the "no active task" state was already reachable

Recorded as an amendment rather than an edit to Context above, per `CLAUDE.md`'s append-only rule and the precedent ADR 0001 set. Nothing in the Decision changes; its **justification** does, and gets stronger.

**Architectural guarantee, stated explicitly:** `active == None` with a non-empty interruption stack is an **accepted, supported application state**. It may arise from Pause, from crash recovery, or from future workflows. The state machine MUST provide legal transitions out of it without requiring a synthetic task start.

**Why this is an amendment and not a restatement.** The Decision above justified that legality as something *Pause needs*. Checking the claim against the implementation showed the state **already occurs today**: `state.rs`'s `AppState::init` appends `RecoverGap` when replay leaves an entry active, and `RecoverGap` deliberately does not auto-resume — so a crash inside an interruption produces it on the next launch, with frames intact. In that state `ReturnPrevious`, `ReturnOriginal`, `Interrupt` and `Rename` all fail on `NoActiveTask`, and `Complete` fails on `CannotCompleteWithOpenStack`. The only escape is `commands.rs`'s `switch`, which dispatches to `Start` when nothing is active.

So the user must already begin a task they may not be doing in order to unwind orphaned frames — [`principles.md`](../principles.md) #3's failure mode, live in shipped code, not a hazard introduced by Pause.

**Consequences:**

- **The stack-state work is a correctness fix in the core interruption model**, not scaffolding for a feature. It would remain necessary if Pause were removed. Tracked on issue #1 with a regression test: crash inside an interruption, restart, unwind without starting anything.
- **The `Switch`-as-`Start` overload this ADR rejected already exists.** `switch` branches on `active.is_none()`. Making `Start` first-class is therefore partly an extraction of existing behaviour, not new construction — cheaper than the Decision above implies.
- **The narrative inverts, usefully:** Pause did not require a new state; Pause *exposed an incomplete state machine.* That is a better foundation, because it does not depend on wanting Pause.

Found by applying [`principles.md`](../principles.md) #8 — verifying an accepted claim against the implementation rather than against the document that asserted it.

## Amendment (2026-07-29): the metadata split is not an on-disk change

**This ADR made two claims about the three-field migration that are false.** Both were caught by checking against the implementation before doing the work ([`principles.md`](../principles.md) #8), and both made the change look far more expensive than it is.

**Claim 1 — "Breaking change, taken deliberately now."** The Consequences section says moving `auto-completed-on-skip` out of the enum "is not additive, against an ADR 0004 contract meant to be stable." **It isn't against that contract at all.** `completion_reason` never reached the log. `TransitionRecord` carries `seq`, `timestamp`, and a `TransitionPayload` — nothing else — and `TimeBlock` appears nowhere in the `log` module. ADR 0004's own doc comment says so explicitly: *"resolved state (which blocks close, which frames get pushed/popped) is derived by the state machine itself, not duplicated into the log."*

The metadata is **pure derived state**, recomputed by `InterruptionStack::apply` on every replay. It is serialised in exactly two places, both write-only: the JSON export, and the IPC payload to the frontend. Neither is ever read back.

**Claim 2 — open item 5's "replay shim for existing `auto-completed-on-skip` lines."** There are no such lines, and never were. No shim was needed or written.

**What this changes.** The migration required no on-disk migration, no compatibility window, and no reason to hurry it while the userbase is one person. ADR 0004's stable-contract guarantee is untouched by it and remains fully intact. The one place this *would* have bitten is the snapshot, which does serialise resolved state — but compaction is unimplemented, so no snapshot exists to migrate. Whoever builds it (#8) will serialise whatever the model is then.

**Kept rather than corrected in place**, per the append-only rule: the original Consequences text stands, and this records that its cost assessment was overstated. The decision it justified was right anyway — for reasons of model clarity, not migration economics.

**Resolution of open item 5**, now that it is a pure model question:

| Field | Wire values (kebab-case) |
|---|---|
| `EndDetermination` | `user-determined`, `system-inferred` |
| `CaptureOrigin` | `live-capture`, `live-capture-adjusted`, `manual-entry`, `manual-entry-adjusted` |
| `InterruptionOutcome` | `resumed`, `skipped` (absent when never interrupted *or* unresolved) |
| `DerivedInterruptionStatus` | `never-interrupted`, `pending`, `resumed`, `skipped` — projection only, never persisted |

`CaptureOrigin` is a flat four-variant enum rather than nested origin/adjusted fields, so the serialised form stays a single string; `origin()` and `is_adjusted()` recover the axes, and `adjusted()` is idempotent and never rewrites origin.

## Resolution of open item 9 (2026-07-29): wake stops auto-starting

> **SUPERSEDED by [ADR 0007](0007-auto-resume-after-a-short-gap.md) (2026-08-03).** A gap shorter than one hour now closes the block *and* restarts the same work; only longer gaps resolve as decided below. The reasoning here was not found to be wrong — the author re-weighed the trade it names, and the "an accurate record over a convenient one" cost was accepted within a bound rather than rejected.
>
> **One claim below turned out to be incomplete**, found while writing ADR 0007. It says handling wake and crash differently *"was the anomaly, and this removes it."* It removed the auto-start half; `power.rs` kept a 90-second grace window that `AppState::init` never had, so a brief sleep-wake stayed a non-event while a brief crash-relaunch closed the block and — with no heartbeat yet landed — produced a zero-duration one. Both paths now share `crate::gap`.
>
> Text retained unedited below. ADRs are append-only.

**Decision: on sleep/wake, `power.rs` emits `RecoverGap` and nothing else.** The active entry is closed with an inferred end; no new Time Block is started. The user resumes deliberately, via the capture action.

**The deciding argument is consistency, which none of the three original options named.** `state::AppState::init` — crash recovery — already does exactly this: `RecoverGap`, no auto-resume. Wake and crash are the same class of event: *Anchor lost continuity and cannot know what happened in the gap.* Handling them differently was the anomaly, and this removes it rather than adding a rule.

The asymmetry had a stated justification in `power.rs`'s module doc — that wake is "the SAME running process," so the task identity is still known. That is true, and it is beside the point: knowing *which task* was active is not knowing *that the user resumed it*, nor *when*. Someone who wakes a laptop to check the time has not gone back to work.

**Why not the other two options:**

- **Exempt it** ("the user was demonstrably at the machine") — presence is not resumption. It proves someone dismissed a lock screen, nothing about which task they intend to work on. Anchor would still be inventing a start time, which is [`principles.md`](../principles.md) #3 exactly.
- **Rename it** — that is a documentation fix for a behaviour problem. The false claim would still be written to the log; it would just be labelled more carefully.

**Cost, stated plainly:** the user loses auto-resume convenience. After a wake they press the capture hotkey instead of finding tracking already running. That is the intended trade — an accurate record over a convenient one.

**One genuine gap this does *not* close, deliberately.** `RecoverGap` closes the active block without pushing a stack frame, so neither wake nor crash preserves a return path for the task that was interrupted by the gap. Its identity survives only as a closed Time Block. That is pre-existing, symmetric across both paths, and fixing it means changing what `RecoverGap` does — which touches [ADR 0004](0004-transition-log-format-and-torn-write-scheme.md)'s contract and belongs with Pause's design work (issue #16), not here. Rename's autocomplete over past task history is the current mitigation.

**Implementation:** `resolve_resume_gap` returns a single `Option<TransitionPayload>` rather than an ordered pair, so the auto-resume cannot be reintroduced by accident — the type no longer has room for it.

## Open — must be resolved before implementation

This ADR does not decide these. They were identified but never worked through, and recording them as open is more honest than inventing answers:

1. ~~**Log order vs. chronological order.**~~ **Resolved 2026-08-01** — [`timeline-reconstruction.md`](../product/features/timeline-reconstruction.md), alternative A: two orderings, each authoritative in its own domain. Log order (`seq`) stays authoritative for replay; block `start` governs display and export. ADR 0004 is untouched. One latent defect fell out of it — full-fidelity JSON emitted in close order, which coincides with start order only until a block is inserted — now carried as a requirement in `export.md`.
2. ~~**Overlap rules.**~~ **Resolved 2026-08-01** — alternative B: collision clamping. No overlap is ever persisted and the user is never told "no"; a gesture stops at the neighbouring boundary and neighbours are never pushed, truncated, or rewritten. Deliberately paired with domain-level rejection, so the invariant does not depend on the editor being correct.
3. ~~**Editing blocks bound to live stack frames.**~~ **Resolved 2026-08-01** — alternative C: three tiers by block state, on the rule that a block whose span is not yet fixed is not a reconstruction target. This item's second half — *dragging the active block's start* — is answered explicitly rather than by implication: forbidden, with Add (clamped at the active block's start) as the path, which keeps the reconstructed minutes labelled `ManualEntry` instead of relabelling them as live capture.
4. ~~**`Rename` versus edit identity.**~~ **Resolved 2026-08-01** — alternative D: two transitions, shared implementation. They produce identical state but answer different questions about how it came to be, and merging them would supersede a shipped, accepted transition and `interruption-stack.md`'s "requires an active task" rule. `Edit Identity` on a block with an open frame must propagate to the frame's copy of the identity.
5. ~~**Exact enum value names and serialised forms** for all three fields, including the kebab-case wire representation and the replay shim for existing `auto-completed-on-skip` lines.~~ **Resolved 2026-07-29 — and this item's premise was wrong.** See "Amendment: the metadata split is not an on-disk change" below.
6. ~~What rounding-off JSON export emits in place of `completion_reason`.~~ **Decided 2026-07-28** (moved out of open items): full-fidelity JSON carries **the three persisted fields**, and deliberately **not** `DerivedInterruptionStatus`. The projection is computed against the *current* interruption stack, so embedding it would make an export of last Tuesday produce different values depending on when it was run — unacceptable in the artifact that has to be reproducible for billing and analysis. It is a **display-surface** projection (History View, Timeline Editor, diagnostics); exports carry raw facts. Capture Rate needs only `CaptureOrigin` and is unaffected. This also resolves the apparent conflict with the glossary's "no view may read `InterruptionOutcome` directly": that rule governs surfaces that *display interruption state*, not an export serialising stored fields. Rationale — `docs/vision/vision.md` requires Capture Rate to be *computable*, and this is the only specified artifact that can carry the per-block metadata to compute it; adding an in-app analytics view was explicitly rejected as expanding MVP surface to validate the product rather than serve the user. Each export mode now has one job: **grouped output is intentionally lossy and billing-oriented; full-fidelity output is the analysis artifact.** Hard constraint unchanged: **no field here may become an aggregation key**, or export totals change silently (see R2). Remaining detail for implementation: exact key names and nesting.
7. ~~**Two `export.md` defects with no other owner.**~~ **Resolved 2026-07-29.** (a) The log/timeline conflation is corrected — the doc no longer calls the timeline "the append-only log," and the citation moved from ADR 0002 to ADR 0004. (b) The acceptance criterion "unchanged, byte-for-byte, after any export" is replaced with the invariant export can actually guarantee: **export itself performs no writes**. The old wording was false because the 60-second heartbeat may legitimately append to the log mid-export; the existing test passed only because no heartbeat timer runs in it. Terminology through the rest of the doc migrated to the three fields at the same time.
8. ~~**Restating `interruption-stack.md`'s acceptance criteria** against the new fields.~~ **Resolved 2026-07-29.** Terminology migrated throughout the normative sections (Goals, UX, Technical Constraints, Acceptance Criteria), with a mapping table added to the header — note the old value `explicit` mapped to **two** of the new fields depending on which block carried it, which is precisely why the split happened. The one criterion that conflated all three responsibilities was split so each asserts a single one. **Alternatives and Trade-offs were deliberately left in their original wording**: they record what was decided in July 2026 and with what reasoning, and rewriting them would falsify a decision record.
9. ~~**`power.rs`'s auto-`Start` after `RecoverGap` on sleep/wake.**~~ **Resolved 2026-07-29 — stop auto-starting.** See "Resolution of open item 9" below. The asserted behaviour is unchanged; only the vocabulary is stale. Both docs are `accepted`, so this needs its own pass rather than an edit in passing — precisely the discipline R11 exists to enforce.
