# Manual time block entry (drag a span on the timeline)

Split out of [`adjustable-timeline-view.md`](adjustable-timeline-view.md) on 2026-07-28. The original idea asked whether the timeline interacts with export; the answer was neither of the read-only options — the author wants to **drag a span on the timeline and add a Time Block covering it**.

That is not a visualization feature. It is the first way to write a Time Block into the record with a start and end that are *not* "whenever the transition was logged" — which makes it the largest of the four UI ideas by a wide margin, and the only one that touches an accepted ADR.

## Why this is bigger than it looks

Every existing transition derives its Time Block boundaries from *when it was logged*. `TransitionPayload` (`app/src-tauri/src/model.rs:100`) has `Start`, `Switch`, `Interrupt`, `Rename`, `ReturnPrevious`, `ReturnOriginal`, `Complete`, `Heartbeat`, `RecoverGap`. Only `RecoverGap` carries an explicit time at all (`inferred_end`), and even that only closes an entry — it never creates one.

So a dragged-in block needs a **new transition type carrying both an explicit start and an explicit end**. Consequences:

- **Log order stops implying chronological order.** ADR 0004's replay is a sequential fold over `seq`-ordered lines. A block dragged in at 16:00 for a 09:00 span is logged *after* blocks that happened later than it. Whatever replays the log has to stop assuming append order equals timeline order — that's a real change to the replay/stack machinery, not just a new enum variant.
- **ADR 0004 calls the on-disk JSONL schema a "stable on-disk contract"** and says changes must stay backward-compatible with existing logs or bring their own migration. Adding a variant is additive (old logs replay fine, like `Rename` was), but the ordering assumption above is the part that isn't purely additive.
- **Overlap rules are undefined.** What happens when a dragged span overlaps an existing block? Reject, truncate the neighbour, allow overlap? Export sums durations per task (`export.md`) — overlapping blocks would silently inflate a billed total. This needs an answer before anything is built.
- **Stack semantics don't apply.** A historical block isn't pushed onto the interruption stack; it never becomes active. The stack is a *live* structure. Adding to history must not disturb it.

## The adjacent gap this exposes

`docs/product/features/interruption-stack.md` (accepted) promises `recovered-gap` entries are "user-correctable" and that "correction happens whenever the user next opens the dashboard." Risk **R4**'s entire mitigation leans on that.

**There is currently no correction mechanism.** No transition type edits a completed Time Block's times, and `app/src-tauri/src/commands.rs` exposes no edit or delete command for one — only template CRUD. The accepted doc describes a capability that cannot be performed.

Correcting a `recovered-gap` end time and dragging in a forgotten block are the same underlying capability: **mutating the historical timeline after the fact.** They should be designed together and covered by one ADR, not solved twice.

## Why it's worth doing anyway

This is a direct mitigation for **R3** (med-high likelihood, high impact — "nothing prevents forgetting to log a Switch/Interrupt"), which currently has *no* MVP mitigation and is consciously deferred. Being able to draw in the meeting you forgot to track is the obvious answer to R3, and it stays fully consistent with ADR 0001: the user states what happened, nothing is inferred from activity.

## Still open

- Is this in MVP scope, or after? It isn't in `docs/product/mvp.md` today, and it's large enough that adding it silently would be scope creep on an MVP the docs already warn shouldn't only grow.
- Does the capability include **editing** and **deleting** existing blocks, or only **adding** new ones? Correcting `recovered-gap` needs editing, so "add only" leaves R4 unresolved.
- Append-only correction (a new transition that supersedes an earlier block, preserving history) vs. mutation? Append-only fits ADR 0004's grain and the ADR-supersede culture in this repo, but complicates replay further.
- Does a dragged-in block get a completion reason? None of `explicit` / `auto-completed-on-skip` / `recovered-gap` describes "the user drew this in later." Probably needs a fourth — which is a glossary and export-visible change.
- How is the new block named — inline entry, the same template/history autocomplete used for Rename, or the `Anchor N` auto-name fallback?
- Precision: what does a drag snap to (1/5/15 minutes)? Interacts with export's rounding interval.

## Promoted to MVP scope (2026-07-28)

The Concept revision (`docs/concept/concept.md`) made the timeline a **reconstruction workspace** and moved this into `docs/product/mvp.md`. Two things changed for this idea:

1. **It is no longer optional or deferred** — it's the mechanism behind `docs/vision/vision.md`'s revised "minimal manual effort, entirely inside Anchor" criterion, and it subsumes the `recovered-gap` correction path (R9).
2. ~~**The edit surface is larger than recorded above** — add, edit, remove, move, resize, **split, and merge**.~~ **Superseded the same day.** The `grill-with-docs` session **removed split and merge** for lacking problem statements. The MVP edit surface is exactly five operations: **add, move, resize, edit identity, delete** — see [ADR 0005](../docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md) and `docs/principles.md` #1. Merge turned out to duplicate what export already does (grouping and summing equivalent work) while adding adjacency rules, `duration ≠ end − start`, and provenance laundering on two axes; split was reachable via resize plus add. Struck through rather than deleted because the laundering concern it raised is what led to the three-field model.

One boundary is now fixed: **every block always represents work that actually happened.** No future-dated or intended blocks — planning was explicitly rejected as belonging in a calendar or task manager.

## Gate

This one should **not** be treated as a UI idea. It needs a full Design pass and its own ADR (new transition type carrying explicit start and end, log-order vs. chronological-order, overlap rules, split/merge semantics) before any implementation — see `.claude/workflows/design.md` and the process gate in `CLAUDE.md`. Being in MVP scope raises its priority; it does not skip the gate.
