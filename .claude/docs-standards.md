# Documentation Standards

## Frontmatter convention

YAML frontmatter (`status`, `date`, `owner`, `related`) is used on: ADRs, Vision, Concept, feature docs, and Risks. Plain Markdown (no frontmatter) elsewhere — glossary, research, ideas.

`status` values: `draft` → `proposed` → `accepted` (or `rejected` / `superseded`).

## ADR numbering

Sequential, starting at `0001` (`0000` is reserved for the template). Never renumber or reuse a number — a reversed decision gets a new ADR that marks the old one `superseded`, not a deletion.

## Definition of Ready

Implementation may begin only when all of the following exist and have cleared the Design/Architecture workflows: Vision, Product Concept, Target Users, MVP, Core Features, Architecture, Technology Decisions, UX Flows, Roadmap, and reviewed Open Questions.

## Graphify regeneration

`graphify-out/` is committed. Regenerate when a discovery/design/architecture phase completes, a batch of ADRs lands, or the glossary changes significantly — not on every commit. (Full cadence recommendation lands in Phase 9.)

## Templates and where they live

| Doc type | Path | Frontmatter | Created via |
|---|---|---|---|
| Vision | `docs/vision/vision.md` | yes | `/discovery-session` |
| Concept | `docs/concept/concept.md` | yes | `/discovery-session` |
| Target Users | `docs/product/users.md` | no | `/discovery-session` |
| MVP | `docs/product/mvp.md` | no | `/discovery-session` |
| Feature doc | `docs/product/features/<name>.md` | yes | `/new-feature <name>` (copy `_template.md`) |
| Architecture Overview | `docs/architecture/overview.md` | no | updated in place by Architecture workflow |
| Architecture Constraints | `docs/architecture/constraints.md` | no | updated in place by Architecture workflow |
| ADR | `docs/decisions/NNNN-title.md` | yes | `/new-adr <title>` (copy `0000-adr-template.md`) |
| Assumptions | `docs/assumptions.md` | no (log format) | updated in place, any workflow |
| Risks | `docs/risks.md` | yes (doc-level; per-row status inside) | updated in place, any workflow |
| Roadmap | `docs/roadmap.md` | no | updated in place by Planning workflow |
| Glossary | `docs/glossary.md` | no (table format) | updated via `grill-with-docs` / `/domain-modeling` |

Every template above already includes a "Keeping this current" note explaining when and how it should be revisited — that note is part of the template, not optional boilerplate to delete.

## Why frontmatter is scoped the way it is

Docs with frontmatter (Vision, Concept, feature docs, ADRs, Risks) are the ones where "what stage is this at, who owns it, what does it relate to" is a question worth answering in structured, queryable form — for Graphify and for a human scanning `docs/decisions/` for what's still `proposed`. Docs without frontmatter (users, mvp, architecture overview/constraints, assumptions, roadmap, glossary) are living/log-style documents that are always "current" by nature — a `status` field on a running log doesn't mean anything, so we didn't force one on. If a plain-Markdown doc ever needs to be tracked through a lifecycle, that's a signal to reconsider its shape, not to bolt frontmatter onto a log.
