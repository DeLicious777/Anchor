---
status: planned
date: 2026-08-07
owner: erich
related: [docs/product/features/timeline-reconstruction.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0006-stable-persistent-time-block-identity.md]
---

# Epic: Timeline Reconstruction

Work that happened but was never captured can be entered, and work recorded wrongly can be corrected — the only mitigation **R3** has, and the mechanism **R4** and **R9** were promised.

## Source docs

- Feature: [`timeline-reconstruction.md`](../../docs/product/features/timeline-reconstruction.md) — `accepted` 2026-08-01
- [ADR 0005](../../docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md), [ADR 0006](../../docs/decisions/0006-stable-persistent-time-block-identity.md)

## Current implementation state

**Substantially built. This epic must not be planned as greenfield.**

| Operation | Domain | Command | UI |
|---|---|---|---|
| Edit Identity | shipped | shipped | shipped — History View row action, 2026-08-02 |
| Delete | shipped | shipped | shipped — History View, with confirmation |
| **Resize** | shipped | shipped | **shipped — History View edit row, 2026-08-06, closing R9** |
| Move | shipped | shipped | **none — no caller** |
| Add | shipped | shipped | **none — no caller** |

All five transitions, all five commands and the whole validation surface ship: overlap rejection, `EndsInTheFuture`, `EndNotAfterStart`, the identity-only tier, the monotonic adjusted flag.

## Remaining work

1. **Add** and **Move** need an interaction surface. Both are spatial by nature, and `mvp.md` makes the Editor a hard prerequisite for exactly these two.
2. Collision **clamping** — the editor half of *"the editor clamps; the domain rejects"*, unbuilt because the editor is unbuilt.

## Dependencies and blockers

- **Implementation dependency on the Timeline Editor epic.** Nothing here is independently unblocked, because the domain is already finished.

## Child issues

- #15 — retitled and narrowed 2026-08-07 to **Add + Move + clamping**. Its previous title claimed all five operations, three of which already ship.

## Exit criteria

The `timeline-reconstruction.md` Acceptance Criteria not already met by shipped code — principally that a correctly working editor never triggers the domain's overlap rejection.

## Can it progress without the design-system assets?

**No.** Its remaining work is entirely Timeline Editor surface.
