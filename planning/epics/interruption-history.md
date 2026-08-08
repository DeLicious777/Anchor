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
4. "Earlier" — flat, most-recent-first, membership by `start`. **Its backend range store landed in #25; the control and filtering ship with this M3c surface.**

## Dependencies and blockers

- **Item 1 landed in #24 and ships no user-reachable behaviour** — it is an *enabling* slice. Until item 3 gives it an entry point, dismissal is a command with no caller, so `BlockReferencedByOpenFrame`'s advice stays false and **this epic does not close on item 1**. *(Recorded 2026-08-07: an earlier draft of this epic claimed item 1 would make that message true. It does not, and shipping item 1 has now demonstrated that.)*
- Items 2–3: external — the design-system assets.
- Item 4: the backend dependency landed in #25. Its range control and filtering remain surface work and ship alongside the panel in M3c.

## Child issues

- **#24 — frame dismissal, transition and command. Implemented 2026-08-07**; item 1 above. **What it does satisfy:** the replay-and-snapshot durability criterion, and the *domain* portions of exact-frame dismissal and order preservation — a dismissal resolves precisely the named frame as `Skipped`, leaves the remaining frames' order intact, leaves `active` untouched, and survives replay and a snapshot round-trip identically. **What it does not:** any user-facing exit criterion, all of which require a caller. It therefore closes neither this epic nor the feature.
- **#25 — shared view range backend. Implemented 2026-08-07;** item 4's store dependency is satisfied, with the range control and filtering deliberately deferred to M3c.
- #17 — the epic issue, retitled 2026-08-07. Retains the asset-blocked panel work.

## Exit criteria

`interruption-history.md`'s Acceptance Criteria, notably: no frame list in the default view at any depth; the skipped count the preview states matches what Return to Original actually writes; dismissing the frame that blocks a restricted block makes it reshapeable with no restart; every action `BlockReferencedByOpenFrame` names is reachable from the running app.

## Can it progress without the design-system assets?

**No — not any longer.** Item 1 landed in #24 and item 4's backend dependency landed in #25. Items 2 through 4 are now surface work and wait on the design-system assets.
