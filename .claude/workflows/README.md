# Workflows

Structured, repeatable processes governing how this project moves from idea to implementation.

1. [discovery.md](discovery.md) — interviews, challenges assumptions, documents answers. **Validated** (Phase 5) — run end-to-end on Anchor's own MVP discovery; question bank and stages reflect what actually worked.
2. [design.md](design.md) — Problem → Goals → Users → Alternatives → Trade-offs → UX → Constraints → Acceptance Criteria. **Validated** (Phase 6) — run end-to-end on the `interruption-stack` feature; the two-pass review requirement (first pass finds issues, second pass verifies the fixes) came directly out of that run.
3. [architecture.md](architecture.md) — compare approaches, recommend, document as ADR. **Validated** (Phase 7) — run end-to-end on ADR 0002 (desktop framework/platform); surfaced the preference-vs-constraint guidance and confirmed the two-pass review pattern from Design also applies here.
4. [planning.md](planning.md) — epics, milestones, GitHub issues. **Validated** (Phase 8) — 3 epics (one per accepted feature doc), 2 milestones by real dependency, and conventions written before any GitHub issue was created.

Sequence: Discovery → Design → Architecture → Planning → Implementation. No phase starts before the prior one's exit criteria are met.
