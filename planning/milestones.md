# Milestones

_Last updated: 2026-07-24_

Sequencing reflects the dependency structure in `planning/epics/` — not arbitrary prioritization.

## M1 — Core Tracking Loop

**Epic:** [Interruption Stack](epics/interruption-stack.md)

Outcome: a full workday can be tracked end-to-end (Switch/Interrupt/Return, hotkeys, mini widget, dashboard shell) with durable, crash-safe storage. Nothing usable exists before this ships.

## M2 — MVP Complete

**Epics:** [Task Templates](epics/task-templates.md), [Export](epics/export.md)

Outcome: recurring activities are low-friction to track, and a day/range can be exported to XLSX/JSON in a billing-ready form. Both epics depend only on M1, not on each other — can proceed in parallel or either order.

Once M2 ships, the MVP (`docs/product/mvp.md`) is feature-complete per its "In scope" list.

## Later (not yet epics)

Per `docs/roadmap.md` "Later": cross-platform support, additional clients (CLI/browser/mobile), and anything currently in MVP's "Explicitly out of scope" list. These don't get epics until a future Discovery/Design pass revisits them.

---

**Keeping this current:** when an epic's status changes (e.g. `planned` → `in-progress` → `done`), update this file's outcome framing if the sequencing assumption changes — don't let this drift silently from `planning/epics/`.
