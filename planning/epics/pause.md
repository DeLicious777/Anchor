---
status: planned
date: 2026-08-07
owner: erich
related: [docs/product/features/pause.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md]
---

# Epic: Pause

Stopping work becomes expressible without falsifying the record. Today `Complete` is rejected while the interruption stack is non-empty, so a user three interruptions deep has no legal way to stop — every alternative corrupts the timeline.

## Source docs

- Feature: [`pause.md`](../../docs/product/features/pause.md) — `accepted` 2026-08-07
- [ADR 0005](../../docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md) — Pause is a specialised Interrupt with no successor

## Current implementation state

**Not started.** `TransitionPayload` has no Pause variant; no command exists.

**Verified already present, and therefore NOT in this epic's scope:**

- Both return arms already accept no active task — `close_active_if_any` is a no-op when nothing is active. `mvp.md:24` claimed otherwise and was retracted 2026-08-07.
- A `start` command already exists (`commands.rs:179`, registered `lib.rs:142`).
- `gap::resolve` already returns `Continue` with nothing active, so no gap transition fires while paused.
- `heartbeat::should_beat` already gates on `active.is_some()`, so heartbeats stop by themselves.

## Remaining work

1. `TransitionPayload::Pause` and its `apply` arm — close active `UserDetermined`, push its frame, leave `active == None`. Reuse the `Interrupt` arm's frame construction so the two cannot drift.
2. A `pause` Tauri command, thin like the rest.
3. Replay and snapshot tests: `log::reader` calls `apply` directly with no dry-run guard.
4. A sixth `HotkeyBindings` field, applied atomically by `hotkeys::apply_remap`. **A durable settings-shape change** — a five-field file must still load. *(Asset-blocked by association — see below.)*
5. Paused-state display on dashboard and widget, and the `Continue` relabel. *(Asset-blocked.)*

## Dependencies and blockers

- **No design dependency.** ADR 0005 settled the event model; the feature doc settled the rest.
- **Items 1–3 are unblocked today.** Items 4 and 5 ship together in M3c.
- **Item 4 cannot ship before item 5, and that is an architectural fact rather than a preference.** *(Established 2026-08-07.)* `hotkeys::register_bindings` iterates the actions and calls `global_shortcut().register` on each unconditionally — there is no enabled flag, no skip list, no dormant registration. Adding a `HotkeyAction` variant makes the accelerator **live on the day it ships**, and a live Pause key with no visible paused state realises risk **R19**, whose only mitigation (assumption **A18**) is the very display in item 5. Building a disabled-registration mechanism to decouple them would be new architecture, which a planning pass does not get to introduce.

## Child issues

- **#23 — Pause transition, command and tests.** Unblocked; items 1–3 above.
- #16 — the epic issue, retitled 2026-08-07. Retains items 4 and 5 — the binding and the display, which ship together.

## Exit criteria

`pause.md`'s Acceptance Criteria, notably: pausing twice is rejected; restart, crash and sleep while paused append **no** transition at any elapsed time; no heartbeat is written during a break; `Complete` becomes available once the stack empties.

## Can it progress without the design-system assets?

**Yes — items 1 through 3**, the transition, the command and their tests. The binding and the display are one unit and wait for M3c.
