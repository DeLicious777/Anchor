# Documentation

Living documentation for the project — kept in sync with every decision, not written once and forgotten.

- [`vision/vision.md`](vision/vision.md) — why this project exists, north star (frontmatter)
- [`concept/concept.md`](concept/concept.md) — product concept, elevator pitch (frontmatter)
- [`product/users.md`](product/users.md), [`product/mvp.md`](product/mvp.md) — target users, MVP scope
- `product/features/` — one doc per feature, copied from [`_template.md`](product/features/_template.md) via `/new-feature` (frontmatter)
- [`architecture/overview.md`](architecture/overview.md), [`architecture/constraints.md`](architecture/constraints.md) — current-state architecture and non-negotiables
- `decisions/` — ADRs, sequential and append-only, copied from [`0000-adr-template.md`](decisions/0000-adr-template.md) via `/new-adr` (frontmatter)
- [`assumptions.md`](assumptions.md) — running log of assumptions, revisited as confirmed or invalidated
- [`risks.md`](risks.md) — risk register (frontmatter)
- [`roadmap.md`](roadmap.md) — milestones and sequencing
- [`glossary.md`](glossary.md) — shared vocabulary, feeds Graphify

All templates are populated but empty — content lands via `/discovery-session`, `/new-feature`, and `/new-adr` (see `.claude/commands/`). See [Documentation Standards](../.claude/docs-standards.md) for the frontmatter convention and the Definition of Ready.
