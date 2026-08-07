# Roadmap

_Last updated: 2026-08-07_

Sequencing of outcomes, not a task list. Tied to `docs/product/mvp.md` and, once Planning starts, to `planning/epics/` and `planning/milestones.md`.

## 2026-07-28 — Discovery revision (historical)

*Kept as the record of that phase. The current state is under "Now" below.*

**Discovery reopened, ran, and closed (2026-07-28).** A Concept revision established Anchor as *capture-first, timeline-assisted*: the interruption model is the product, the Timeline is the interface. `vision.md`, `concept.md`, and **ADR 0005** are `status: accepted`, after a `grill-with-docs` session and three independent review passes (15 → 14 → 12 findings, each round's fixes generating the next round's) plus the author's own review of the full diff. Discovery's exit criteria are met.

**Planning already ran, and is now stale.** Correcting an error this doc carried both before and after the revision: it listed Planning as "Next" when Planning had in fact completed on 2026-07-23/24 — `planning/epics/` holds three epics, `planning/milestones.md` defines M1/M2, and there are 13 GitHub issues. Implementation then ran through two slices. The 2026-07-28 Discovery revision invalidated parts of all of it.

So the next phase is **alignment, not planning**: bring the existing epics, issues and milestones into sync with the accepted design, without re-planning. New epics for the four newly-scoped features must wait for their feature docs, per `CLAUDE.md`'s gate.

What changed: two Vision success criteria were absolute in a way the product isn't and are now measurable (Capture Rate ≥90%, Capture Latency ≤1s, both provisional with revisit triggers). The differentiator claim was reframed from market gap to **build-vs-buy** after research found ManicTime and Memtime already shipping local-first timeline + billing export — the 2026-07-23 "existing tools don't solve this" assumption is **invalidated** (`docs/assumptions.md`), the first invalidation from evidence rather than a design change, with a narrower replacement dated 2026-07-28. `completion_reason` split into three orthogonal fields. Timeline reconstruction, the Timeline Editor, Pause/Continue Session, and the interruption-history panel entered MVP scope, with the scope trade deliberately deferred to Planning. Split and merge were considered and removed.

Still accepted, with amendment notes rather than reopened: `interruption-stack.md` and `export.md` (both gained "pending revision" blockquotes pointing at ADR 0005), `task-templates.md`, and ADRs 0001–0004 — ADR 0004 now carries an amendment blockquote, since ADR 0005 deliberately breaks its on-disk contract once and adds the snapshot-payload guarantee it never specified. New: **ADR 0005** (event model), `accepted` with nine open implementation-level items; `docs/principles.md` (7 principles); a fourth feature doc, `visual-redesign.md`, unblocked but not yet past Alternatives. Risk register is at **13** entries (was 8) — R9–R13 added, R1/R3/R4 amended. `docs/architecture/constraints.md` has a second entry: the event log is the single source of truth.

## Now (2026-08-07)

**Design is finished, and Planning's second pass has run.** Everything the "Next" list below called for has happened, except one item that is now overdue; that list is kept as the record of what was outstanding on 2026-07-28 rather than deleted.

- **All eight feature docs are `accepted`** — the four that entered MVP scope on 2026-07-28 (timeline reconstruction, Timeline Editor, Pause, interruption history) plus `visual-redesign.md`, and the three from M1/M2. Every one has an epic in `planning/epics/`.
- **M3 — Editable Timeline is defined** (`planning/milestones.md`), split by *blocker* rather than by feature: **M3a** backend integration (unblocked, and **not releasable**), **M3b** Visual Redesign (gated on design-system assets that are not in this repository), **M3c** the surfaces.
- **Two ADRs were added since:** 0006 (stable Time Block identity) and 0007 (auto-resume after a short gap), taking the set to **seven accepted** (0001–0007), plus the 0000 template.
- The registers grew with the work: risks at **19** (was 13), assumptions at **18**. **R9 closed on 2026-08-06** — a wrongly inferred end is correctable at last, via the History View's edit row.

**What becomes true for the user next:** nothing, until the design-system assets arrive. M3a hardens three event-model changes behind the scenes; every user-visible improvement in M3 sits behind M3b. That is stated plainly because it is the single most important scheduling fact about this milestone.

## Next

**Outstanding from the list below, and now the oldest open item:** **Graphify regeneration.** Its trigger — "once the architecture and feature docs settle" — has fired: a phase completed, eight feature docs and eight ADRs are accepted, and the glossary has grown substantially. `graphify-out/` is committed and currently reflects a repository that no longer exists.

**The order is: Graphify regeneration, then M3a (#24 → #23 → #25), then the design-system assets, M3b, M3c, verification.** Graphify goes first because it is the artifact other tooling reads as current, and starting implementation before regenerating it would bake a stale snapshot into that role.

Note M3a's scope narrowed on 2026-08-07: **Pause's sixth hotkey binding is not in it.** `hotkeys::register_bindings` registers every action unconditionally with no dormant path, so a binding cannot be landed inert — and a live Pause key with no visible paused state realises risk **R19**. The binding ships in M3c with the display.

---

### The 2026-07-28 list, kept as a record

1. **Resolve ADR 0005's nine open items** — log-order vs. chronological-order, overlap rules, editing blocks bound to live stack frames, `Rename` vs. edit identity, enum wire formats, **the two `export.md` defects it inherited** (the log/timeline conflation, and the "unchanged byte-for-byte after any export" criterion that is false whenever a heartbeat appends mid-export), and restating `interruption-stack.md`'s acceptance criteria.
2. **Feature docs for the four items that entered MVP scope on 2026-07-28** — timeline reconstruction, Timeline Editor, Pause/Continue Session, and the interruption-history panel. **None currently has one**, and `CLAUDE.md`'s gate is explicit: no epics or Project work before the feature docs and ADRs they depend on are `accepted`. The interruption-history panel is the most exposed — it acquired a user action (dismissing a frame) that permanently alters the billing record, via a header blockquote in an `accepted` doc, with no Alternatives, UX, or acceptance criteria.
3. **Design workflow for `visual-redesign.md`** — enabling work for the Timeline Editor, so it precedes it.
4. **Graphify regeneration** — once the architecture and feature docs settle, so Planning gets an accurate graph rather than one that goes stale immediately.
5. **Re-open Planning** for the newly-scoped work (`.claude/workflows/planning.md`) — only after all of the above. It inherits the deferred MVP scope trade and must actually draw the v1.0/v1.1 line, or the scope grew with nothing given back. Note this is a *second* Planning pass, not the first; the existing epics and milestones are the input, not a blank page.

**Status of that list as of 2026-08-07:** item 2 **done** — all four feature docs accepted, between 2026-08-01 and 2026-08-07. Item 3 **done** — `visual-redesign.md` accepted, and it did precede the Timeline Editor as intended. Item 5 **done** — Planning's second pass ran on 2026-08-07 and produced M3; it did inherit the existing epics rather than starting blank, and it did draw a line, though by *blocker* rather than by v1.0/v1.1. Item 1 **done** — ADR 0005 records that **all nine open items were resolved on 2026-08-01**: items 5–9 during implementation and follow-up passes, items 1–4 by `timeline-reconstruction.md`. The two `export.md` defects it named were fixed on 2026-07-29 and 2026-08-06. *(An earlier version of this line claimed the remainder was unaudited; that was wrong, and the ADR says so at its head.)* Item 4 **outstanding** — see above.

Step 0, before any of it: **align the existing planning artifacts** with the accepted design. Alignment only — "is this still valid, is this vocabulary current, does this issue still solve the same problem" — not a re-planning exercise, and no new epics for features that don't yet have feature docs.

## Later

Cross-platform (macOS/Linux) support, and any client beyond desktop — sequenced after the Windows MVP is built and real usage validates the core mechanic (see `docs/assumptions.md`). Actual implementation only begins once Planning is done and the Definition of Ready (`docs/README.md`) is fully met.

---

**Keeping this current:** when an epic lands on `planning/milestones.md`, reflect it here in outcome terms (what becomes true for users), not implementation terms.
