---
status: draft
date: 2026-07-28
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/decisions/0002-desktop-app-framework-and-platform.md, docs/architecture/constraints.md, ideas/visual-redesign.md, ideas/switch-between-mini-and-full-ui.md, ideas/adjustable-timeline-view.md]
---

# Visual Redesign

> Created via `/new-feature visual-redesign`. Follow `.claude/workflows/design.md` — fill sections in order, don't skip to UX or Acceptance Criteria before Problem/Goals/Users/Alternatives/Trade-offs are settled. Run `grill-with-docs` on Alternatives/Trade-offs before moving `status` to `accepted`.
>
> **Stages complete:** Problem, Goals, Users. **Alternatives onward are blocked on a Concept revision** — see "Decisions taken" (fork 1) below. The author chose that the `Anchor Design System`'s timeline-first premise governs over this repo's accepted hotkey-first Concept. That is an upstream change: writing this feature's Alternatives, UX, or Acceptance Criteria first would design a presentation layer for a product definition the accepted docs currently contradict.

## Problem

The dashboard (`app/src/routes/+page.svelte`) and mini widget (`app/src/routes/widget/+page.svelte`) are functional but visually unconsidered: default form controls, plain sections, no shared component vocabulary, no color system, no typographic hierarchy. The dashboard's `app.html` still carries the scaffold title `Tauri + SvelteKit + Typescript App`.

This matters for three concrete reasons, not for polish:

1. **It blocks other work.** Two further UI ideas — the mini/full window switch (`ideas/switch-between-mini-and-full-ui.md`) and the timeline view (`ideas/adjustable-timeline-view.md`) — touch the same two surfaces. Designed against the current UI, both get redesigned again afterwards.
2. **Information architecture, not just styling, is implicated.** The dashboard carries history review, template management, export controls, hotkey settings, and gap correction in one undifferentiated surface. `docs/product/features/interruption-stack.md` establishes that the dashboard is "not meant for rapid interaction — opened deliberately," which is an IA claim the current layout does not express.
3. **A design system now exists for this product but was authored without the codebase.** Its own README states no codebase, Figma, or deck was attached and that its components and screens are "original interpretations of the brief." It therefore encodes product assumptions that were never checked against this repo's accepted decisions — see "Open forks."

Evidence is direct inspection of the two route files and `app/src-tauri/tauri.conf.json`, plus the accepted feature docs above. No user research is involved or needed: the sole user is the author (see Users).

## Goals

Tied to `docs/vision/vision.md`'s success criterion that the author can run a real workday through Anchor and trust the result.

- **A single component vocabulary covers both windows.** Every control on either surface comes from a defined set rather than a bare HTML default. Measurable: zero unstyled native form controls remain on either route.
- **The dashboard's IA expresses its stated role.** The deliberate, non-time-critical surfaces (history, templates, export, settings) are visually grouped and ordered so that the ones used daily are reachable without hunting, and the ones used rarely do not compete with them.
- **Light and dark are both first-class.** Every component is specified in both, with a persisted user preference — not a dark theme retrofitted onto light-mode assumptions.
- **The widget stays within its hard envelope.** 260×90, `resizable: false`, `decorations: false` (`app/src-tauri/tauri.conf.json`). The redesign must not require growing it, and must leave legible room for longer strings than English (`ideas/multi-language-ui-support.md`).
- **The redesign does not change tracked behavior.** No transition type, no persisted timeline data, and no export output changes as a result of this feature. It is a presentation-layer change.

Explicit non-goal: this feature does not add the timeline visualization, the window-mode switch, or timeline editing. It establishes the system those are later built in.

## Users

The single segment in `docs/product/users.md`: **the interrupted billable developer**, currently the author alone in personal use.

Two persona properties bear directly on this feature:

- *"Values speed and low friction above all — if tracking takes more than a couple of seconds of thought, it won't be used consistently."* The redesign is judged on whether it makes the fast path faster, not on whether it looks considered. Any visual choice that adds a step to Switch/Interrupt/Return fails this.
- *"Comfortable with hotkeys and a command palette"* and explicitly not needing onboarding. The redesign should not add explanatory chrome, empty-state tutorials, or first-run guidance that a sole expert user does not need.

No new user segment is implied. Per `docs/product/users.md`, if one were, that doc would be updated first rather than a segment invented here.

## Decisions taken (2026-07-28)

**1. The design system's timeline-first premise governs.** Its README opens: *"Anchor is a work-time-tracking product built on a timeline-first premise: the workday is a continuous line to capture, view, and edit — timesheets are a generated report, not the primary interface."* The author chose this over the accepted hotkey-first Concept, in full knowledge that it is a Concept-level change.

Most of that sentence is already true of Anchor. The conflict is narrower than "the whole product changes," and precisely two things are genuinely in conflict:

- **(a) Editing.** "Capture, view, and **edit**" — timeline editing does not exist and is not in MVP scope. This is `docs/risks.md` **R9** and `ideas/manual-time-block-entry.md`. Under the new premise it stops being a deferred nice-to-have and becomes premise-critical.
- **(b) Which surface is primary.** The accepted docs make hotkeys and the mini widget primary, with the dashboard deliberate and secondary. `docs/product/features/interruption-stack.md` states the dashboard is "not meant for rapid interaction — opened deliberately, not part of the fast switch/interrupt/return loop." A timeline-first product inverts that.

Everything else the design system's premise asserts — the workday as a continuous line, timesheets as generated output rather than the interface — the accepted docs already say (flat timeline model in `docs/product/mvp.md`; export as generated output in `docs/product/features/export.md`).

**Blast radius — what this invalidates and must be revised before this feature resumes:**

| Doc | What changes |
|---|---|
| `docs/vision/vision.md` | States the hotkey-first premise and non-goals. Needs revisiting for (b). |
| `docs/concept/concept.md` | "How it's different" is built on the interruption mechanic as the differentiator, not a timeline. Needs revisiting for (b). |
| `docs/product/mvp.md` | Dashboard/widget roles and in-scope list. Timeline editing (a) must move into scope, and something else should move out — this doc's own rule is that an MVP which only grows isn't an MVP. |
| `docs/product/features/interruption-stack.md` | `status: accepted`; its UX section's "not meant for rapid interaction" claim looked contradicted by (b). |
| `docs/risks.md` R9 | Reclassifies from an unmitigated gap to premise-critical scope. R3's mitigation story changes with it. |
| New ADR | (a) needs one: a transition type carrying explicit start **and** end, log-order vs. chronological-order under ADR 0004, and overlap rules. See `ideas/manual-time-block-entry.md`. |

### Resolution (2026-07-28, Concept revision session)

The Concept revision ran and settled this. The outcome was **narrower than the table above predicted**, and one row of it turned out to be wrong:

- **Anchor is capture-first, timeline-assisted.** Primary capture stays sub-second on hotkeys and the mini widget; the timeline is the canonical visualization and a *reconstruction workspace*, not a data-entry surface. `docs/vision/vision.md`'s "couple of seconds via a global hotkey" criterion **survives**.
- **`interruption-stack.md`'s "not meant for rapid interaction" claim survives too** — the row above was wrong, since capture never moved to the dashboard. That doc needed only two smaller amendments (progressive disclosure of the stack; R9 rescheduled).
- **What did change:** two `vision.md` success criteria were absolute in a way the product isn't — "no manual reconciliation" and "without hand-editing" became "minimal manual effort, entirely inside Anchor" and "no post-processing in Excel or any external tool." `concept.md`'s differentiator claim was rewritten around evidence: the timeline is explicitly *not* the differentiator (Toggl already ships drag-create/drag-edit), the interruption model is.
- **Timeline reconstruction entered MVP scope**, subsuming R9. `docs/product/mvp.md` records an unpaid scope trade this created.

**This feature is unblocked for Alternatives** once `vision.md`/`concept.md` return to `accepted` via the Discovery workflow's Stage 5 reviewer pass. The IA work can now proceed against a settled premise: a dashboard organized around a timeline that is reviewed and refined deliberately, with capture living in the widget.

What is **not** affected: [ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md) survives intact — timeline editing is still the user stating what happened, never activity inference. [ADR 0003](../../decisions/0003-billable-classification-out-of-scope.md) is untouched; the design system's "timesheet" means generated output, which is what `export.md` already produces.

**2. Tokens transfer, components are rebuilt as needed.** Every design-system component is React JSX (`components/**/*.jsx`); Anchor is Svelte ([ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md), `docs/architecture/constraints.md`). The CSS tokens (`tokens/colors.css`, `tokens/spacing.css`, `tokens/typography.css`) are imported directly; Svelte components are hand-built only for what the two windows actually use, with the JSX kept as a reference spec rather than ported wholesale.

**3. Per-project tag hues use a persisted mapping.** An explicit, user-controlled project→hue mapping, stable across renames. **Constraint this must respect:** `docs/product/mvp.md` establishes there is no stored task entity and that aggregation happens at export time by exact name/project/client match. This mapping is presentation-only — it must never become an aggregation key, or it silently changes export output. Worth stating in Technical Constraints once that stage is reachable.

**4. The widget uses a constrained subset** of the dashboard's component set — same tokens, fewer components, tighter density — not a distinct visual language. Resolves the open question carried from `ideas/visual-redesign.md`.

## Alternatives

_Blocked. Decisions 2–4 are settled and survive the premise change, but the largest alternative set in this feature — the dashboard's information architecture (Goals, bullet 2) — depends entirely on decision 1's unresolved Concept revision. Writing it now guarantees rewriting it._

## Trade-offs

_Blocked on Alternatives._

## UX

_Blocked on Trade-offs. Owned by ux-designer._

## Technical Constraints

_Blocked on UX. Owned by technical-architect / senior-software-engineer._

Provisionally noted, not yet decided — recorded here so they are not lost, and flagged because per `.claude/workflows/design.md` stage 7 a constraint asserting something no Alternative established is a gap, not a detail:

- **Fonts must be bundled, not fetched.** The system specifies Familjen Grotesk, Hanken Grotesk, and JetBrains Mono. A Tauri desktop app has no guaranteed network access and `tauri.conf.json` sets `csp: null` (itself worth revisiting). Three bundled families have a real binary-size cost that the alternatives should weigh.
- **Motion, hover, and press are unspecified.** The design system's README flags all three as placeholders awaiting real specs.
- **`+page.svelte` is classified binary by git.** Two NUL bytes, used deliberately as a delimiter in a `R.uniqBy` grouping key, cost the file textual diffs and line-level merges — on the largest file this feature will rewrite. Separable from this feature; noted so the rewrite can choose to resolve it.

## Acceptance Criteria

_Blocked on Technical Constraints._

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
