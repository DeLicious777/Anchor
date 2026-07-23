---
status: planned
date: 2026-07-24
owner: erich
related: [docs/product/features/interruption-stack.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/decisions/0002-desktop-app-framework-and-platform.md, docs/decisions/0004-transition-log-format-and-torn-write-scheme.md]
---

# Epic: Interruption Stack

The foundational MVP mechanic. Nothing else can be built before this — Task Templates and Export both consume the Time Block data this produces.

## Source docs

- Feature: [`docs/product/features/interruption-stack.md`](../../docs/product/features/interruption-stack.md)
- ADR 0001: manual-assisted tracking (no passive/automatic activity detection)
- ADR 0002: Tauri + Svelte + TypeScript + Ramda, Windows-first
- ADR 0004: JSON Lines transition log, checksum framing, watermark-based compaction

## Why this is first

Task Templates (epic below) needs the quick-input/Time-Block model this defines. Export (epic below) needs the Time Block data and durable storage this produces. No milestone after this one can start meaningfully before it ships.

## Scope (derived from the feature doc's Acceptance Criteria)

- Core Switch / Interrupt / Return-to-Previous / Return-to-Original / Complete mechanic, backed by a true nested stack
- Durable append-only transition log (JSON Lines, checksum-framed, per ADR 0004)
- Crash and sleep/hibernate gap recovery (`recovered-gap` completion reason), with the 60-second heartbeat bounding inference accuracy
- Compaction (snapshot + watermark-based replay, per ADR 0004)
- Global hotkeys (user-remappable, sane defaults, conflict detection where possible)
- Mini widget (always-on-top, current task + stack depth)
- Dashboard (timeline view — Task Templates and Export attach to this later, but the base dashboard shell belongs here)

## Not in this epic

Task Template management UI, and XLSX/JSON export — both are separate epics below, even though the dashboard shell they attach to is built here.

## Depends on

Nothing — this is the first epic.

## Blocks

Both epics below.
