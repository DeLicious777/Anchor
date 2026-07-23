# Architecture Overview

_Last updated: 2026-07-24_

> This describes **current-state** architecture only — the "why" behind each piece lives in its ADR, not here. If you're explaining a rationale in this doc instead of linking an ADR, that's a sign an ADR is missing.

## System summary

Anchor is a Tauri desktop application: a Rust core process handling OS-level concerns (global hotkeys, window management, sleep/hibernate detection, durable file I/O), paired with a Svelte + TypeScript (Ramda for functional composition) frontend rendering two windows — an always-on-top mini widget and a full dashboard — against shared backend state. Tracking is fully manual (no passive activity inference); every state transition is durably appended to a JSON Lines transition log with checksum-based torn-write detection and periodic snapshot-based compaction. Billable/non-billable classification is explicitly out of scope — Anchor attributes time to project/client only, and classification happens downstream in whatever billing process the author already uses. Target platform is Windows for MVP, with cross-platform (macOS/Linux) anticipated but not built.

## Component map

- **Rust core (Tauri backend)**: OS bridges (global hotkey registration via `tauri-plugin-global-shortcut`, tray, window management, sleep/hibernate detection via raw Windows API calls), the transition log's read/write/checksum/compaction logic, and the 60-second heartbeat timer.
- **Svelte/TypeScript frontend**: the always-on-top mini widget (current task, stack depth) and the dashboard (timeline view, Task Template management, export), both reading from the same backend state via IPC — no divergent in-memory caches.
- **Persisted state (on disk)**: an append-only JSON Lines transition log (one file per user), periodically compacted into a snapshot + truncated log, both keyed by a monotonic per-line sequence number for crash-safe watermark-based replay.
- **No external services**: no calendar integration, no cloud sync, no accounts — everything is local to the single Windows machine for MVP.

## Key decisions

| Component/Area | Decision | ADR |
|---|---|---|
| Tracking method | Manual, assisted tracking only — no passive/automatic activity detection; gap-recovery timestamps are inferred, but never activity content | [ADR 0001](../decisions/0001-manual-assisted-tracking-for-mvp.md) |
| App framework & platform | Tauri (Rust core + native OS webview) + Svelte/TypeScript/Ramda frontend, Windows-first for MVP | [ADR 0002](../decisions/0002-desktop-app-framework-and-platform.md) |
| Data model scope | Billable/non-billable classification is out of scope — Anchor attributes project/client only; classification happens downstream | [ADR 0003](../decisions/0003-billable-classification-out-of-scope.md) |
| Transition log format | JSON Lines, checksum framed outside the JSON object (avoids a self-reference bug caught in review), watermark-based compaction surviving a crash at any point | [ADR 0004](../decisions/0004-transition-log-format-and-torn-write-scheme.md) |

---

**Keeping this current:** every accepted ADR that changes system structure should add or update a row here in the same turn it's accepted — not as a later cleanup pass.
