---
name: discovery
description: Structured interview process that surfaces vision, users, and constraints before any solution design begins.
status: validated
---

# Discovery Workflow

Runs before Design (see [design.md](design.md)). Do not begin solution design until discovery reaches sufficient maturity — see Definition of Ready in `docs/README.md`.

Validated end-to-end on Anchor's own MVP discovery (2026-07-23) — the stages and question bank below reflect what actually worked, not a speculative design.

## Purpose

Interview the user about the project, challenge assumptions, and document answers — rather than jumping to technical decisions.

## Roles involved

- **product-manager** — leads on vision, users, MVP scope questions
- **researcher** — investigates claims that need evidence before being accepted
- **reviewer** — runs a formal, independent grilling pass before Vision/Concept/MVP move to `status: accepted` (see Stage 4)

## Stages

### 1. Elicit

Ask the fewest high-impact questions that remove ambiguity — per global rules, prefer 2-4 questions that fork the whole direction over a long checklist. Question bank, roughly in priority order:

- **Problem** — what's broken/missing, for whom, evidenced how (concrete moment of friction, not aspiration)
- **Why now / why you** — personal itch, professional gap, market shift, new technical capability — this usually reveals the real constraint set later
- **Users** — the single narrowest group who'd feel the most relief, not "everyone eventually"
- **Scope** — personal tool vs. product for others, decided explicitly and early; it silently shapes every later answer (accounts, sync, distribution) if left unstated

Prefer plain prose questions over the `AskUserQuestion` tool for this exploratory ground-laying stage — forcing early, wide-open answers into multiple-choice options tends to lose nuance the user would otherwise volunteer. Reserve `AskUserQuestion` for later, genuinely enumerable forks (e.g. "which of these 3 architectures").

### 2. Deepen

Once the shape of the idea exists, ask a second round targeting the mechanics specifically — this is where hidden complexity actually lives:

- Are there implicit nested/recursive cases the first answer didn't address? (e.g. "what if this happens again while you're already handling it")
- Does the stated method actually deliver the stated goal, or is there tension between them? (e.g. "manual only" vs. "nothing gets forgotten")
- What's the minimum data/state model implied by the answers so far, and does it support the reporting/output already asked for?

Ask these as a single batch of 3-5 concrete, cited questions (referencing the user's own prior words) rather than drip-feeding one at a time.

### 3. Challenge (inline)

Before recording anything as final, actively look for — and say out loud, citing the specific answer — contradictions between stated goals and stated mechanics, ambiguous terms that need a real definition, and edge cases the user's answer didn't cover. This is lighter-weight than Stage 4's formal review; it's the same skepticism applied conversationally, in real time, so the Record stage starts from already-tightened answers rather than raw first-draft ones.

### 4. Record

Write/update `docs/vision/vision.md`, `docs/concept/concept.md`, `docs/product/users.md`, `docs/product/mvp.md` as `status: draft` (where frontmatter applies). Log every assumption to `docs/assumptions.md` and every load-bearing term to `docs/glossary.md` as they come up — don't batch this for later.

### 5. Formal review

Invoke the **reviewer** agent (via the Agent tool) with a self-contained prompt: full context on the product, explicit instruction to check cross-doc consistency (Vision vs. Concept vs. Users vs. MVP), flag which assumptions are critical blockers vs. safe to carry forward as open, and check `docs/risks.md` for gaps. An independent pass reliably catches real issues that inline self-review misses — on Anchor's first run, it caught a genuine contradiction between a stated mechanic and a stated success criterion that had gone unnoticed through two rounds of inline challenge.

### 6. Remediate

Fix must-fix findings concretely — edit the actual docs, populate `docs/risks.md` with real entries (likelihood/impact/mitigation), create any ADR the review flags as decided-but-undocumented via `/new-adr`. Only then flip `status` to `accepted`. Update `docs/roadmap.md`'s "Now"/"Next" to reflect the move into Design.

## Exit criteria

Vision, Concept, and MVP are `status: accepted` (Target Users is current, per its log-doc nature — no separate accept step), internally consistent with each other, and have survived an independent reviewer pass whose must-fix findings were actually resolved, not just acknowledged.
