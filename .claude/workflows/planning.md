---
name: planning
description: Bridges completed Design/Architecture work into executable epics, features, milestones, and GitHub issues.
status: validated
---

# Planning Workflow

Runs once a feature has cleared Design ([design.md](design.md)) and any necessary ADRs from Architecture ([architecture.md](architecture.md)) are accepted.

Validated end-to-end on Anchor's MVP planning (2026-07-24) — one epic per accepted feature doc, sequenced into milestones by actual dependency, backed by a real GitHub Project.

## Purpose

Turn accepted feature docs and ADRs into structured, sequenced, trackable work — not before.

## Roles involved

- **product-manager** — epic/milestone sequencing and prioritization
- **technical-architect** — flags cross-cutting technical dependencies between epics
- **documentation-steward** — ensures planning artifacts stay cross-linked to the docs that justify them

## Stages

1. **Epic definition** — one epic file per accepted feature doc under `planning/epics/`, linking to that feature doc and any ADRs it depends on. Each epic states its scope (derived from the feature doc's Acceptance Criteria), what's explicitly excluded, and its dependencies on other epics.
2. **Milestone sequencing** — group epics into milestones on `planning/milestones.md` by actual dependency (what blocks what), not by arbitrary priority. Epics with no dependency on each other belong in the same milestone even if built sequentially in practice.
3. **Conventions before issues** — before creating any GitHub issue, `planning/issue-conventions.md` must exist: labels, naming, required links (every issue cites the feature doc section/AC it implements and any constraining ADR — an ADR never gets its own issue, it's a decision record), and the Project board's fields/views.
4. **GitHub Project setup** — before creating anything on GitHub, confirm scope with the user explicitly (repo visibility, what gets created) since this is publishing to a shared/public surface, not a local file change. Project fields: Status, Epic (matching `planning/epics/*.md` exactly), Milestone (matching `planning/milestones.md`).
5. **Issue breakdown** — decompose each epic into issues following the conventions above, only after stages 1-4 are settled.

## Exit criteria

Every epic in `planning/epics/` links back to accepted feature docs and ADRs, has a place on `planning/milestones.md`, and (if GitHub tracking is in use) a matching Project board field option exists before its issues are created.
