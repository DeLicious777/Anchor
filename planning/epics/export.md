---
status: planned
date: 2026-07-24
owner: erich
related: [docs/product/features/export.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Epic: Export (XLSX / JSON)

Turns the tracked timeline into billing-usable output. Depends on the Interruption Stack epic's Time Block data and dashboard shell.

## Source docs

- Feature: [`docs/product/features/export.md`](../../docs/product/features/export.md)
- ADR 0003: billable/non-billable classification is out of scope (export attributes project/client only; classification happens downstream)

## Scope (derived from the feature doc's Acceptance Criteria)

- Date-range picker (Today default, This Week, custom) in the dashboard
- XLSX export: single flat worksheet, one row per unique task (name/project/client), summed-then-rounded duration
- JSON export: raw per-Time-Block when rounding is off; grouped-by-task (same shape as XLSX) when rounding is on
- Rounding control (on/off toggle, configurable interval, ceiling rounding), persisted as a user setting
- Still-active-entry handling (elapsed-so-far duration included, live computation, never written back to storage)

## Depends on

Interruption Stack epic (Time Block data, dashboard shell, durable storage). Not functionally dependent on Task Templates, though both improve billing-data quality together.

## Blocks

Nothing — this is the last epic needed for MVP completeness.
