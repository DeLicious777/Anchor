# Roadmap

_Last updated: 2026-07-24_

Sequencing of outcomes, not a task list. Tied to `docs/product/mvp.md` and, once Planning starts, to `planning/epics/` and `planning/milestones.md`.

## Now

**All three MVP features and four ADRs are `status: accepted`**: `interruption-stack.md`, `task-templates.md`, `export.md`; ADR 0001 (manual tracking), 0002 (Tauri/Svelte platform), 0003 (billable classification out of scope), and **0004** (transition log format: JSON Lines, checksum framed outside the JSON object to avoid a self-reference bug, watermark-based compaction that survived two review rounds — the first found the checksum/compaction gaps, a follow-up found heartbeats had been left unable to satisfy the watermark filter they need to pass). Risk register has 7 entries; `docs/architecture/constraints.md` has its first entry (frontend stack).

## Next

- Planning workflow (`.claude/workflows/planning.md`): group the three accepted feature docs and four ADRs into epics/milestones, now that there's enough accepted surface to plan against.

## Later

Cross-platform (macOS/Linux) support, and any client beyond desktop — sequenced after the Windows MVP is built and real usage validates the core mechanic (see `docs/assumptions.md`). Actual implementation only begins once Planning is done and the Definition of Ready (`docs/README.md`) is fully met.

---

**Keeping this current:** when an epic lands on `planning/milestones.md`, reflect it here in outcome terms (what becomes true for users), not implementation terms.
