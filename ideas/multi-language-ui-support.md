# Multi-language UI support

Support additional UI languages beyond English — German mentioned as the first candidate.

Raw idea, not yet scoped. Open questions for whenever this goes through Discovery/Design:
- Which languages, in what order, and is this driven by the author's own need or eventual other users?
- Where does translated text live — a strings file, an i18n library (e.g. `svelte-i18n`, `typesafe-i18n`), something else?
- Does this touch just the dashboard/widget UI text, or also anything user-entered (task names) — almost certainly not the latter, but worth confirming explicitly rather than assuming.
- Interacts with the mini-widget's small footprint — some languages produce longer strings than English; the widget's fixed 260×90 size (see `docs/product/features/interruption-stack.md`) may need to accommodate that.
