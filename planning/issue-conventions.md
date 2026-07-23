# Issue & GitHub Project Conventions

_Last updated: 2026-07-24_

## Labels

Two orthogonal dimensions, kept as labels (the "Epic" grouping itself lives in the Project board as a field, not a label — see below):

- `type:feature` — new functionality from an accepted feature doc
- `type:bug` — something built doesn't match its feature doc/ADR
- `type:chore` — non-functional work (tooling, refactor, doc sync)
- `type:research` — a spike to answer a question before committing to an approach (belongs in `prototypes/` too, per that folder's own convention)

## Issue naming

Imperative, concrete, no epic prefix needed (the Project board's Epic field covers that): `Implement append-only transition log writer`, not `[Interruption Stack] logging`.

## Issue body — required links

Every issue must link:
1. The feature doc section it implements (e.g. `docs/product/features/interruption-stack.md#technical-constraints`), and which Acceptance Criterion it satisfies.
2. Any ADR that constrains the implementation (e.g. `Implements: ADR 0004` for anything touching the transition log format).

An ADR never gets its own issue — it's a decision record, not a work item. The issues that implement a decision cite it; the ADR file itself is the traceability anchor other issues link back to.

## GitHub Project board

**Fields:**
- `Status` (single-select): Todo, In Progress, Blocked, Done
- `Epic` (single-select): Interruption Stack, Task Templates, Export — matches `planning/epics/*.md` exactly; add a new option only when a new epic file is created
- `Milestone` (GitHub's native milestone field): M1 — Core Tracking Loop, M2 — MVP Complete, matching `planning/milestones.md`

**Views:**
- Board, grouped by `Status` — the default day-to-day working view
- Table, grouped by `Epic` — for checking an epic's overall completeness against its feature doc
- Table, grouped by `Milestone` — for checking what's left before M1/M2 ship

## Keeping this current

When `planning/epics/` gains a new epic file, add the matching option to the Project board's `Epic` field in the same change — don't let the board's options drift from the actual epic files.
