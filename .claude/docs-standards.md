# Documentation Standards

## Frontmatter convention

YAML frontmatter (`status`, `date`, `owner`, `related`) is used on: ADRs, Vision, Concept, feature docs, and Risks. Plain Markdown (no frontmatter) elsewhere — glossary, research, ideas.

`status` values: `draft` → `proposed` → `accepted` (or `rejected` / `superseded`).

## ADR numbering

Sequential, starting at `0001` (`0000` is reserved for the template). Never renumber or reuse a number — a reversed decision gets a new ADR that marks the old one `superseded`, not a deletion.

## When *not* to write an ADR

_(Added 2026-08-02, author's guidance, at the point where the foundations are in place.)_

ADRs 0004, 0005 and 0006 established the architectural foundations — event sourcing, reconstruction semantics, persistent identity. **From here, new ADRs should become markedly rarer.** The remaining work (Timeline Editor, redesign implementation, reconstruction implementation) is mostly *implementing* existing decisions, not making new ones.

So before starting an ADR, ask:

> **Is this actually a new architectural decision, or is it an implementation detail that belongs in the feature design?**

The test is not "is this important" or "is this hard to reverse in code" — plenty of implementation choices are both. It is whether the decision **constrains work outside the feature that makes it**: a durable on-disk contract, a rule other features must obey, or a change to what an accepted ADR already decided. If the answer is "only this feature depends on it," it belongs in the feature doc's Technical Constraints, where it stays next to the design it serves.

Why this matters here specifically: an ADR set that absorbs implementation-level decisions stops being a map of the architecture and becomes a second, competing home for design detail — and the two drift. The existing set earns its place because each entry is referenced by documents that did not write it. Keep that property.

**A recent example of the boundary working correctly:** ADR 0006 deliberately did *not* decide whether reconstruction payloads carry the derived `Uuid` or the `seq`, and delegated it to whichever design specifies the payloads. `timeline-reconstruction.md` made the call in its own Alternatives. That is the split this section is asking for.

## Definition of Ready

Implementation may begin only when all of the following exist and have cleared the Design/Architecture workflows: Vision, Product Concept, Target Users, MVP, Core Features, Architecture, Technology Decisions, UX Flows, Roadmap, and reviewed Open Questions.

## Graphify regeneration

`graphify-out/` is committed. Validated cadence (Phase 9, run 2026-07-24 on the full doc set — 52 files, 141 nodes, 12 communities):

- **Full rebuild** (`/graphify .`) when a Discovery/Design/Architecture/Planning phase completes, or a batch of ADRs lands. Confirmed this is the right trigger, not an arbitrary one: fixing a single doc (`docs/architecture/overview.md`'s empty Key Decisions table) produced a genuinely new community, not just an edge update — community structure, not just node content, shifts at these boundaries.
- **Incremental `/graphify . --update`** for a single-doc fix — cache means unrelated files cost nothing. But cost scales with cross-reference density, not doc length: the initial 52-file build averaged ~5.2K tokens/file; re-extracting one doc after densifying its cross-references cost 47K tokens alone (~18% of the entire initial build). A "small" edit to a heavily-referenced doc (an ADR, Vision, Concept, MVP) is not necessarily a cheap regeneration.
- **Not on every commit** — full rebuilds are too costly relative to typical doc-editing velocity. Reserve them for the phase/ADR-batch cadence above; use `--update` in between if a graph refresh is needed sooner.
- Machine-specific state (`graphify-out/.graphify_python`, `.graphify_root`) is gitignored — not portable across machines, regenerated automatically on next run.

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
