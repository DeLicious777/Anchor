---
name: design
description: Every feature moves through Problem -> Goals -> Users -> Alternatives -> Trade-offs -> UX -> Technical Constraints -> Acceptance Criteria before implementation.
status: validated
---

# Design Workflow

Runs after Discovery ([discovery.md](discovery.md)) has established vision/users/MVP, and before Architecture ([architecture.md](architecture.md)) commits to technology for a given feature.

Validated end-to-end on Anchor's `interruption-stack` feature (2026-07-23) — the process below, including the two-pass review requirement, reflects what actually caught real issues, not a speculative design.

## Purpose

No implementation before every stage below is complete for a feature, documented in that feature's file under `docs/product/features/`.

## Roles involved

- **product-manager** — Problem, Goals, Users
- **ux-designer** — UX stage, working flows/states
- **technical-architect** / **senior-software-engineer** — Technical Constraints
- **reviewer** — reviews Alternatives/Trade-offs (and everything downstream of them) before they're locked in

## Funnel stages

1. **Problem** — what's broken or missing, for whom, evidenced how. Often directly derivable from an already-accepted Vision/Concept without new questions — don't re-ask what's already answered.
2. **Goals** — what success looks like for this feature specifically, measurable if possible, tied explicitly back to a Vision/MVP success criterion.
3. **Users** — which segment(s) from `docs/product/users.md` this serves. Also usually derivable, not re-elicited.
4. **Alternatives** — at least two genuinely different approaches per decision. Ask the user directly at real forks (interaction model, data/persistence model, etc.) rather than guessing — these are exactly the questions worth spending the user's attention on, per the Discovery workflow's own principle of asking fewer, higher-impact questions.
5. **Trade-offs** — explicit comparison of the alternatives; not just pros of the chosen one.
6. **UX** — flows, states, interaction details (ux-designer owns this).
7. **Technical Constraints** — feasibility, performance, data, integration constraints. Watch specifically for a **decision hiding inside a descriptive phrase**: e.g. "inferred from the last heartbeat write" sounds like a fact, but "does a heartbeat exist, and at what interval" is an undecided design choice with its own real trade-off (accuracy vs. write frequency) that skipped stages 4-5. If a Technical Constraint asserts something no Alternative/Trade-off ever established, that's a gap, not a detail.
8. **Acceptance Criteria** — concrete, testable conditions for "done." Reject vague quantifiers ("negligible delay," "reasonable," "a couple of seconds") in favor of actual numbers — if a number can't be committed to yet, that's a sign the underlying Technical Constraint isn't actually decided either.

## Review is two passes, not one

An independent reviewer pass (Agent tool, `reviewer` subagent, self-contained prompt with full doc context) reliably finds real, must-fix issues — not just polish. But fixing those findings can introduce *new* undecided-decisions-hiding-in-phrases (see stage 7) or new inconsistencies elsewhere in the doc set. On Anchor's first run, the reviewer's second pass — verifying the fixes from the first — caught a genuine gap (the heartbeat mechanism) that the fixes themselves had introduced. Budget for both passes as standard, not as a contingency:

1. **First pass** — full review against Problem/Goals/Users/Alternatives/Trade-offs/UX/Technical Constraints/Acceptance Criteria, cross-checked against Vision/Concept/MVP/risks/assumptions for consistency.
2. **Remediate** — fix must-fix items concretely (edit the docs, populate `docs/risks.md`, amend or create ADRs as needed — see [architecture.md](architecture.md) for ADR mechanics).
3. **Second pass (verification)** — a fresh reviewer invocation, given both the original findings and a summary of what changed, explicitly asked to check whether each item was actually resolved (not just superficially addressed) and whether the fixes introduced anything new. If it finds a new must-fix (this happens — on Task Templates, fixing an overclaim led to force-fitting a decision into an evidence-based status field that didn't actually cover it), fix that too and run another verification pass. Only move `status` to `accepted` once a pass comes back clean — budget for "however many rounds it takes," not literally always exactly two.

## Exit criteria

All eight stages are filled in the feature doc, the doc is internally consistent with Vision/Concept/MVP and any related ADRs, a two-pass reviewer cycle found no remaining must-fix items, and Acceptance Criteria are concrete/numeric enough to plan against (see [planning.md](planning.md)).
