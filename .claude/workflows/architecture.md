---
name: architecture
description: Architecture decisions emerge from requirements, not preference. Multiple approaches are compared, trade-offs explained, one recommended, and the decision documented as an ADR.
status: validated
---

# Architecture Workflow

Runs once a feature or system-level need has cleared Design ([design.md](design.md)) with concrete Technical Constraints and Acceptance Criteria.

Validated end-to-end on ADR 0002 (desktop app framework/platform, 2026-07-23) — the stages, the two-pass review requirement, and the preference-vs-constraint guidance below reflect what actually happened, not a speculative design.

## Purpose

Whenever multiple technical approaches exist, compare them, explain trade-offs, recommend one, and document the decision — never decide from personal preference alone.

## Roles involved

- **technical-architect** — leads the comparison and recommendation
- **senior-software-engineer** — feasibility/effort reality-check on each option
- **reviewer** — reviews the recommendation (two-pass, see below) before it's accepted

## Is this decision ADR-worthy?

Write an ADR when a choice: forecloses or is expensive to reverse (a tech stack, a data format, a persistence model), was resolved by eliminating options rather than a trivial pick, or was explicitly deferred by a feature doc's Technical Constraints. Don't write one for a choice that's cheap to reverse and has no real alternative worth comparing (that's just an implementation detail, noted in the relevant doc, not an ADR).

## Stages

1. **Frame the decision** — state the requirement/constraint driving it, from the relevant feature doc's deferred Technical Constraints or `docs/architecture/constraints.md`.
2. **Ask at the real forks** — where the user's own constraints/priorities genuinely fork the option space (e.g. target platform, language/framework preference, a named priority ordering), ask directly rather than guessing. This is usually a small number of high-impact questions, same principle as Discovery.
3. **Generate options** — at least two genuinely viable approaches, not a strawman vs. the preferred one. Include the option(s) implied by the user's stated preference even if likely to be recommended — don't presuppose the answer by only comparing it against weak alternatives.
4. **Compare trade-offs** — cost, complexity, maintainability, performance, team/author familiarity, reversibility. Hedge any figure or maturity claim that isn't independently verified for this specific case ("commonly cited," "vendor-claimed," not asserted as bare fact) — per this project's own documentation-first rule.
5. **Recommend** — pick one, state why, state a concrete, observable reversal condition (a specific technical finding or time-box — not "if it turns out harder than expected").
6. **Preference vs. constraint** — if a stated preference is doing the work of eliminating options outright (not just tipping a close call), that's a signal it should be promoted to a row in `docs/architecture/constraints.md`, not left ambiguously described as both a preference and a constraint in the same document. Do this explicitly, in the ADR's Decision section, rather than letting the two framings quietly disagree.
7. **Document** — write the next sequential ADR in `docs/decisions/` using the [ADR template](../../docs/decisions/0000-adr-template.md); update `docs/architecture/overview.md` if the decision changes current-state architecture; update `docs/architecture/constraints.md` if step 6 applies; cross-link back to any feature doc whose Technical Constraints this ADR resolves, and update that feature doc's own text (not just its `related` list) if it was describing something as "not decided here."

## Review is two passes, not one

Same lesson as [design.md](design.md): a first reviewer pass reliably finds real must-fix issues (on ADR 0002: an unresolved deferred requirement dressed up as resolved, an unsupported factual claim, a preference/constraint inconsistency, a missing risk entry). Fixing those can leave something new unaddressed — budget for a second, verification-focused pass before flipping `status` to `accepted`, using the same reviewer-agent-with-full-context pattern as Design.

## Exit criteria

An accepted, sequentially-numbered ADR exists, cross-linked from the driving feature doc (which has itself been updated, not just linked-from), any newly-promoted constraint is recorded in `docs/architecture/constraints.md`, any risk the ADR itself surfaces is recorded in `docs/risks.md`, and a two-pass reviewer cycle found no remaining must-fix items.
