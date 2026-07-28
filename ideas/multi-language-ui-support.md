# Multi-language UI support

Support additional UI languages beyond English — German as the first candidate.

## Decided (2026-07-28)

- **Driver is the author's own preference**, not anticipated distribution to other users. That sets the ceiling on what this justifies: a strings layer so the UI *can* be German, not a localization program (no pluralization frameworks, no translator workflow, no locale-aware formatting beyond what's already needed).
- **Sequenced last** of the four UI ideas. It's the only one with no user-visible benefit to a workflow that already works in English, and it multiplies the surface every other UI change has to touch.

## Still open

- **Does it stay author-only?** If distribution beyond the author is ever likely, the cost calculus flips — building the seam during the visual redesign is far cheaper than retrofitting it across a finished component set. Worth answering deliberately rather than by default.
- **Where does the string layer live, and is a library warranted at all?** Per the process gate in `CLAUDE.md`, this is an architecture decision and shouldn't be settled before the idea clears Design. Noting the candidates only: a plain strings module, `svelte-i18n`, `typesafe-i18n`. Global rules prefer standard-library/existing-dependency solutions first, and a single-user two-window app with one extra language is close to the floor of what justifies a dependency.
- **Is anything but UI chrome in scope?** Task names, project and client names are user-entered and must never be translated — worth stating explicitly in the doc rather than leaving as an obvious-to-everyone assumption. Export column headers are the genuine grey area: a German UI exporting English headers is defensible (downstream billing tools may expect them) but should be a decision, not an accident.
- **Do the auto-generated `Anchor N` names translate?** They're durably logged and exportable (`interruption-stack.md:89`), so translating them would put locale-dependent strings into the persisted record and into exports — almost certainly they should stay fixed. Same question for the three completion reasons, which are enum values on disk (`CompletionReason`, `app/src-tauri/src/model.rs:10`): the *display* label can translate, the stored value must not.

## Interacts with

- [`visual-redesign.md`](visual-redesign.md) — the widget is 260×90 and `resizable: false`. German strings run noticeably longer than English; if this is ever going to happen, the redesign should size the widget's text areas for the longest supported language rather than for English, since resizing it later isn't an option. **This is the one concrete thing this idea asks of the redesign even if translation itself is never built** — worth carrying over as a constraint regardless.
- Hotkey settings UI — action labels (`HotkeyAction::label()`) are the most obviously translatable strings in the app, and also the ones with the naming-collision problem noted in [`switch-between-mini-and-full-ui.md`](switch-between-mini-and-full-ui.md).
