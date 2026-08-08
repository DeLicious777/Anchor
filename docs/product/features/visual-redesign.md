---
status: accepted
date: 2026-08-02
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/principles.md, docs/risks.md, docs/glossary.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/product/features/timeline-reconstruction.md, docs/decisions/0002-desktop-app-framework-and-platform.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0006-stable-persistent-time-block-identity.md, docs/architecture/constraints.md, ideas/visual-redesign.md, ideas/switch-between-mini-and-full-ui.md, ideas/adjustable-timeline-view.md, ideas/multi-language-ui-support.md]
---

# Visual Redesign

> Created via `/new-feature visual-redesign`. Follow `.claude/workflows/design.md` — fill sections in order, don't skip to UX or Acceptance Criteria before Problem/Goals/Users/Alternatives/Trade-offs are settled. Run `grill-with-docs` on Alternatives/Trade-offs before moving `status` to `accepted`.
>
> **Unblocked 2026-07-29.** This doc previously said "Alternatives onward are blocked on a Concept revision." That revision **ran and was accepted on 2026-07-28** (`docs/concept/concept.md`, `docs/vision/vision.md`, [ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md)), so the prerequisite no longer exists. The resolution is recorded under "Decisions taken" below and its outcome was narrower than expected: **Anchor is capture-first, timeline-assisted**, so the hotkey/widget capture path is unchanged and only the dashboard's role gained a definition.
>
> **Design pass 2026-07-29** completed Alternatives through Acceptance Criteria. **Independently reviewed 2026-08-01**: no architectural blockers, seven must-fixes and four should-fixes, all applied. Two of them mattered more than wording — Alternative C rejected OS-follow theming by citing a decision that did not exist (now decision 5, argued), and the theme-persistence constraint named a shared `settings.json` path that does not exist in the code (four separate files; the accepted `architecture/constraints.md` carried the same error and was corrected with it).
>
> **`status: accepted` 2026-08-02.** Promoted on the distinction the review established: **this document defines the contract; the design system supplies the values.** Every decision here is decidable without the assets, and every acceptance criterion is a rule the assets get checked against rather than a value copied from them — which is why the missing design system does not prevent a correct implementation.
>
> **Accepted is not ready to implement.** The design system is not in this repository, so the spacing scale's steps, the hue palette and its size, and the font weights actually used cannot be reproduced by anyone but the author. Those must be recorded in Technical Constraints before implementation begins. This is the same gate ADR 0006 and `timeline-reconstruction.md` carry — accepted design, implementation blocked on an input — not a weaker form of acceptance.
>
> Two things the review flagged as *missing decisions* rather than missing assets were added before promotion, because they would have survived the assets arriving: a **minimum interactive target size** (D.2 — a spacing scale describes gaps, not targets, so #14 would still have been inventing numbers), and the **first-paint theme constraint** (Technical Constraints — the predictable cost of a backend-owned theme value).
>
> **This feature is enabling work, not user-facing scope** (`docs/product/mvp.md` build order, step 3). It establishes the visual foundation the **Timeline Editor** (#14) is built on — designing that editor against today's debug-grade UI would mean building it twice. It does **not** design the editor. Decisions that belong to #14 are explicitly deferred, not absorbed.

## Problem

The dashboard (`app/src/routes/+page.svelte`) and mini widget (`app/src/routes/widget/+page.svelte`) are functional but visually unconsidered: default form controls, plain sections, no shared component vocabulary, no color system, no typographic hierarchy. The dashboard's `app.html` still carries the scaffold title `Tauri + SvelteKit + Typescript App`.

This matters for three concrete reasons, not for polish:

1. **It blocks other work.** Two further UI ideas — the mini/full window switch (`ideas/switch-between-mini-and-full-ui.md`) and what became the Timeline Editor (#14, from `ideas/adjustable-timeline-view.md`) — touch the same two surfaces. Designed against the current UI, both get redesigned again afterwards.
2. **Information architecture, not just styling, is implicated.** The dashboard carries history review, template management, export controls and hotkey settings, and it does the sorting by **surface** rather than by **frequency**: a two-tab split (`+page.svelte:50`, `:362-369`) puts hotkey bindings under Settings and everything else — history, templates, export, the active task — under one Dashboard tab as a flat stack of sections. `docs/product/features/interruption-stack.md` establishes that the dashboard is "not meant for rapid interaction — opened deliberately," which is an IA claim the current layout does not express.

  *(Corrected 2026-08-01. This previously said the dashboard presents everything as "one undifferentiated surface" and listed **gap correction** among them. Neither held: the tab split already exists, and gap correction does not — `+page.svelte:671-675` renders `end_determination` read-only as `inferred`/`exact`, and no command edits a closed block's times. That mechanism is designed but unimplemented; see `docs/risks.md` **R9** and `timeline-reconstruction.md`. *(**Superseded 2026-08-06:** it is implemented — the same edit row now carries the block's start and end, and R9 is closed. The citation above has also shifted twice since it was written and now reads `+page.svelte:760-764`. Both facts are instances of the very pattern this note is about, which is why they are recorded here rather than silently corrected.)* Naming an unimplemented capability as present-tense evidence is exactly the failure `principles.md` #8 and **R11** exist to catch.)*
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

**1. The design system's timeline-first premise governs — and the Concept revision it triggered has since run and been accepted.**

**Outcome first, because everything below is how it was reached:** Anchor is **capture-first, timeline-assisted**. Primary capture stays sub-second on hotkeys and the mini widget; the Timeline is the canonical visualization and a reconstruction workspace, not a data-entry surface. **Nothing about the capture path changed.** What this feature gained is a settled definition of the dashboard's role — the thing the IA work needed.

<details>
<summary>How that was reached (historical; safe to skip)</summary>

The design system's README opens: *"Anchor is a work-time-tracking product built on a timeline-first premise: the workday is a continuous line to capture, view, and edit — timesheets are a generated report, not the primary interface."* The author chose this over the accepted hotkey-first Concept, in full knowledge that it was a Concept-level change.

Most of that sentence was already true of Anchor. The conflict was narrower than "the whole product changes," and precisely two things were genuinely in conflict:

- **(a) Editing.** "Capture, view, and **edit**" — timeline editing does not exist and is not in MVP scope. This is `docs/risks.md` **R9** and `ideas/manual-time-block-entry.md`. Under the new premise it stops being a deferred nice-to-have and becomes premise-critical.
- **(b) Which surface is primary.** The accepted docs make hotkeys and the mini widget primary, with the dashboard deliberate and secondary. `docs/product/features/interruption-stack.md` states the dashboard is "not meant for rapid interaction — opened deliberately, not part of the fast switch/interrupt/return loop." A timeline-first product inverts that.

Everything else the design system's premise asserts — the workday as a continuous line, timesheets as generated output rather than the interface — the accepted docs already say (flat timeline model in `docs/product/mvp.md`; export as generated output in `docs/product/features/export.md`).

**Blast radius as predicted at the time — retained for history, and two rows of it turned out to be wrong. Do not read this table as current:**

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

What was **not** affected: [ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md) survives intact — timeline editing is still the user stating what happened, never activity inference. [ADR 0003](../../decisions/0003-billable-classification-out-of-scope.md) is untouched; the design system's "timesheet" means generated output, which is what `export.md` already produces.

`vision.md` and `concept.md` returned to `accepted` on 2026-07-28, which is what unblocked the Alternatives below.

</details>

**2. Tokens transfer, components are rebuilt as needed.** Every design-system component is React JSX (`components/**/*.jsx`); Anchor is Svelte ([ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md), `docs/architecture/constraints.md`). The CSS tokens (`tokens/colors.css`, `tokens/spacing.css`, `tokens/typography.css`) are imported directly; Svelte components are hand-built only for what the two windows actually use, with the JSX kept as a reference spec rather than ported wholesale.

**3. Per-project tag hues use a persisted mapping.** An explicit, user-controlled project→hue mapping, **keyed by the project string** and persisted in its own store alongside the other settings (see Technical Constraints).

*"Stable across renames" — the phrase this decision originally used — is withdrawn as ambiguous (2026-08-01).* It could mean either "renaming a **task** does not disturb its project's hue," which is free and true, or "renaming a **project** carries its hue across," which **is not implementable**: there is no project entity and no durable project identifier (`docs/product/mvp.md` — aggregation happens at export time by exact name/project/client match, with no stored task entity; **R2**). Introducing one would create exactly the persisted identity the next sentence forbids. **Renaming a project therefore yields an unmapped project string, which takes the next unused hue** — the same behaviour as a project seen for the first time, and the user can re-assign it. That is a real limitation and it is accepted, not designed around.

What a persisted mapping still buys over hashing the project name to a hue: the user chooses which project gets which colour, and adding a new project cannot silently re-colour an existing one. **Constraint this must respect:** `docs/product/mvp.md` establishes there is no stored task entity and that aggregation happens at export time by exact name/project/client match. This mapping is presentation-only — it must never become an aggregation key, or it silently changes export output. Now carried as a Technical Constraint below.

**4. The widget uses a constrained subset** of the dashboard's component set — same tokens, fewer components, tighter density — not a distinct visual language. Resolves the open question carried from `ideas/visual-redesign.md`.

**5. Light and dark are both first-class, selected by an explicit persisted preference rather than by following the OS.** *(Added 2026-08-01. Alternative C previously cited "decision 3" for this — decision 3 is the hue mapping, and no decision recorded it. The only place the trade had been made was `ideas/visual-redesign.md`, which `CLAUDE.md` explicitly says is not held to feature-doc rigour. Argued here rather than inherited.)*

OS-follow is the cheaper option and was seriously considered: it is one fewer stored value, one fewer control, and it matches what most desktop apps do. It is rejected because Anchor's two windows have **different viewing conditions from each other**, which an OS-level signal cannot express. The widget sits over arbitrary applications all day at `alwaysOnTop`, while the dashboard is opened deliberately and looked at directly. A user who wants a dark widget that disappears into a dark IDE may still want a light dashboard to read a day's timeline in — and the reverse in a bright room. Deferring to one OS-wide bit forecloses that permanently.

The cost is honest: one more persisted setting, one more control to place, and a theme that can now disagree with the OS. Accepted because the preference is set approximately once.

## Alternatives

Decisions 1–5 above settled the *inputs* (premise, porting strategy, hue mapping, widget language, theme selection). What follows are the design decisions that remained open once they did.

### A. Interaction philosophy — what the dashboard is *for*

The Concept revision defined the split but not its visual consequence.

1. **Treat both windows as peers**, styled identically — one system, minimum effort. Rejected: it contradicts the accepted position that the widget is the fast path and the dashboard is *"not meant for rapid interaction — opened deliberately."* Identical treatment would make the dashboard look like something to act in quickly, which is precisely what capture-first says it is not.
2. **Optimise the dashboard for density**, fitting the whole day on one screen — good for review, but pushes toward small targets and tight spacing, directly hostile to the direct-manipulation gestures the Timeline Editor will need.
3. **Two deliberately different postures from one system.** **Chosen.** The widget is **glanceable** — read in under a second, from across a desk, without focus. The dashboard is **inspectable** — read carefully, acted on deliberately, with room to manipulate. Same tokens, same components, different density and target sizing. The posture difference is what stops "one system" from meaning "one size."

### B. Information hierarchy on the dashboard

Today the dashboard has a two-tab split (Dashboard / Settings) with hotkey bindings behind Settings, and everything else — history review, template management, export, the active task — as a flat stack of sections under Dashboard.

1. **Keep the current split, restyled** — smallest change, but leaves the stated IA problem (Goals, bullet 2) unsolved. This feature's scope is *styling **plus** IA*, which is what Goals bullet 2 argues for and what makes styling-only insufficient.
2. **Extend tabbed navigation to everything** — one concern at a time, scales to more surfaces. Rejected: applied to the timeline it would hide the day behind a tab, when reviewing the day is the dashboard's whole purpose, and it fragments a surface the user opens *to see everything at once*. Note this rejects **extending** the existing pattern, not introducing it — tabs already exist and the chosen option keeps a version of them.
3. **One primary surface, with secondary concerns demoted.** **Chosen.** The Timeline — presented as the Timeline Editor and the History View — is the dashboard's subject and occupies its main area. Export sits adjacent to it, because exporting is what reviewing leads to. Template management and hotkey settings are **configuration**, not daily work, and move behind a settings surface. The active task gets persistent placement, because it is the one thing that must be true at a glance even here.

   Frequency, not category, drives the ordering: reviewed daily, exported daily, configured rarely.

   **What is new here versus what already ships.** Hotkey bindings are already behind the Settings tab, so that half is done. The change is that **template management joins them** (today it sits in the Dashboard tab beside the day's work), that the Timeline becomes the Dashboard tab's *subject* rather than one section among several, and that export moves adjacent to it. Stated explicitly so this doc is not read as claiming credit for shipped behaviour.

### C. Theme mechanism

Decision 5 above settled *light and dark, user-selectable, persisted*. How they are expressed is separate.

1. **Two hand-authored stylesheets** — total control per theme, but every component is specified twice and they drift. Rejected.
2. **Semantic design tokens with two value sets.** **Chosen.** Components reference *roles* (surface, text-primary, border, accent, danger), never raw colours; a theme is a set of values bound to those roles. The design system already ships tokens in this shape, so this is adopting its structure rather than inventing one. It also makes the third theme — high contrast, if ever needed — a value set rather than a rewrite.
3. **OS-follow only, no in-app control** — fewer moving parts, but decision 5 rejected it: one OS-wide bit cannot express two windows with different viewing conditions.

### D. Density and target sizing

1. **One density everywhere** — simplest, but either the widget wastes its 260×90 or the dashboard is too tight for dragging.
2. **Two named densities: `compact` (widget) and `comfortable` (dashboard).** **Chosen.** A single spacing scale with two step selections, not two scales. This is the mechanism by which A's "two postures, one system" is actually delivered.

   **What this does and does not hand to #14.** It gives that work a named structure — one scale, two step selections — but this doc **does not yet state the scale's values or a minimum target size**, because the spacing tokens live in the design system and that artifact is not in this repository (see Technical Constraints). Until the values are recorded, #14 would still be inventing spacing, which is the failure this feature exists to prevent. **Importing the concrete steps is a prerequisite of implementing this feature**, not of accepting it. *(An earlier draft claimed a "defined baseline" was already delivered; it was not.)*

   **What #14 actually needs, and what this doc therefore decides here: a minimum interactive target size of 24×24 CSS pixels at `comfortable` density**, for any element a pointer must hit. A spacing scale alone would not have given #14 this — spacing describes gaps, not targets — so it is a design decision rather than an asset to import, and leaving it to the design system would have left the gap open even after the assets arrive.

   24×24 is WCAG 2.2 SC 2.5.8 (Target Size, Minimum, Level AA). Adopted for the same reason F.3 adopts contrast rather than full conformance: it is the floor below which a pointer interaction stops being reliable, not a conformance ambition. **It is a floor, not a target** — #14 may need considerably larger hit areas for drag handles on short blocks, and that is #14's call to make above this line, not below it. `compact` density on the widget is exempt: the widget has no controls that require aiming.

### E. Discoverability, given an expert sole user

1. **Conventional affordances** — tooltips, hover hints, visible labels on everything. Rejected as adding the explanatory chrome `users.md` says this persona does not need.
2. **Nothing but hotkeys** — matches the persona's fluency, but the dashboard contains operations (export range, rounding, template edits) that are used too rarely to memorise.
3. **Hotkey-first, discoverable-second.** **Chosen.** Every frequent action has a hotkey and shows it inline; every rare action is visibly labelled and clickable. The rule: **frequency determines whether an action must be memorable or merely findable.** No onboarding, no first-run tour, no empty-state tutorials.

### F. Accessibility scope

1. **Full WCAG AA conformance** — the right default for a shipped product, but this is a single-user tool for one known user, and conformance work not driven by a real need is scope this MVP has not paid for.
2. **Nothing beyond defaults** — rejected: two requirements here are not accessibility niceties but *correctness*, because the record's meaning depends on them.
3. **Contrast and non-colour encoding only.** **Chosen.** Two commitments, both because they materially affect interaction: text and meaningful boundaries meet **AA contrast in both themes**, and **no state is encoded by colour alone** — `SystemInferred` ends, reconstructed blocks, and project hues must each carry a second channel (shape, weight, icon, or label). *(This previously said "the eight project hues." No accepted doc, and nothing in this repo, establishes a palette of eight; the figure was unsourced and is withdrawn. The palette's size is an open input — see Technical Constraints.)* Everything else — full keyboard traversal of every control, screen-reader labelling, reduced-motion — is deferred, not refused.

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

  **The widget needs its own edge treatment in light theme, and this is where that is decided.** It is `decorations: false`, `shadow: false`, `alwaysOnTop: true` (`tauri.conf.json:26-31`), and today it is a translucent dark panel (`widget/+page.svelte:56`, `rgba(20,20,20,0.92)`) floating over arbitrary application windows. Dark-on-anything reads as a distinct object; **light-on-anything does not** — a pale panel over a pale editor has no boundary at all, and neither `decorations` nor `shadow` is available to supply one. So: **in light theme the widget carries an explicit 1px border in a token that meets AA contrast against both light and dark backdrops**, and keeps its translucency in dark theme where the contrast already does the work. Without this rule "both themes are first-class" is false for the surface that spends all day over other applications.

  **The theme control is not on the widget.** It has no controls that require aiming, and theme is set roughly once — it belongs on the dashboard's settings surface.
- **The dashboard**: the Timeline as the primary surface, export adjacent, active task persistently placed, configuration behind a settings surface. `comfortable` density throughout.

  "The Timeline" here means the *data*, not any particular view of it — so this IA is **implementable today with only the History View**, and the Timeline Editor slots into the same primary area when #14 lands. The redesign must not assume the Editor exists, or it cannot ship before the thing it is meant to enable.
- **Timestamps in monospace**, everything else in the humanist body face — the design system's rule, and it earns its place here: scanning a column of times is the History View's main reading task.
- **Per-project hue** from the persisted mapping (decision 3), applied as an accent on tags — never as the sole carrier of meaning, and never near the accent colour used for interactive state.
- **Three things must be visually distinct in both themes and without relying on colour**, on **two independent channels** — not one combined "this block was touched" mark:

  1. an end that is `SystemInferred` rather than `UserDetermined`;
  2. **origin** — whether the block was live-captured or manually entered;
  3. **adjusted-ness** — whether it has since been edited.

  *(Corrected 2026-08-01. This previously required one mark for "manual **or** adjusted", which would satisfy the letter of the rule while making `ManualEntryAdjusted` and `LiveCaptureAdjusted` indistinguishable. `timeline-reconstruction.md` is explicit that origin and adjusted-ness are preserved **independently** — "a manually entered block nudged once must stay distinguishable from a live capture that needed correcting" — so collapsing them is precisely the provenance loss F.3 calls a correctness failure rather than an accessibility one.)*

  The commitments come from `interruption-stack.md` and `timeline-reconstruction.md`; the redesign is where they get a visual form. **Partly real already**: `+page.svelte:671-675` renders an inferred end in italic (styled at `:811-818`) — non-colour by deliberate choice, with a comment saying so. That is the existing precedent to extend, not replace.

  *(Line references corrected 2026-08-04. Both citations here were accurate when written on 2026-08-01 and went stale when the Edit/Delete row actions landed on 2026-08-02. Found by `timeline-editor.md`'s first-pass review, which caught this doc's numbers being inherited into that one unchecked — a **R11**-shaped failure in miniature, and the reason a line citation is re-verified rather than copied.)*
- **Motion**: the design system leaves it unspecified. Adopt one rule — 150–200ms ease-out for state changes, nothing decorative, nothing that delays a capture action. **The duration is provisional and unvalidated** (`principles.md` #7): it is a conventional default, not a measured one. **Revisit if** any animation is perceptible as lag on a capture action, or if #14's drag feedback needs a different response curve — in which case the drag case gets its own value rather than this one being stretched to cover it.

**Deferred to #14, not decided here**: drag affordances and hit targets, how a clamp is visually communicated, zoom and time-range controls, minimum rendered block size, and orientation. This doc gives that work a spacing scale, a density baseline, and a component set to build from.

## Technical Constraints

Owned by technical-architect / senior-software-engineer.

- **The design system is not in this repository, and three decisions depend on it.** Decision 2 names `components/**/*.jsx` and `tokens/colors.css` / `spacing.css` / `typography.css`; the three-font constraint below, C.2's "the design system already ships tokens in this shape," D.2's spacing steps, and the project-hue palette all rest on it. Nothing under this repo root contains any of it, so **none of those claims is currently reproducible by anyone but the author.** *(Raised 2026-08-01.)* **Before implementation begins**, the system's location and version must be recorded here, and the three things this doc needs from it — the spacing scale's steps, the palette and its size, the font weights actually used — copied into this doc so the design survives the artifact moving or changing.

  **Recorded 2026-08-08 — Anchor Design System `1.0.0`.** The system has no repository or package; its durable form is its file tree (`tokens/{colors,typography,spacing}.css`, `styles.css`, `components/{core,forms,navigation,feedback}/*.jsx` + `.d.ts`), which must be vendored into this repo at a path we control and that path recorded here. **The values below satisfy this gate on their own** — they are copied, not referenced, so they survive the artifact moving. Vendoring the tree remains required before implementation, for the component sources; it is not required for the decisions above to be reproducible.

  **Spacing — one scale, named `--space-N` where N is the value ÷ 4** (so the scale skips: there is no `--space-7`).

  | | px |
  |---|---|
  | Scale | 4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80 |
  | `compact` (widget) | 4, 8, 12 — `--space-1/2/3` |
  | `comfortable` (dashboard) | 16, 24, 32 — `--space-4/6/8` |

  Radii 8 / 12 / 18 / 24 px plus pill; shadows sm / md / lg plus a focus ring.

  **Project hues — 8**, each a foreground with a paired tint. amber `#c98a1e`/`#f7ecd4` · coral `#d1634a`/`#f9e1da` · teal `#1f7a6c`/`#dcf0ea` · indigo `#3f4f9e`/`#e2e5f5` · moss `#5c7a3a`/`#e7f0da` · plum `#8a4a7a`/`#f2e1ee` · sky `#2f7fa8`/`#dcedf5` · clay `#a85c3a`/`#f3e2d6`. **The collision rule past 8 projects is ours and is not yet decided** — the system does not define one.

  **Font weights actually used — four files.** Familjen Grotesk **600** (display); Hanken Grotesk **400** (body) and **500** (labels); JetBrains Mono **500** (timestamps). Imported-but-unused weights (Familjen 500/700, Hanken 600 and italic) may be dropped from the bundle with no visual loss — which is the concrete answer to **R13**(b).

  **Semantic tokens.** Light and dark are both complete; five light values were corrected against AA contrast before adoption, and every ratio below was recomputed rather than taken on trust.

  | Role | Light | Dark |
  |---|---|---|
  | canvas | `#faf8f4` | `#15130f` |
  | surface | `#ffffff` | `#1f1c16` |
  | surface-sunken / elevated | `#f2efe9` | `#2a261e` |
  | hairline | `#e7e1d7` | `#332e25` |
  | hairline-strong | `#a39075` *(was `#d8d0c2`, 1.53:1)* | `#d8d0c2` |
  | ink / text-primary | `#1a1712` | `#f5f2ec` |
  | ink-soft / text-secondary | `#55504a` | `#b3ac9f` |
  | text-muted | `#746d63` *(was `#8a8377`, 3.75:1)* | `#b3ac9f` |
  | accent — **fill** | `#2a4373` | `#2a4373` (unchanged; white text on it is 9.77:1 in both) |
  | accent — **border** | `#2a4373` | `#4a67a0` |
  | accent — **foreground** | `#2a4373` | `#7d94c9` |
  | accent-on | `#ffffff` | `#ffffff` |
  | secondary-accent | `#d13a24` | `#d13a24` (fill only) |
  | success | `#276846` *(was `#2f7a52`)* | `#4fae7d` |
  | warning | `#846115` *(was `#c98a1e`, 2.94:1)* | `#dba53d` |
  | danger | `#993a29` *(was `#b9432f`)* | `#e0705a` |
  | success/warning/danger tint | `#dcf0e2` / `#f9ecd2` / `#fae0da` | **none — see below** |

  **`accent` is three roles, not one.** It is theme-invariant as a *fill* and must be remapped as a *border* or *foreground*: the light value on the dark surface is **1.74:1**. Reusing the fill value for dark link text or a focus ring is the one mistake this table exists to prevent.

  **Status colour in dark theme has no tint** — see the separate constraint below. The `-100` tints are light-theme only.

- **Fonts must be bundled, not fetched.** The system specifies three families (Familjen Grotesk, Hanken Grotesk, JetBrains Mono); a Tauri desktop app has no guaranteed network. Three families is a real binary-size cost against [ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md), which chose Tauri partly for size — so **subset the bundled weights to those actually used**, and treat dropping to two families as a live option if the cost proves material. Tracked as `docs/risks.md` **R13**.
- **~~`csp: null` in `tauri.conf.json` is a scaffold default, not a decision.~~ Fixed 2026-08-08, independently of this redesign, exactly as this note said it should be.** A two-layer policy now ships — `svelte.config.js`'s `csp.mode: "hash"` for the one inline bootstrap script whose hash changes every build, and a header policy in `tauri.conf.json` for everything else. Bundling fonts locally, when it happens, needs no CSP change: `font-src 'self'` already covers it. The original note follows, since its reasoning is what got this done.
- **`csp: null` in `tauri.conf.json` is a scaffold default, not a decision.** Noted here because the font decision surfaces it — bundling locally removes the main reason to relax CSP — but **this redesign does not own it and it should not wait for it.** It is a live security posture in shipped config, tracked as `docs/risks.md` **R13**, and fixable independently today.
- **Status colour in dark theme sits directly on the surface, not on a tint.** *(Decided 2026-08-08 with the design system.)* The `-100` tint backgrounds are light-theme only: a dark status foreground on a light tint measures 1.90–2.52:1, below even the non-text floor, so remapping the tints was the alternative and dropping them is the choice. Dark status components render as colour on `surface-dark`, where the supplied values clear 4.5:1 (5.37–7.66). This also matches what already ships — `+page.svelte`'s `.error` and `.success` are flat coloured text with **bold weight** as their second channel, never tinted pills. If a pill silhouette is ever wanted back in dark, it needs a **1 px border in the status colour**, not a neutral fill: `surface-elevated` against `surface` is 1.13:1 and would not read as a shape at all.
- **Components may not reference raw colour values.** Every colour resolves through a semantic token, or the second theme breaks silently and inconsistently. This is the one constraint whose violation is invisible until someone switches themes.
- **The theme preference persists** in its own `theme.json`, following the pattern the other settings already use.

  *(Corrected 2026-08-01. This previously said "alongside hotkey bindings and export settings in `settings.json` — the existing durable-settings path, not a new mechanism." **There is no such shared path.** `paths.rs:7-29` resolves four separate files: `settings.json` holds only `HotkeyBindings`, `export_settings.json` holds `ExportSettings`, plus `templates.json` and `transitions.jsonl`. The accepted `docs/architecture/constraints.md` carries the same error and is corrected in the same pass.)*

  **Why a fourth settings file rather than consolidating.** Consolidation is the tidier end state and was the alternative considered. It is rejected **for this feature**: the two existing stores have materially different lifecycles — hotkeys are read once at startup, export settings are live-mutable through `ExportSettingsState` — so merging them is a change to shipped, working persistence code, made by a presentation-layer feature, in service of tidiness. That violates this doc's own "no behavioural change" constraint below and `principles.md`'s smallest-correct-change discipline. A separate file matches the established pattern exactly and costs one `paths.rs` function.

  **The decisive argument is ownership, not consistency.** The three existing stores exist because the **Rust core consumes them**: hotkeys are registered by the global-hotkey system, export settings are read by `export.rs`, templates are served over IPC. Theme is the first setting the backend has no use for at all — only the webview reads it. That makes frontend-owned persistence (`localStorage`) a real third option, and it is rejected on two specific grounds: it would introduce a **second persistence tier** into a project that has exactly one (JSON in the app data dir), for a single value; and the dashboard and widget are separate webviews that must agree, which a Rust-owned value gives for free.

  **If consolidation is wanted, it is its own piece of work** and should be sequenced deliberately rather than absorbed here. The duplication worth removing is the load/save boilerplate — now written three times and about to be four — which argues for a generic persistence helper, not for merging files. Recorded so the fragmentation is a knowing choice rather than an accreted one.

- **The theme must be applied before first paint.** This is the cost the ownership choice above takes on: a backend-owned value is not available to the webview synchronously, so a naive implementation renders the default theme and then corrects itself — a visible flash on every launch, on both windows, and worst in dark theme where the flash is bright. **The mechanism is implementation's to choose** (passing the value at window creation, holding the window hidden until the theme resolves, or another route); **the constraint is not**: no window may render one theme and then switch to the other. Stated because it is the one predictable defect this decision invites, and it is much cheaper to design around than to retrofit.
- **The project→hue mapping is presentation-only.** It must never become an export aggregation key: export groups by exact name/project/client, and adding a field would silently change billed totals (`docs/risks.md` R2).
- **No behavioural change.** No transition type, no persisted timeline data, and no export output changes. If a visual requirement appears to need one, that is a signal to stop and raise it, not to make it.
- **~~`app/src/routes/+page.svelte` is classified binary by git~~ — fixed 2026-08-08.** The `R.uniqBy` key that needed them now uses `JSON.stringify([name, project, client])`, which needs no delimiter character at all, so the ambiguity NUL was chosen to avoid cannot arise — and the file is ordinary text, with line-level diffs and merges. The original note follows, since it is what got this done:
- **`app/src/routes/+page.svelte` is classified binary by git** — two NUL bytes used deliberately as a delimiter in an `R.uniqBy` grouping key — so it gets no textual diffs or line-level merges, on the largest file this feature will rewrite. Separable; the rewrite may resolve it by choosing a delimiter that isn't NUL.

## Acceptance Criteria

- Zero unstyled native form controls remain on either route; every control resolves to a component in the defined set.
- No component references a raw colour value; every colour resolves through a semantic token. Verifiable by search, not by inspection.
- Both themes are complete: switching produces no unstyled, invisible, or illegible element on either surface.
- The theme preference survives an app restart, persisted alongside hotkey and export settings.
- Text and meaningful boundaries meet WCAG AA contrast **in both themes** — checked, not assumed.
- A `SystemInferred` end is distinguishable **with colour removed**, in both themes.
- Origin and adjusted-ness are distinguishable **independently and with colour removed**: all four of `LiveCapture`, `LiveCaptureAdjusted`, `ManualEntry` and `ManualEntryAdjusted` are told apart from one another, not merely separated into touched and untouched.
- The project→hue mapping survives an app restart, and a project string with no mapping is assigned the next unused hue rather than sharing one.
- The Timeline occupies the dashboard's primary area, with export adjacent to it and the active task persistently placed; template management and hotkey bindings are reachable but do not occupy that area.
- Both densities resolve from a single spacing scale — `compact` and `comfortable` select different steps of the same scale, and no component hard-codes a spacing value outside it.
- Timestamps render in the monospace face on both surfaces; no other content does.
- The widget displays current task name, elapsed time, stack depth and state, and nothing else.
- State-change animations complete within the adopted motion rule, and no animation sits between a capture action and its visible result.
- Neither window renders one theme and then switches to the other on startup; the persisted theme is applied before first paint.
- Every element a pointer must hit is at least 24×24 CSS pixels at `comfortable` density.
- Every pair of project hues in the palette is distinguishable from every other, and no hue is confusable with the interactive accent — checked across the whole palette rather than only between neighbours, since the mapping is user-assigned and any two projects can end up side by side.
- The mini widget renders correctly at exactly 260×90 with no scrolling, clipping, or overflow, and its text areas are sized so that a **German** rendering of each label fits without clipping — the concrete thing `ideas/multi-language-ui-support.md` asks of this redesign whether or not translation is ever built, since the window is `resizable: false` and cannot be grown later. *(The "~40% longer" figure this criterion previously cited appears nowhere in that document; it was invented. Testing against the actual longest supported language is both truer and easier to check.)*
- Every dashboard action classified **frequent** displays its hotkey inline; every action classified **infrequent** is visibly labelled and reachable by pointer. E.3 states the rule but never applies it, so the classification is fixed here: **frequent** — Start, Switch, Interrupt, Return Previous, Return Original, Complete, Rename, **Pause** (those that have or warrant bindings); **infrequent** — export range and rounding, template create/edit/delete, hotkey rebinding, theme selection, project hue assignment, **opening the Interruption History panel and dismissing a frame**.

  *(Extended 2026-08-07. This list was exhaustive when written on 2026-08-01 and named seven actions; **Pause** and the Interruption History actions did not exist as accepted features until 2026-08-07. `pause.md` decision 6 classifies Pause as frequent on this doc's own rule — its value is that the recorded end is the moment the user stopped, so a window trip makes every paused block's end late — and `interruption-history.md` decision 8's disclosure is infrequent and therefore findable rather than memorable. Completing an enumeration that a later accepted feature outgrew is not reopening it; the rule and the classification method are unchanged. Note "have or warrant" is doing real work here: Start and Rename are frequent with no binding today, and Pause joins them as warranting one.)* Drawing the line in the design rather than during implementation is the point of having the rule.
- Configuration (templates, hotkey bindings) is reachable from the dashboard without occupying its primary surface.
- A full Switch/Interrupt/Return cycle performed **on the dashboard** takes no additional interaction step compared to before the redesign, and the **hotkey and widget paths are untouched** — no markup, handler, or command on either changes.

  *(Split 2026-08-01 from a criterion that conflated two things and tested the wrong surface. It asserted a step count but claimed to measure it "against `vision.md`'s Capture Latency target" — Capture Latency is a **time**, inclusive of the durable write, which `vision.md` itself records as not measured and which a presentation-layer change cannot move. It also named the primary capture path, which runs through hotkeys and the widget; `widget/+page.svelte` is display-only, so a click-driven capture cycle exists solely on the dashboard — the surface capture-first deliberately de-prioritises. What the redesign can actually guarantee is that it does not touch the fast path at all.)*
- No transition type, stored Time Block, or export output differs before and after the redesign, proven by an unchanged export from identical input.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
