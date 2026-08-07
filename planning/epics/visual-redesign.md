---
status: planned
date: 2026-08-07
owner: erich
related: [docs/product/features/visual-redesign.md]
---

# Epic: Visual Redesign

The dashboard and widget get a coherent visual system and an information architecture ordered by frequency rather than by surface — the foundation every other visual epic in M3 builds on.

## Source docs

- Feature: [`visual-redesign.md`](../../docs/product/features/visual-redesign.md) — `accepted`

## Current implementation state

**Not started.** The dashboard is still the original two-tab split with a flat stack of sections. Two pieces of the accepted design ship incidentally: hotkey bindings already sit behind Settings, and an inferred end already renders in italic — provenance encoded without relying on colour.

## Remaining work

1. Import the design system: spacing scale steps, hue palette and its size, font weights. **External — see blockers.**
2. Rebuild the component layer in Svelte from the React JSX reference (**R13**(a) — an unpriced cost nobody has estimated).
3. Semantic tokens with two value sets; light and dark, user-selectable, persisted.
4. Two densities — `compact` for the widget, `comfortable` for the dashboard — with a 24×24 minimum target at `comfortable`.
5. The IA change: the Timeline becomes the dashboard's subject, export sits adjacent, template management joins hotkeys behind Settings.
6. Bundle three font families (**R13**(b) — against ADR 0002's binary-size reasoning).

## Dependencies and blockers

- **External, and the only true external blocker in M3: the design system is not in this repository.** The spacing scale's steps, the hue palette and its size, and the font weights cannot be reproduced without it.
- **Blocks** the Timeline Editor, the Interruption History surfaces, Pause's paused-state display, and therefore Timeline Reconstruction's remaining work.
- Independent of this epic, and deliberately not a sub-task of it: `tauri.conf.json` ships `"csp": null` (**R13**(c)), a live security posture that wants its own fix.

## Child issues

- #20 — already correctly titled; blocked on the assets, and the only M3 epic with no unblocked slice.
- **#26 — replace scaffold identity** (webview title, favicon, bundle icons, `productName`). Also asset-blocked, and **not** in M3a. `identifier: "com.erich.app"` is explicitly out of its scope: it drives `app_data_dir`, so changing it would orphan `transitions.jsonl` and `snapshot.json`.

## Exit criteria

`visual-redesign.md`'s Acceptance Criteria, notably: every element a pointer must hit is at least 24×24 at `comfortable`; the widget displays exactly four facts and nothing else; provenance stays distinguishable with colour removed, in both themes.

## Can it progress without the design-system assets?

**No.** This epic *is* the asset dependency.
