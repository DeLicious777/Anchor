# MVP Scope

_Last reviewed: 2026-07-28 (Concept revision — see `docs/concept/concept.md`)_

> **Scope grew on 2026-07-28 by four items, not one; the trade is deliberately deferred to Planning.** The Concept revision added timeline reconstruction, the Timeline Editor, Pause/Continue Session, and the interruption-history panel (see "In scope"). An earlier draft of this note counted only the first and called the other three clarifications — corrected after the independent review, because deferring a scope trade to Planning is only legitimate if Planning receives an accurate scope. This doc's closing rule says growth should be paid for by removing something; the author's decision is to cut nothing during Discovery, on the grounds that Discovery defines the right product and prioritisation belongs to the phase that follows. See "Open scope trade" below.

## MVP definition

A desktop application for a single user that lets them track a full workday as a sequence of independent time blocks, using fast, hotkey/command-palette-driven operations, always able to return cleanly from any depth of interruption — and export the resulting timeline for billing and reporting.

## In scope

- **Core interruption model**: Switch, Interrupt, Return to Previous Task, Return to Original Task, Complete — backed by a true nested stack internally, simple UI on top. See `docs/product/features/interruption-stack.md`.
- **Durable persistence with gap recovery**: every transition is written durably (append-only log); an ungraceful crash or a resume from sleep/hibernate is detected and reconciled (`EndDetermination::SystemInferred`, inferred end time, user-correctable via timeline reconstruction) rather than silently lost or silently trusted.
- **Manual, assisted tracking only** — no passive/automatic *activity* detection (what the user was doing is never inferred), no calendar integration. This is distinct from crash/sleep gap recovery above, which infers only a recovery-time timestamp, never activity content — see ADR 0001's amendment.
- **Desktop app, always-available**, with user-remappable global hotkeys, an always-on-top mini widget, and a dashboard for history/templates/export.
- **Task Templates** — reusable presets pre-populating name/project/client for recurring activities.
- **Flat timeline data model** — each time block is an independent entry (name, start, end, duration, optional project/client); aggregation by name/project/client happens at export/report time, not via a stored task entity.
- **Exports**: XLSX and JSON, generated from the timeline — the state reconstructed from the transition log, which is the actual source of truth (corrected 2026-07-28; see `docs/concept/concept.md`). Column structure should map cleanly onto an enterprise time-booking system's project/activity hierarchy (e.g. Produra) so hours transfer by copy rather than by rework — a **format consideration only**. Anchor's own model stays flat (name + optional project/client); no sub-project/activity fields are added, and no direct integration is built (that remains out of scope below, decided 2026-07-28).
- **Timeline reconstruction** _(added 2026-07-28 by the Concept revision)_: five operations — **add, move, resize, edit identity, delete** — each with a stated problem. Split and merge were considered and removed. Every block always represents work that actually happened; no future-dated or planned blocks. This subsumes the `recovered-gap` correction path that `docs/product/features/interruption-stack.md` has always promised but never had a mechanism for (`docs/risks.md` R9).
- **Timeline Editor** _(added 2026-07-28; previously hidden inside "reconstruction")_: the graphical, direct-manipulation surface. Listed separately because it is a **hard prerequisite** — move and resize are meaningless against the tabular History View — and because folding it into "reconstruction" was concealing implementation-critical scope from Planning. Its open design questions (minimum block size, hit targets, zoom, orientation, range, live-growing active block) are in `ideas/adjustable-timeline-view.md`.
- **Pause, and `Start` as a first-class action** _(added 2026-07-28)_: Pause stops tracking without unwinding an open interruption stack — a specialised Interrupt that pushes a frame without starting a successor, so the paused task keeps its return path. A hole-closing change, not a convenience feature: `Complete` is currently rejected while the stack is non-empty. Requires `ReturnPrevious`/`ReturnOriginal` to become legal with no active task, and `Start` to be exposed as its own command (the transition exists; no command does). Continue Session is a UI action, not a transition.
- **Progressive disclosure of the interruption stack** _(added 2026-07-28; a new UI surface, not a clarification)_: default UI stays exactly as designed (current task + two return options); the full stack is inspectable behind an optional interruption-history panel, which is also where a frame can be explicitly dismissed. No third return path.

The three-field Time Block model (End Determination, Capture Origin, Interruption Outcome), the new transitions (reconstruction edits, Pause/Continue Session, frame dismissal), and the snapshot-payload guarantee all require **one Event Model ADR** before implementation — amending, not editing, [ADR 0004](../decisions/0004-transition-log-format-and-torn-write-scheme.md), whose schema is an accepted on-disk contract.

_(Feature docs for each of the above land in `docs/product/features/` once the Design workflow runs for each — see `.claude/workflows/design.md`.)_

## Open scope trade

**Decision (2026-07-28): nothing is cut now; the trade moves to Planning.**

The author's reasoning: Discovery's job is to define the right product, and prioritisation is the next phase's job. The four things this session settled — capture-first, timeline reconstruction as core, the interruption stack as the differentiator, the timeline as the best surface for that data — are structural. The plausible cuts below are all *feature-level* and stay easy to move once epics exist. Cutting one now would be optimising a document before there is anything to sequence against.

What replaces the cut is an explicit build order, to be turned into epics and milestones by `.claude/workflows/planning.md`:

1. **Capture** — hotkeys and mini widget
2. **Interruption stack** (including Pause/Continue Session, which closes a hole in it)
3. **Visual redesign** — *enabling work, not user-facing scope*
4. **Timeline Editor**, then **timeline reconstruction** on top of it
5. **Export** — at least one working end-to-end workflow
6. **Task Templates**
7. **Further export formats**

**This is a sequence of scope, not of unstarted work.** Items 1, 2, 5 and 6 are already substantially built — `app/src-tauri/src/commands.rs` exposes switch, interrupt, rename, both returns, complete, template CRUD, both exports, and hotkey bindings. They appear here because the 2026-07-28 revision changed what they must *become* (the three-field model, Pause, first-class `Start`, `Return*` legal with no active task), not because they are greenfield. Planning must size the deltas, not the features — and note the candidates table below already calls Task Templates "already built" while sequencing it at position 6. Flagged rather than silently reordered: this is an R11 instance, and the deferral to Planning is only legitimate if Planning receives an accurate picture.

Two things this ordering makes explicit that the first draft got wrong:

- **The redesign is a prerequisite, not a follow-up.** `ideas/visual-redesign.md` and `docs/product/features/visual-redesign.md` both record that designing the Timeline Editor against today's debug-grade UI means building it twice. It is enabling work — it does not become an MVP *feature*, but it does have to precede the Editor.
- **The Timeline Editor precedes reconstruction**, because reconstruction's core gestures only exist on it.

Items 6 and 7 are the natural v1.1 boundary, so the MVP-vs-v1.1 line gets drawn there rather than by deleting anything from this doc. **The risk being accepted** is that a deferred cut is not a cut — if Planning does not actually draw that line, this doc has grown with nothing given back, which is the failure mode its own closing rule exists to prevent.

Candidates, retained for that Planning decision (none chosen):

| Candidate | Argument for removing | Argument against |
|---|---|---|
| **JSON export** | XLSX is the format that feeds the actual billing workflow; JSON has no named consumer yet, and `export.md` already had to do real work making the two agree numerically. | Cheap — it reuses the same grouping/rounding computation as XLSX. |
| **Task Templates** | Reconstruction plus autocomplete over past task history covers much of the same "fast start for recurring work" need. | Already `accepted`, already built, and directly mitigates R2/R8 fragmentation. |
| **Nothing — accept a larger MVP** | The reconstruction path is what makes the manual-tracking bet survivable (it mitigates R3, which currently has no mitigation at all). | Contradicts this doc's stated rule; risks an MVP that never ships. |

## Explicitly out of scope (deferred)

- Passive/automatic activity detection, idle detection (i.e. flagging that the user forgot to track something during normal operation), and calendar integration. (Crash/sleep gap *recovery* — reconciling the app's own tracking continuity, not inferring user activity — is in scope; see above.)
- Additional clients — CLI, browser extension, mobile app (architecturally anticipated via a shared data model, but not built now).
- Multi-user accounts, team/collaboration features, sync across users.
- PDF generation, timesheets, invoices, statistics/analytics views.
- Integrations with external tools (Jira, Harvest, Toggl, company-specific systems).

## Success criteria

Ties directly to `docs/vision/vision.md`'s "Success looks like": the author can run a real, full personal workday through Anchor using only hotkey/command-palette interactions, and at the end of the day export an XLSX/JSON timeline that accurately attributes every Time Block to its project/client — with zero lost or orphaned interruptions, and with any refinement of the day done inside Anchor rather than in Excel (revised 2026-07-28; previously "zero manual reconciliation," which the reconstruction workspace makes false as written — see `docs/vision/vision.md`). Billable-vs-non-billable classification itself happens downstream (see `docs/decisions/0003-billable-classification-out-of-scope.md`), not within Anchor.

This success claim is conditional on two things the MVP does not resolve, tracked in `docs/vision/vision.md` "Open questions" and `docs/risks.md`: (1) that manual-only tracking (risk R3) doesn't in practice let work go untracked through simple forgetfulness, and (2) that the interruption-stack mechanic is validated against the author's own real workflow only — not against whether existing tools (Toggl, Harvest, etc.) already solve this adequately, which remains unresearched.

---

**Keeping this current:** when a feature doc's status changes to `accepted`, check whether it belongs in "In scope" here. When scope grows, actively ask whether something else should move to "Out of scope" — MVP that only grows isn't an MVP.
