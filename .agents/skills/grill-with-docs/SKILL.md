---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also creates docs (ADRs and glossary) as we go.
disable-model-invocation: true
---

Run a `/grilling` session, using the `/domain-modeling` skill.

## Project-specific wiring

- Ground every question in this repo's current `docs/assumptions.md` and `docs/risks.md` — surface contradictions with what's already recorded, don't just interrogate in the abstract.
- Any term that comes out of the grilling session as load-bearing (used repeatedly, disputed, or redefined mid-session) gets added to `docs/glossary.md` via `/domain-modeling`.
- Any decision that survives the grilling gets written up as the next sequential ADR in `docs/decisions/` (see [ADR template](../../../docs/decisions/0000-adr-template.md)) — decisions that don't survive get logged in `docs/assumptions.md` as invalidated, not silently dropped.
- Invoke this before locking in Vision, Concept, or any ADR — per the [Discovery workflow](../workflows/discovery.md) and [Architecture workflow](../workflows/architecture.md).

Source: adapted from [mattpocock/skills grill-with-docs](https://github.com/mattpocock/skills/tree/main/skills/engineering/grill-with-docs).
