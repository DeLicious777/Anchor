# Technical Constraints

_Last updated: 2026-07-23_

Non-negotiables and hard limits that every architecture decision (`.claude/workflows/architecture.md`) must respect. Distinct from preferences — a preference can lose to a better trade-off; a constraint here cannot.

## Constraints

| Constraint | Rationale | Source |
|---|---|---|
| **The event log is the single source of truth.** All state — the timeline, the interruption stack, every Time Block — is a projection replayed from the append-only transition log. Every input path produces transitions and nothing else: hotkeys, mini widget, Timeline Editor edits, and any future import. No component may hold durable tracked-work state outside the log, and nothing downstream needs to know which path produced a change. | Recorded as a foundational principle rather than an ADR (author's decision, 2026-07-28): it is not a decision between alternatives — it is the premise the existing ADRs already build on. [ADR 0004](../decisions/0004-transition-log-format-and-torn-write-scheme.md) decides *how* that source is persisted and replayed; the Event Model ADR decides *what* the events and resulting state mean. A separate ADR asserting the premise would be a meta-ADR everything else has to reference. **Scope:** tracked timeline state only — Task Templates (`docs/product/features/task-templates.md`), hotkey bindings, and export settings live in `settings.json` by design and are outside this constraint. | Concept revision + `grill-with-docs` session, 2026-07-28. Corrected a prior error: `concept.md`, `glossary.md`, `mvp.md`, and `vision.md` had all named the *timeline* as the source of truth, contradicting ADR 0004 and the implementation. |
| Frontend stack is Svelte + TypeScript, with Ramda for functional composition. | Formalized by ADR 0002 as a project-wide constraint, not a per-decision preference — this is a solo project where author familiarity and velocity matter more than benchmarking every option against a hypothetical native rewrite each time. Future ADRs should not re-litigate this; a change here should be its own ADR, not an implicit drift. | Author's stated stack preference (2026-07-23), promoted to a constraint in `docs/decisions/0002-desktop-app-framework-and-platform.md` after that ADR's review found "preference" and "constraint" language being used inconsistently to justify the same decision. |

Source is typically a stakeholder requirement, a compliance need, an existing commitment, or a hard technical limit (not "the architect prefers it").

---

**Keeping this current:** if an ADR's recommendation is only valid because of a constraint listed here, cite the row. If a constraint turns out to be a preference in disguise, move it out and let the trade-off be re-argued honestly.
