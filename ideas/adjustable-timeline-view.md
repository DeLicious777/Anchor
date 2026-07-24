# Adjustable timeline view

An optional timeline visualization of the day's Time Blocks (in addition to the existing table view), orientation (vertical or horizontal) chosen by the user, toggled on/off.

Raw idea, not yet scoped. Open questions for whenever this goes through Discovery/Design:
- Where does this live — the dashboard's existing timeline section, or a new view/window?
- What does "activated" mean concretely — a per-session toggle, a persisted setting (like the hotkey bindings in `settings.json`), both?
- How does it render `recovered-gap` / `auto-completed-on-skip` entries and the interruption stack's nesting — is nesting visually represented at all, or is this a flat chronological timeline?
- Interaction with export (`docs/product/features/export.md`) — purely a read-only visualization, or does it inform how a range gets selected for export?
