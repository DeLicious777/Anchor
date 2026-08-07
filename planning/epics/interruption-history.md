---
status: planned
date: 2026-08-07
owner: erich
related: [docs/product/features/interruption-history.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md]
---

# Epic: Interruption History

The interruption stack becomes inspectable and resolvable: progressive disclosure of open frames, a preview of what each return will do, and the only surface from which a frame can be dismissed.

## Source docs

- Feature: [`interruption-history.md`](../../docs/product/features/interruption-history.md) — `accepted` 2026-08-07
- [ADR 0005](../../docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md) — dismissal writes `Skipped`; there is no `Abandoned` value

## Current implementation state

**The domain half is built; no surface is.** The shipped behaviour is still the inverse of the accepted design: `app/src/routes/+page.svelte` renders the entire stack **unconditionally** as a flat list of names — no timings, no outcome, no actions — while the accepted design says the default view shows only the current task and the two return options.

**Frame dismissal's transition and command landed in #24** (2026-08-07): `TransitionPayload::DismissFrame` plus the `dismiss_frame` command, keyed on the paused block's id, with replay and snapshot coverage. **Nothing calls them.** `StackError::BlockReferencedByOpenFrame` still tells the user to *"resume or dismiss"* an interruption they have no way to dismiss, so **R11**'s eighth instance stays open until this panel ships.

## Remaining work

1. ~~A frame-dismissal transition and command.~~ **Done in #24** (2026-08-07) — resolves the named frame's block to `Skipped` through the same `resolve_paused` path the returns use, removes that frame, leaves `active` untouched, keyed on the paused block's id. Ships no **user-reachable** behaviour on its own.
2. Replace the always-visible list with a labelled disclosure carrying depth. *(Asset-blocked.)*
3. "Now" — frames deepest-first, paused-at and waiting-for times, the return preview, per-frame Dismiss with inline confirmation. *(Asset-blocked.)*
4. "Earlier" — flat, most-recent-first, membership by `start`. **Depends on the shared view range.**

## Dependencies and blockers

- **Item 1 landed in #24 and ships no user-reachable behaviour** — it is an *enabling* slice. Until item 3 gives it an entry point, dismissal is a command with no caller, so `BlockReferencedByOpenFrame`'s advice stays false and **this epic does not close on item 1**. *(Recorded 2026-08-07: an earlier draft of this epic claimed item 1 would make that message true. It does not, and shipping item 1 has now demonstrated that.)*
- Items 2–3: external — the design-system assets.
- Item 4: implementation dependency — the shared view range, in the Timeline Editor epic.

## Child issues

- **#24 — frame dismissal, transition and command. Implemented 2026-08-07**; item 1 above. **What it does satisfy:** the replay-and-snapshot durability criterion, and the *domain* portions of exact-frame dismissal and order preservation — a dismissal resolves precisely the named frame as `Skipped`, leaves the remaining frames' order intact, leaves `active` untouched, and survives replay and a snapshot round-trip identically. **What it does not:** any user-facing exit criterion, all of which require a caller. It therefore closes neither this epic nor the feature.
- #25 — the shared view range, which item 4 depends on.
- #17 — the epic issue, retitled 2026-08-07. Retains the asset-blocked panel work.

## Exit criteria

`interruption-history.md`'s Acceptance Criteria, notably: no frame list in the default view at any depth; the skipped count the preview states matches what Return to Original actually writes; dismissing the frame that blocks a restricted block makes it reshapeable with no restart; every action `BlockReferencedByOpenFrame` names is reachable from the running app.

## Can it progress without the design-system assets?

**No — not any longer.** Item 1 was this epic's only asset-independent work and it landed in #24 (2026-08-07). Items 2 and 3 are the panel itself and wait on the design-system assets; item 4 waits on those **and additionally** on the shared view range (#25, Timeline Editor epic), so it is the one piece with two separate blockers.
