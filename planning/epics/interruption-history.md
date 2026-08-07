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

**Not started, and the shipped behaviour is the inverse of the accepted design.** `app/src/routes/+page.svelte` renders the entire stack **unconditionally** as a flat list of names — no timings, no outcome, no actions — while the accepted design says the default view shows only the current task and the two return options.

Frame dismissal does not exist: no `TransitionPayload` variant, no command. `StackError::BlockReferencedByOpenFrame` nevertheless tells the user to *"resume or dismiss"* — a live defect, logged as **R11**'s eighth instance.

## Remaining work

1. A frame-dismissal transition and command — resolve the named frame's block to `Skipped` through the same `resolve_paused` path the returns use, remove that frame, leave `active` untouched. **Keyed on the paused block's id, never a stack index.**
2. Replace the always-visible list with a labelled disclosure carrying depth. *(Asset-blocked.)*
3. "Now" — frames deepest-first, paused-at and waiting-for times, the return preview, per-frame Dismiss with inline confirmation. *(Asset-blocked.)*
4. "Earlier" — flat, most-recent-first, membership by `start`. **Depends on the shared view range.**

## Dependencies and blockers

- **Item 1 is unblocked today, but it is an *enabling* slice and ships no behaviour.** Until item 3 gives it an entry point, dismissal is a command with no caller — so `BlockReferencedByOpenFrame`'s advice stays false, and this epic cannot close on item 1 alone. *(Corrected 2026-08-07: the first draft of this epic claimed item 1 made that message true. It does not.)*
- Items 2–3: external — the design-system assets.
- Item 4: implementation dependency — the shared view range, in the Timeline Editor epic.

## Child issues

- **#24 — frame dismissal, transition and command.** Unblocked; item 1 above.
- #25 — the shared view range, which item 4 depends on.
- #17 — the epic issue, retitled 2026-08-07. Retains the asset-blocked panel work.

## Exit criteria

`interruption-history.md`'s Acceptance Criteria, notably: no frame list in the default view at any depth; the skipped count the preview states matches what Return to Original actually writes; dismissing the frame that blocks a restricted block makes it reshapeable with no restart; every action `BlockReferencedByOpenFrame` names is reachable from the running app.

## Can it progress without the design-system assets?

**Partly — item 1 only.** The panel itself cannot.
