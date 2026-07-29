---
status: draft
date: 2026-07-28
owner: erich
related: [docs/vision/vision.md, docs/product/mvp.md, docs/risks.md, docs/glossary.md, docs/product/features/interruption-stack.md, ideas/manual-time-block-entry.md]
---

# Concept

> **Under revision (2026-07-28), was `accepted` since 2026-07-23.** The Concept revision session established the product as **capture-first, timeline-assisted**: the interruption model is the product, the timeline is the interface. Returns to `accepted` only after the Discovery workflow's Stage 5 reviewer pass.

## Elevator pitch

Anchor is a desktop time-tracking app built around an interruption stack: when a call, message, or spontaneous task pulls you away from what you're doing, you push it in one hotkey press, and you can always find your way back to exactly where you left off — with every minute of the day accounted for. At the end of the day, the timeline shows you what actually happened and lets you fix what the capture missed.

## Core concept

- A desktop app, always available via user-remappable global hotkeys, an always-on-top mini widget (current task, stack depth), and a full dashboard for history/templates/export — not a browser tab you have to remember to switch to. See `docs/product/features/interruption-stack.md` for the interaction design.
- The atomic unit of tracked work is a **Time Block**: a name, start time, end time, duration, optional project/client, and **three independent metadata fields** — see `docs/glossary.md` and the Event Model ADR:
  - **End Determination** — how was this block's end time established? (`UserDetermined` / `SystemInferred`)
  - **Capture Origin** — how did this block enter the system, and how much do we trust it? (captured live vs. reconstructed afterwards, and whether it has since been adjusted)
  - **Interruption Outcome** — what ultimately happened to this interrupted work? (absent / `Resumed` / `Skipped`)

  These replace the single `completion_reason` field (revised 2026-07-28), which conflated all three questions. `auto-completed-on-skip` was the clearest symptom: it described the fate of a stack frame, not a reason a block ended. Each field now answers exactly one question — see [`principles.md`](../principles.md) #5.
- An **interruption stack frame** is an *unresolved obligation to record the outcome of interrupted work* — it carries both the return path and the only link back to the block whose fate is undetermined. That definition is why Return to Original Task stamps every frame it pops rather than discarding them, and why dismissing a frame is a historical statement rather than deleting a reminder.
- **Pause is a specialised Interrupt: it creates an interruption frame without creating a successor task.** It closes the active block, pushes its frame, and leaves nothing active. Paused work and interrupted work are deliberately the same state — *stopped, with the intention of returning* — because whether the cause was a phone call, lunch, or the end of the day doesn't change the work's lifecycle. The paused task is simply the top frame, so Return to Previous lands on it. **Continue Session** is a UI action, not a transition: after Pause the state is already correct, and after a restart replay rebuilds it from the persisted stack. Named that way because it continues the *tracking session*, never a task — Anchor must not claim a start time it cannot know. This closes a hole rather than adding a feature: `Complete` is rejected while the stack is non-empty, so stopping mid-stack previously required leaving the clock running or fabricating returns — see [`principles.md`](../principles.md) #3.
- **`active == None` with a non-empty stack is a legal state**, not an error. The state machine must permit Return to Previous and Return to Original from it — popping the frame, starting the returned task, resolving its outcome, with no active block to close. Without this, Pause would merely relocate the hole it was admitted to close.
- **Five actions, five distinct intents**, each with its own precondition rather than one command changing meaning by state: **Start** (nothing active → begin), **Switch** (active → stop this, begin something else), **Interrupt** (active → stop this, begin something else, intend to return), **Pause** (active → stop this, begin nothing, intend to return), **Return** (≥1 frame → resume interrupted work). `Start` is newly first-class: the transition exists in the model but no command exposes it today. The UI stays simple by being context-sensitive — Start where nothing is active, Switch where something is.
- Two distinct operations for changing what's being tracked:
  - **Switch** — deliberate move to a new task, no expectation of returning.
  - **Interrupt** — the current task is paused for something urgent, pushed onto an internal interruption stack, with intent to return.
- On completing an interruption, two explicit return paths: **Return to Previous Task** (step back one level) or **Return to Original Task** (jump straight to the root). Choosing the latter marks every skipped intermediate Time Block `InterruptionOutcome::Skipped` (revised 2026-07-28; previously `auto-completed-on-skip`) — distinct from work the user actually resumed. This is what keeps the mechanic from contradicting Vision's "never silently dropped" success criterion: nothing is *silent*, because the record permanently shows it was skipped, not finished on its own merits, and remains reviewable/reopenable later if it turns out to have been genuinely unfinished.
- The interruption stack is a true, arbitrarily deep stack internally. **By default the UI surfaces only the current task and the two return options** — the complexity stays structural, not something the user has to think about. The full stack is available behind **progressive disclosure** (an optional "interruption history" panel), so it can be inspected for confidence that nothing was lost, without competing for attention in the default view. Inspecting the stack does not add a third return path: Return to Previous and Return to Original remain the only actions (revised 2026-07-28).
- The **Timeline is the primary visualization and a reconstruction workspace**, not a passive log. It has two views over the same data — the **Timeline Editor** (graphical, direct-manipulation) and the **History View** (tabular) — plus the interruption-history panel behind progressive disclosure. They are views of one thing, not separate features: the user thinks *"I'm looking at today's timeline,"* not *"I'm switching between tools,"* and an edit in one is immediately reflected in the other.
- The MVP edit surface is exactly five operations: **add, move, resize, edit identity, delete**. Every one has a stated problem — forgotten work, incorrect placement, incorrect timing, incorrect attribution, mistakes. **Split and merge were considered and removed** (2026-07-28): merge duplicates what export already does while introducing adjacency and provenance-laundering problems, and split is reachable via resize plus add. See [`principles.md`](../principles.md) #1.
- Every block always represents work that actually happened. The Timeline answers *"what happened today?"*, never *"what should happen next?"*; planning belongs in a calendar or task manager, and Anchor has no future-dated or intended blocks (decided 2026-07-28).
- **Capture-first, timeline-assisted.** Primary capture is instant — sub-second, via hotkey or the mini widget, without opening the app — because interruptions arrive while the user is looking at something else entirely. Manual timeline entry exists for work that went untracked or needs reconstructing; it is the exception, not the default path.
- **Task Templates** let recurring activities (daily standup, sprint retro, a specific client's work) start with one action instead of re-entering name/project/client each time.
- **The event log is the single source of truth; the timeline is a projection of it** (corrected 2026-07-28 — this doc previously named the timeline itself, which contradicted [ADR 0004](../decisions/0004-transition-log-format-and-torn-write-scheme.md), where the append-only transition log is the durable artifact and all state is replayed from it). Every input produces events and nothing else: hotkeys, the mini widget, timeline edits, and any future import all append transitions. The timeline is the reconstructed state, and the primary interface for reading and editing it; exports read the reconstruction, never the raw log.

  **Scope:** this covers *tracked timeline state* only. Task Templates (`docs/product/features/task-templates.md`), hotkey bindings, and export settings live in `settings.json` by design and are deliberately outside it — see `docs/architecture/constraints.md`.

  The consequence is that **no part of the system needs to know where a change came from.** A block created by a hotkey at the time it happened and a block drawn onto the timeline three hours later are the same kind of thing downstream — which is what keeps the reconstruction workspace from becoming a second, parallel write path with its own rules. Reports, exports, and any future integration remain derived views; capture never changes to serve a downstream format.

## How it's different

**The timeline is the interface; the interruption system is the product.** The timeline is the best representation of the data, but it is explicitly *not* the differentiator — the interruption model is what produces better data in the first place.

That distinction is now evidence-based rather than assumed (research 2026-07-28, first competitive work done on this project):

- **Timeline editing is precedented.** Toggl Track's Calendar view already offers click-drag to create an entry, edge-drag to adjust start/end, and drag-to-move. Anchor's reconstruction workspace is the same interaction model. Any claim that Anchor's timeline is novel would be false, and this project should not make one.
- **The stack mechanic has precedent, but thinner.** `task-stack` keeps a persistent, hotkey-accessible task stack with push-on-interruption and pop-on-done — without time tracking, project/client attribution, or export. Existing time trackers do offer *some* return-to-interrupted-work: Toggl's Continue button resumes a previous entry, and Tyme's cluster mode addresses the same fragmentation differently. What none of them has is **arbitrary interruption depth, explicit root-task recovery, and persistent skip provenance** as one model. That narrower claim is the defensible one; an earlier draft of this section asserted they had "no first-class get-back-to-where-I-was," which one keystroke in Toggl falsifies.
- **The closest existing products are not the ones this doc used to name.** **ManicTime** (Windows, local-first, timeline-primary with manual editing, project/client tagging, Excel export, invoices from timesheets) and **Memtime** (local-only, automatic day timeline, built for transferring a captured day into an external project-time system — the Produra workflow, from a German vendor) both already cover "local-first timeline + billing-grade export."

**So this section is not a differentiation claim — it is a build-vs-buy record.** A tool with one user does not need a market position; it needs a defensible reason not to just install ManicTime. That reason is a difference in *philosophy*, not feature count (`docs/principles.md` #2):

- **Deterministic, manual capture.** ManicTime and Memtime both capture **automatically**. [ADR 0001](../decisions/0001-manual-assisted-tracking-for-mvp.md) rejected that deliberately, for determinism and privacy — it is Anchor's actual reason to exist, and it is a trade, not a superiority.
- **Interruption-aware capture** — arbitrary depth, root-task recovery, explicit skip provenance — producing better data at the point of capture rather than better analysis afterwards.
- **Export shaped for enterprise time-booking systems** (e.g. Produra), as a format concern, not an integration.

The narrow claim that survives: *no evaluated tool provides an interruption model combining arbitrary stack depth, explicit root-task recovery, and persistent skip provenance.* **That is an assumption, not a conclusion** — the search was not exhaustive and nothing has been evaluated hands-on. See the 2026-07-28 replacement row in `docs/assumptions.md`; the original, broader assumption is recorded there as **invalidated**. MVP success (`docs/product/mvp.md`) is still validated against the author's own workflow only, never against competing tools.

## Key assumptions

See `docs/assumptions.md` for the full, current log — not restated here, to avoid the two drifting out of sync. The rows most load-bearing for this Concept are the ones on manual tracking vs. "no work forgotten," the auto-complete mechanic, and export-time aggregation.

## Open questions

- Whether the interruption-stack mechanic is a genuine differentiator or something existing tools already handle adequately. **Partially answered 2026-07-28** — timeline editing is definitively precedented (Toggl), the stack mechanic is precedented outside time tracking (`task-stack`), and the untested claim is now specifically the *combination*. Hands-on evaluation of Toggl/Harvest/Clockify still not done.
- Whether a reconstruction workspace weakens capture discipline in practice: if the timeline can always be fixed later, does the user stop capturing in the moment — turning the exception path into the default and re-creating exactly the after-the-fact guesswork Anchor exists to eliminate? Tracked as a risk; only real usage answers it.
- ~~What "minimal manual effort" means concretely~~ — **answered 2026-07-28**: `docs/vision/vision.md` defines Capture Rate (≥90% of tracked minutes live-captured, rolling working week, provisional). That *is* the threshold. Retained struck-through rather than deleted, because the question was real until it was answered in the same session.

---

**Keeping this current:** if the MVP (`docs/product/mvp.md`) or a feature doc no longer matches this concept, update whichever is wrong — don't let the concept become aspirational fiction.
