---
status: planned
date: 2026-07-24
owner: erich
related: [docs/product/features/task-templates.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Epic: Task Templates

Reduces entry-time friction and naming drift for recurring activities. Depends on the Interruption Stack epic's quick-input and dashboard shell.

## Source docs

- Feature: [`docs/product/features/task-templates.md`](../../docs/product/features/task-templates.md)
- ADR 0003: billable/non-billable classification is out of scope (relevant here since templates carry only name/project/client, no billable flag)

## Scope (derived from the feature doc's Acceptance Criteria)

- Dashboard section: create/edit/delete Task Templates (name, optional project, optional client)
- Quick-input autocomplete against existing templates (no dedicated per-template hotkeys — explicitly rejected in the feature doc)
- Durable persistence for templates, independent of the Time Block timeline
- Explicit non-retroactivity: editing a template never changes already-recorded Time Blocks

## Depends on

Interruption Stack epic (quick-input, dashboard shell, Time Block model).

## Blocks

Nothing directly, but reduces the practical severity of risk R2 (`docs/risks.md`) that the Export epic's XLSX row-grouping surfaces concretely — reasonable to sequence before or alongside Export, not strictly required first.
