# Visual redesign — layout, components, and color

The current dashboard (`app/src/routes/+page.svelte`) is functional debug-grade UI: plain sections, default form controls, no real visual identity or color system. This idea is a proper design pass — a cohesive component set and color palette applied across the dashboard and the mini widget, not just a CSS touch-up of the existing markup.

Raw idea, not yet scoped. Open questions for whenever this goes through Discovery/Design:
- Is this a redesign of the existing dashboard's information architecture too (section order, grouping, what's visible vs. tucked away), or purely a visual/styling pass over the current structure?
- Does the mini widget (`app/src/routes/widget/+page.svelte`) get its own distinct visual language (it's deliberately minimal/always-on-top) or should it share the dashboard's new component system exactly?
- Light/dark mode, or a single fixed theme?
- Any existing brand/color preference from the author, or fully open?
- Does this block or interleave with the other logged UI ideas — mini/full UI switch (`switch-between-mini-and-full-ui.md`), timeline view (`adjustable-timeline-view.md`) — since all three touch the same surfaces?
