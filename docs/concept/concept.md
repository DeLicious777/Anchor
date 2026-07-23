---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/product/mvp.md, docs/risks.md, docs/glossary.md]
---

# Concept

## Elevator pitch

Anchor is a desktop time-tracking app built around an interruption stack: when a call, message, or spontaneous task pulls you away from what you're doing, you push it in one hotkey press, and you can always find your way back to exactly where you left off — with every minute of the day accounted for.

## Core concept

- A desktop app, always available via user-remappable global hotkeys, an always-on-top mini widget (current task, stack depth), and a full dashboard for history/templates/export — not a browser tab you have to remember to switch to. See `docs/product/features/interruption-stack.md` for the interaction design.
- The atomic unit of tracked work is a **Time Block**: a name, start time, end time, duration, optional project/client, and a **completion reason** (`explicit`, `auto-completed-on-skip`, or `recovered-gap`) — see below and `docs/glossary.md`.
- Two distinct operations for changing what's being tracked:
  - **Switch** — deliberate move to a new task, no expectation of returning.
  - **Interrupt** — the current task is paused for something urgent, pushed onto an internal interruption stack, with intent to return.
- On completing an interruption, two explicit return paths: **Return to Previous Task** (step back one level) or **Return to Original Task** (jump straight to the root). Choosing the latter marks every skipped intermediate Time Block completed with reason `auto-completed-on-skip` — distinct from a Time Block the user explicitly completed. This is what keeps the mechanic from contradicting Vision's "never silently dropped" success criterion: nothing is *silent*, because the record permanently shows it was skipped, not finished on its own merits, and remains reviewable/reopenable later if it turns out to have been genuinely unfinished.
- The interruption stack is a true, arbitrarily deep stack internally, but the UI only ever surfaces the current task and the two return options — the complexity is structural, not something the user has to think about.
- **Task Templates** let recurring activities (daily standup, sprint retro, a specific client's work) start with one action instead of re-entering name/project/client each time.
- The **timeline** (the ordered sequence of time blocks) is the single source of truth. Reports, exports, and any future integration are all derived views over it — capture never changes to serve a downstream format.

## How it's different

Existing time trackers (Toggl, Harvest, Clockify) center on a single active timer you start/stop/switch manually, with no first-class model for "I was interrupted, and I need to get back to exactly where I was." Passive tools (RescueTime) infer activity automatically, trading determinism and privacy for less manual effort. Anchor's bet is that the interruption-stack mechanic — not just another timer UI — is the actual missing piece for people whose real workday is a stack of nested contexts, not a single line of tasks.

This is currently an untested assumption, not validated competitive research — see `docs/assumptions.md`. Because of this, no claim in this project should treat the interruption-stack mechanic as a *proven* differentiator — MVP success (`docs/product/mvp.md`) is validated against the author's own workflow only, not against competing tools.

## Key assumptions

See `docs/assumptions.md` for the full, current log — not restated here, to avoid the two drifting out of sync. The rows most load-bearing for this Concept are the ones on manual tracking vs. "no work forgotten," the auto-complete mechanic, and export-time aggregation.

## Open questions

- Whether the interruption-stack mechanic is a genuine differentiator or something existing tools already handle adequately — needs research before this concept is considered validated.

---

**Keeping this current:** if the MVP (`docs/product/mvp.md`) or a feature doc no longer matches this concept, update whichever is wrong — don't let the concept become aspirational fiction.
