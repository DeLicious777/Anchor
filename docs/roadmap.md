# Roadmap

_Last updated: 2026-07-28_

Sequencing of outcomes, not a task list. Tied to `docs/product/mvp.md` and, once Planning starts, to `planning/epics/` and `planning/milestones.md`.

## Now

**Discovery reopened and is mid-flight.** A Concept revision (2026-07-28) established Anchor as *capture-first, timeline-assisted*: the interruption model is the product, the Timeline is the interface. `vision.md` and `concept.md` are back to `status: draft` pending a verification pass; **Planning cannot start until they return to `accepted`.**

What changed: two Vision success criteria were absolute in a way the product isn't and are now measurable (Capture Rate ≥90%, Capture Latency ≤1s, both provisional with revisit triggers). The differentiator claim was reframed from market gap to **build-vs-buy** after research found ManicTime and Memtime already shipping local-first timeline + billing export — the 2026-07-23 "existing tools don't solve this" assumption is **invalidated** (`docs/assumptions.md`), the first invalidation from evidence rather than a design change, with a narrower replacement dated 2026-07-28. `completion_reason` split into three orthogonal fields. Timeline reconstruction, the Timeline Editor, Pause/Continue Session, and the interruption-history panel entered MVP scope, with the scope trade deliberately deferred to Planning. Split and merge were considered and removed.

Still accepted, with amendment notes rather than reopened: `interruption-stack.md` and `export.md` (both gained "pending revision" blockquotes pointing at ADR 0005), `task-templates.md`, and ADRs 0001–0004 — ADR 0004 now carries an amendment blockquote, since ADR 0005 deliberately breaks its on-disk contract once and adds the snapshot-payload guarantee it never specified. New: **ADR 0005** (event model) at `status: draft`; `docs/principles.md` (7 principles); a fourth feature doc, `visual-redesign.md`, unblocked but not yet past Alternatives. Risk register is at **13** entries (was 8) — R9–R13 added, R1/R3/R4 amended. `docs/architecture/constraints.md` has a second entry: the event log is the single source of truth.

## Next

1. **Third verification pass**, then flip `vision.md`, `concept.md`, and ADR 0005 to `accepted`. The second pass (2026-07-28) returned "not ready" and found real new problems created by the first round of fixes — including Pause relocating rather than closing the hole it was admitted for. Remediated; not yet re-verified.
2. **Resolve ADR 0005's remaining open items** — log-order vs. chronological-order, overlap rules, editing blocks bound to live stack frames, `Rename` vs. edit identity, enum wire formats, **the two `export.md` defects it inherited** (the log/timeline conflation, and the "unchanged byte-for-byte after any export" criterion that is false whenever a heartbeat appends mid-export), and restating `interruption-stack.md`'s acceptance criteria.
3. **Feature docs for the four items that entered MVP scope on 2026-07-28** — timeline reconstruction, Timeline Editor, Pause/Continue Session, and the interruption-history panel. **None currently has one**, and `CLAUDE.md`'s gate is explicit: no epics or Project work before the feature docs and ADRs they depend on are `accepted`. The interruption-history panel is the most exposed — it acquired a user action (dismissing a frame) that permanently alters the billing record, via a header blockquote in an `accepted` doc, with no Alternatives, UX, or acceptance criteria.
4. **Design workflow for `visual-redesign.md`** — enabling work for the Timeline Editor, so it precedes it.
5. **Planning workflow** (`.claude/workflows/planning.md`) — only after all of the above. It inherits the deferred MVP scope trade and must actually draw the v1.0/v1.1 line, or the scope grew with nothing given back.

## Later

Cross-platform (macOS/Linux) support, and any client beyond desktop — sequenced after the Windows MVP is built and real usage validates the core mechanic (see `docs/assumptions.md`). Actual implementation only begins once Planning is done and the Definition of Ready (`docs/README.md`) is fully met.

---

**Keeping this current:** when an epic lands on `planning/milestones.md`, reflect it here in outcome terms (what becomes true for users), not implementation terms.
