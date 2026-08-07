---
name: "source-command-discovery-session"
description: "Run a Discovery session — interview the user about the project, challenge assumptions, and document answers."
---

# source-command-discovery-session

Use this skill when the user asks to run the migrated source command `discovery-session`.

## Command Template

Follow `.Codex/workflows/discovery.md` end to end. Act as the **product-manager** agent for elicitation and the **researcher** agent for any claim needing evidence. Before writing anything to `docs/vision/`, `docs/concept/`, or `docs/product/users.md` as final, run the `grill-with-docs` skill against the emerging answers.

Do not proceed to solution design or technology choices during this session — that belongs to `/design-feature` and the Architecture workflow.
