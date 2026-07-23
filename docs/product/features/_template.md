---
status: draft
date:
owner:
related: []
---

# <Feature Name>

> Created via `/new-feature <name>`. Follow `.claude/workflows/design.md` — fill sections in order, don't skip to UX or Acceptance Criteria before Problem/Goals/Users/Alternatives/Trade-offs are settled. Run `grill-with-docs` on Alternatives/Trade-offs before moving `status` to `accepted`.

## Problem

What's broken or missing, for whom, evidenced how.

## Goals

What success looks like for this feature specifically, ideally measurable.

## Users

Which segment(s) from `docs/product/users.md` this serves. If none fit, that's a signal to revisit `users.md`, not invent a new segment here.

## Alternatives

At least two genuinely different approaches — not a strawman vs. the preferred one.

## Trade-offs

Explicit comparison: complexity, cost, UX impact, technical risk, reversibility.

## UX

Flows, states (empty/loading/error/success), interaction details. Owned by ux-designer.

## Technical Constraints

Feasibility, performance, data, integration constraints. Owned by technical-architect / senior-software-engineer. Link any resulting ADR from `docs/decisions/`.

## Acceptance Criteria

Concrete, testable conditions for "done."

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
