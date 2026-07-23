---
description: Record a new architecture decision as the next sequentially-numbered ADR.
argument-hint: <short-decision-title>
---

Follow `.claude/workflows/architecture.md`. Determine the next sequential number by listing existing files in `docs/decisions/`, then create `docs/decisions/NNNN-$ARGUMENTS.md` from the [ADR template](../../docs/decisions/0000-adr-template.md).

Act as **technical-architect**: frame the decision, generate at least two genuinely viable options, compare trade-offs explicitly, and recommend one. Before marking the ADR `status: accepted`, run the `grill-with-docs` skill against the recommendation as the **reviewer** agent. Cross-link the ADR from any feature doc that drove it, and mark any prior ADR it supersedes.
