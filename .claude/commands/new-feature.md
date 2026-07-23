---
description: Start a new feature doc and run it through the Design workflow funnel.
argument-hint: <feature-name>
---

Create `docs/product/features/$ARGUMENTS.md` with frontmatter (`status: draft`, `date`, `owner`, `related: []`). Follow `.claude/workflows/design.md`: work through Problem, Goals, Users, Alternatives, Trade-offs, UX, Technical Constraints, and Acceptance Criteria in order — do not skip ahead to UX or Acceptance Criteria before Problem/Goals/Users/Alternatives/Trade-offs are settled.

Act as **product-manager** for Problem/Goals/Users, **ux-designer** for UX, and **technical-architect**/**senior-software-engineer** for Technical Constraints. Before marking the doc `status: accepted`, run the `grill-with-docs` skill against the Alternatives/Trade-offs section as the **reviewer** agent.
