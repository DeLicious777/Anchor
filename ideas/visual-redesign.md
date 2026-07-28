# Visual redesign — layout, components, and color

The current dashboard (`app/src/routes/+page.svelte`, 739 lines) is functional debug-grade UI: plain sections, default form controls, no real visual identity or color system. This idea is a proper design pass — a cohesive component set and color palette applied across the dashboard and the mini widget, not just a CSS touch-up of the existing markup.

## Decided (2026-07-28)

- **Scope is styling *plus* information architecture** — section order, grouping, and what's visible vs. tucked away are all in play, not just a restyle of the existing structure.
- **Light and dark themes, user-selectable.** Persisted like the existing hotkey bindings and export settings (`settings.json`, via the `HotkeyBindings::load` / `update_export_settings` precedent) — a stored preference, not an OS-follow.
- **This idea sequences first**, ahead of the mini/full switch and the timeline view. All three touch `+page.svelte` and `widget/+page.svelte`; designing the other two against today's debug UI means redesigning them twice.

## Still open

- Does the widget get its own visual language, or a constrained subset of the dashboard's component set? It's 260×90, `decorations: false`, `alwaysOnTop` — it can't host the same components at the same density, but "distinct language" and "same system, fewer parts" are different design commitments.
- Any existing brand/color preference, or fully open? Nothing in the repo expresses one.
- Does dark mode change what the widget does visually when it sits over other apps all day (opacity, borderless edge definition given `shadow: false`)?
- Does "user-selectable" mean a control in the dashboard only, or also reachable from the widget?
- The dashboard's default window is 800×600 (`app/src-tauri/tauri.conf.json`). Is that still the right default after an IA pass, or does the redesign want more room?

## Interacts with

- [`switch-between-mini-and-full-ui.md`](switch-between-mini-and-full-ui.md) — a true mode switch changes what each window is *for*, which is an IA input to this redesign, not a consequence of it.
- [`adjustable-timeline-view.md`](adjustable-timeline-view.md) — placing a timeline beside the existing table is an IA decision that belongs to this pass.
- Theme persistence lands in the same `settings.json` surface as export settings and hotkey bindings — worth designing as one "settings" area rather than a third independent store.
