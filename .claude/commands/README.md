# Commands

Project-specific slash commands that drive the Discovery → Design → Architecture → Planning sequence:

- [/discovery-session](discovery-session.md) — run a Discovery workflow session
- [/new-feature](new-feature.md) `<feature-name>` — start a feature doc through the Design workflow funnel
- [/new-adr](new-adr.md) `<short-decision-title>` — record a new sequentially-numbered ADR via the Architecture workflow
- [/new-epic](new-epic.md) `<epic-name>` — group accepted feature docs/ADRs into a planned epic

Each composes the relevant `.claude/agents/*.md` roles and, where a decision needs stress-testing, the `grill-with-docs` skill.
