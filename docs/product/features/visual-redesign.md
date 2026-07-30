---
status: draft
date: 2026-07-28
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/decisions/0002-desktop-app-framework-and-platform.md, docs/architecture/constraints.md, ideas/visual-redesign.md, ideas/switch-between-mini-and-full-ui.md, ideas/adjustable-timeline-view.md]
---

# Visual Redesign

> Created via `/new-feature visual-redesign`. Follow `.claude/workflows/design.md` — fill sections in order, don't skip to UX or Acceptance Criteria before Problem/Goals/Users/Alternatives/Trade-offs are settled. Run `grill-with-docs` on Alternatives/Trade-offs before moving `status` to `accepted`.
>
> **Unblocked 2026-07-29.** This doc previously said "Alternatives onward are blocked on a Concept revision." That revision **ran and was accepted on 2026-07-28** (`docs/concept/concept.md`, `docs/vision/vision.md`, [ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md)), so the prerequisite no longer exists. The resolution is recorded under "Decisions taken" below and its outcome was narrower than expected: **Anchor is capture-first, timeline-assisted**, so the hotkey/widget capture path is unchanged and only the dashboard's role gained a definition.
>
> **Design pass 2026-07-29** completed Alternatives through Acceptance Criteria. `status: draft` until an independent reviewer pass finds no must-fix items.
>
> **This feature is enabling work, not user-facing scope** (`docs/product/mvp.md` build order, step 3). It establishes the visual foundation the **Timeline Editor** (#14) is built on — designing that editor against today's debug-grade UI would mean building it twice. It does **not** design the editor. Decisions that belong to #14 are explicitly deferred, not absorbed.

## Problem

The dashboard (`app/src/routes/+page.svelte`) and mini widget (`app/src/routes/widget/+page.svelte`) are functional but visually unconsidered: default form controls, plain sections, no shared component vocabulary, no color system, no typographic hierarchy. The dashboard's `app.html` still carries the scaffold title `Tauri + SvelteKit + Typescript App`.

This matters for three concrete reasons, not for polish:

1. **It blocks other work.** Two further UI ideas — the mini/full window switch (`ideas/switch-between-mini-and-full-ui.md`) and what became the Timeline Editor (#14, from `ideas/adjustable-timeline-view.md`) — touch the same two surfaces. Designed against the current UI, both get redesigned again afterwards.
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

Decisions 1–4 above settled the *inputs* (premise, porting strategy, hue mapping, widget language). What follows are the design decisions that remained open once they did.

### A. Interaction philosophy — what the dashboard is *for*

The Concept revision defined the split but not its visual consequence.

1. **Treat both windows as peers**, styled identically — one system, minimum effort. Rejected: it contradicts the accepted position that the widget is the fast path and the dashboard is *"not meant for rapid interaction — opened deliberately."* Identical treatment would make the dashboard look like something to act in quickly, which is precisely what capture-first says it is not.
2. **Optimise the dashboard for density**, fitting the whole day on one screen — good for review, but pushes toward small targets and tight spacing, directly hostile to the direct-manipulation gestures the Timeline Editor will need.
3. **Two deliberately different postures from one system.** **Chosen.** The widget is **glanceable** — read in under a second, from across a desk, without focus. The dashboard is **inspectable** — read carefully, acted on deliberately, with room to manipulate. Same tokens, same components, different density and target sizing. The posture difference is what stops "one system" from meaning "one size."

### B. Information hierarchy on the dashboard

Today the dashboard presents history review, template management, export, hotkey settings, and the active task as one undifferentiated stack of sections.

1. **Keep the flat section list, restyled** — smallest change, but leaves the stated IA problem (Goals, bullet 2) unsolved. The decision recorded above was explicitly *styling **plus** IA*.
2. **Tabbed navigation** — one concern at a time, scales to more surfaces. Rejected: it hides the day's timeline behind a tab, when reviewing the day is the dashboard's whole purpose, and it fragments a surface the user opens *to see everything at once*.
3. **One primary surface, with secondary concerns demoted.** **Chosen.** The Timeline — presented as the Timeline Editor and the History View — is the dashboard's subject and occupies its main area. Export sits adjacent to it, because exporting is what reviewing leads to. Template management and hotkey settings are **configuration**, not daily work, and move behind a settings surface. The active task gets persistent placement, because it is the one thing that must be true at a glance even here.

   Frequency, not category, drives the ordering: reviewed daily, exported daily, configured rarely.

### C. Theme mechanism

Decision 3 above settled *light and dark, user-selectable, persisted*. How they are expressed is separate.

1. **Two hand-authored stylesheets** — total control per theme, but every component is specified twice and they drift. Rejected.
2. **Semantic design tokens with two value sets.** **Chosen.** Components reference *roles* (surface, text-primary, border, accent, danger), never raw colours; a theme is a set of values bound to those roles. The design system already ships tokens in this shape, so this is adopting its structure rather than inventing one. It also makes the third theme — high contrast, if ever needed — a value set rather than a rewrite.
3. **OS-follow only, no in-app control** — fewer moving parts, but decision 3 explicitly chose a persisted user preference.

### D. Density and target sizing

1. **One density everywhere** — simplest, but either the widget wastes its 260×90 or the dashboard is too tight for dragging.
2. **Two named densities: `compact` (widget) and `comfortable` (dashboard).** **Chosen.** A single spacing scale with two step selections, not two scales. This is the mechanism by which A's "two postures, one system" is actually delivered, and it gives #14 a defined baseline to design drag targets against rather than inventing spacing.

### E. Discoverability, given an expert sole user

1. **Conventional affordances** — tooltips, hover hints, visible labels on everything. Rejected as adding the explanatory chrome `users.md` says this persona does not need.
2. **Nothing but hotkeys** — matches the persona's fluency, but the dashboard contains operations (export range, rounding, template edits) that are used too rarely to memorise.
3. **Hotkey-first, discoverable-second.** **Chosen.** Every frequent action has a hotkey and shows it inline; every rare action is visibly labelled and clickable. The rule: **frequency determines whether an action must be memorable or merely findable.** No onboarding, no first-run tour, no empty-state tutorials.

### F. Accessibility scope

1. **Full WCAG AA conformance** — the right default for a shipped product, but this is a single-user tool for one known user, and conformance work not driven by a real need is scope this MVP has not paid for.
2. **Nothing beyond defaults** — rejected: two requirements here are not accessibility niceties but *correctness*, because the record's meaning depends on them.
3. **Contrast and non-colour encoding only.** **Chosen.** Two commitments, both because they materially affect interaction: text and meaningful boundaries meet **AA contrast in both themes**, and **no state is encoded by colour alone** — `SystemInferred` ends, reconstructed blocks, and the eight project hues must each carry a second channel (shape, weight, icon, or label). Everything else — full keyboard traversal of every control, screen-reader labelling, reduced-motion — is deferred, not refused.

   The non-colour rule is a *correctness* requirement: if an inferred end or a reconstructed block is distinguishable only by hue, the record's honesty depends on the viewer's colour perception and on the theme.

## Trade-offs

| | Interaction philosophy | Information hierarchy | Theme mechanism | Density | Discoverability | Accessibility |
|---|---|---|---|---|---|---|
| **Chosen** | Two postures, one system | Timeline primary; config demoted | Semantic tokens, two value sets | `compact` / `comfortable` from one scale | Hotkey-first, discoverable-second | Contrast + non-colour encoding only |
| Complexity | Moderate — one component set proven at two densities | Moderate — a settings surface that does not exist today | Low — the design system already ships tokens in this shape | Low — one scale, two step selections | Low | Low — two rules, checkable |
| Reversibility | High — postures are spacing and sizing, not structure | Moderate — moving config back is cheap; moving the Timeline out of primary is not | High — a third theme becomes a value set | High | High | High — remaining AA work is additive |
| Risk if wrong | Dashboard reads as a fast-path surface and invites capture there, undermining capture-first | Config buried too deep for something edited during setup | Components referencing raw colours leak past the token layer and break the second theme silently | Targets tuned for review turn out too small for #14's dragging | A rare action with no visible label becomes unreachable in practice | A state distinguishable only by hue misleads about the record's honesty |

## UX

Owned by the ux-designer. This section defines the **system**, not the screens; the Timeline Editor's own interaction design is #14's.

- **Component set, scoped to what the two surfaces actually use**: button (primary/secondary/danger), icon button, text input, select, toggle, tag/chip, card/section, dialog, and inline status. Built in Svelte from the design system's JSX as reference spec, per decision 2 — not ported wholesale, and not built ahead of need.
- **The widget's constrained subset**: current task name, elapsed time, stack depth, and state. `compact` density, no chrome, no controls that require aiming. It must stay legible at a glance from across a desk, and it may not grow beyond 260×90.
- **The dashboard**: the Timeline as the primary surface, export adjacent, active task persistently placed, configuration behind a settings surface. `comfortable` density throughout.

  "The Timeline" here means the *data*, not any particular view of it — so this IA is **implementable today with only the History View**, and the Timeline Editor slots into the same primary area when #14 lands. The redesign must not assume the Editor exists, or it cannot ship before the thing it is meant to enable.
- **Timestamps in monospace**, everything else in the humanist body face — the design system's rule, and it earns its place here: scanning a column of times is the History View's main reading task.
- **Per-project hue** from the persisted mapping (decision 3), applied as an accent on tags — never as the sole carrier of meaning, and never near the accent colour used for interactive state.
- **Two states must be visually distinct in both themes and without relying on colour**: an end that is `SystemInferred`, and a block whose `CaptureOrigin` is manual or adjusted. Both are commitments this project has already made in `interruption-stack.md` and `timeline-reconstruction.md`; the redesign is where they become real.
- **Motion**: the design system leaves it unspecified. Adopt one rule — 150–200ms ease-out for state changes, nothing decorative, nothing that delays a capture action.

**Deferred to #14, not decided here**: drag affordances and hit targets, how a clamp is visually communicated, undo presentation, zoom and time-range controls, minimum rendered block size, and orientation. This doc gives that work a spacing scale, a density baseline, and a component set to build from.

## Technical Constraints

Owned by technical-architect / senior-software-engineer.

- **Fonts must be bundled, not fetched.** The system specifies three families (Familjen Grotesk, Hanken Grotesk, JetBrains Mono); a Tauri desktop app has no guaranteed network. Three families is a real binary-size cost against [ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md), which chose Tauri partly for size — so **subset the bundled weights to those actually used**, and treat dropping to two families as a live option if the cost proves material. Tracked as `docs/risks.md` **R13**.
- **`csp: null` in `tauri.conf.json` is a scaffold default, not a decision.** Noted here because the font decision surfaces it — bundling locally removes the main reason to relax CSP — but **this redesign does not own it and it should not wait for it.** It is a live security posture in shipped config, tracked as `docs/risks.md` **R13**, and fixable independently today.
- **Components may not reference raw colour values.** Every colour resolves through a semantic token, or the second theme breaks silently and inconsistently. This is the one constraint whose violation is invisible until someone switches themes.
- **The theme preference persists** alongside hotkey bindings and export settings in `settings.json` — the existing durable-settings path, not a new mechanism.
- **The project→hue mapping is presentation-only.** It must never become an export aggregation key: export groups by exact name/project/client, and adding a field would silently change billed totals (`docs/risks.md` R2).
- **No behavioural change.** No transition type, no persisted timeline data, and no export output changes. If a visual requirement appears to need one, that is a signal to stop and raise it, not to make it.
- **`app/src/routes/+page.svelte` is classified binary by git** — two NUL bytes used deliberately as a delimiter in an `R.uniqBy` grouping key — so it gets no textual diffs or line-level merges, on the largest file this feature will rewrite. Separable; the rewrite may resolve it by choosing a delimiter that isn't NUL.

## Acceptance Criteria

- Zero unstyled native form controls remain on either route; every control resolves to a component in the defined set.
- No component references a raw colour value; every colour resolves through a semantic token. Verifiable by search, not by inspection.
- Both themes are complete: switching produces no unstyled, invisible, or illegible element on either surface.
- The theme preference survives an app restart, persisted alongside hotkey and export settings.
- Text and meaningful boundaries meet WCAG AA contrast **in both themes** — checked, not assumed.
- A `SystemInferred` end and a manual or adjusted `CaptureOrigin` are each distinguishable **with colour removed**, in both themes.
- Two project hues that are adjacent on the palette remain distinguishable from each other, and neither is confusable with the interactive accent.
- The mini widget renders correctly at exactly 260×90 with no scrolling, clipping, or overflow, and remains legible with strings ~40% longer than their English equivalents (`ideas/multi-language-ui-support.md`).
- Every frequent dashboard action displays its hotkey inline; every infrequent one is visibly labelled and reachable by pointer.
- Configuration (templates, hotkey bindings) is reachable from the dashboard without occupying its primary surface.
- A full Switch/Interrupt/Return cycle takes no additional interaction step compared to before the redesign — measured against `docs/vision/vision.md`'s Capture Latency target, which the redesign must not regress.
- No transition type, stored Time Block, or export output differs before and after the redesign, proven by an unchanged export from identical input.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
