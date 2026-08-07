# Adjustable timeline view

> **Promoted to design 2026-08-04.** This idea now has a feature document — [`docs/product/features/timeline-editor.md`](../docs/product/features/timeline-editor.md), currently `status: draft` — which is where its decisions live from here. Four of the six "Still open" questions below are settled there (orientation, the toggle, time range, short blocks); the remaining two are carried into that doc's open Alternatives. This file stays as the inbox record of how the idea started, per `CLAUDE.md` — it is not held to feature-doc rigour and should not be updated to track the design.

A timeline visualization of the day's Time Blocks, shown **beside** the dashboard's existing table rather than replacing it — the table stays the detail view, the timeline gives shape to the day at a glance.

Note: `docs/product/features/interruption-stack.md` already calls the dashboard's table "the timeline view" and has an accepted acceptance criterion on it (line 102). This idea adds a *second representation of the same data*, so the two need distinct names in the docs to stay unambiguous.

## Decided (2026-07-28)

- **Both visible, side by side.** Not a toggle between table and timeline, and not a replacement — they coexist horizontally on the dashboard.
- **Flat chronological.** Every Time Block is a peer in start order; interruption nesting is *not* represented visually. This matches the accepted flat timeline data model (`docs/product/mvp.md`: "each time block is an independent entry"), so nothing has to be reconstructed to render it.
- **Scope split**: the drag-a-span-to-add-an-entry capability originally filed here is a different feature and now lives in [`manual-time-block-entry.md`](manual-time-block-entry.md). This idea is the *visualization*; that one is *timeline editing*. They ship in that order.

## Still open

- **Does user-selectable orientation still earn its place?** The original idea said vertical or horizontal, user's choice. "Beside the table" largely forces vertical — a horizontal timeline in a narrow column next to a 5-column table doesn't work, and the dashboard's default window is only 800×600. Two rendering modes plus a persisted setting is real cost; the layout decision may have already made it moot. Recommend: pick vertical, drop the setting, revisit if it actually chafes.
- **Is the timeline still "toggled on/off"** as an optional element, now that it's a permanent side-by-side companion rather than an alternate view? If yes, that's another persisted setting.
- **How are the three completion reasons rendered** — `explicit`, `auto-completed-on-skip`, `recovered-gap` (see `docs/glossary.md`)? `recovered-gap` in particular is supposed to be "surfaced distinctly" per the accepted feature doc; a timeline is arguably a *better* place to spot a wrong inferred end time than a table row is, since a bad duration shows up as a visibly wrong-sized block. That's a genuine argument for this feature against risk R4, worth making explicitly if it goes to Design.
- **What happens to very short blocks?** A day with many short interruptions produces slivers that may be unreadable or unclickable at any sane scale. Minimum rendered size, or zoom?
- **What time range does it cover** — strictly the current day, matching the idea's original framing, or does it follow whatever range the dashboard is showing?
- **Does the still-active (in-progress) block render live and grow?** `export.md` already establishes elapsed-so-far as a live, never-persisted computation; the same idea applies here.

## Interacts with

- [`visual-redesign.md`](visual-redesign.md) — "beside the table" is an information-architecture decision, so this should be designed as part of the redesign's IA pass, not bolted on after it.
- [`manual-time-block-entry.md`](manual-time-block-entry.md) — the split-out editing capability. If that one is going to happen, the timeline's rendering (hit targets, minimum block size, scale) needs to be usable as an *input* surface, not only a display. Worth knowing before finalizing this one's visual design.
