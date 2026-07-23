# Technical Constraints

_Last updated: 2026-07-23_

Non-negotiables and hard limits that every architecture decision (`.claude/workflows/architecture.md`) must respect. Distinct from preferences — a preference can lose to a better trade-off; a constraint here cannot.

## Constraints

| Constraint | Rationale | Source |
|---|---|---|
| Frontend stack is Svelte + TypeScript, with Ramda for functional composition. | Formalized by ADR 0002 as a project-wide constraint, not a per-decision preference — this is a solo project where author familiarity and velocity matter more than benchmarking every option against a hypothetical native rewrite each time. Future ADRs should not re-litigate this; a change here should be its own ADR, not an implicit drift. | Author's stated stack preference (2026-07-23), promoted to a constraint in `docs/decisions/0002-desktop-app-framework-and-platform.md` after that ADR's review found "preference" and "constraint" language being used inconsistently to justify the same decision. |

Source is typically a stakeholder requirement, a compliance need, an existing commitment, or a hard technical limit (not "the architect prefers it").

---

**Keeping this current:** if an ADR's recommendation is only valid because of a constraint listed here, cite the row. If a constraint turns out to be a preference in disguise, move it out and let the trade-off be re-argued honestly.
