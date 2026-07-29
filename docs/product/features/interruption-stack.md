---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/risks.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/decisions/0002-desktop-app-framework-and-platform.md, docs/decisions/0004-transition-log-format-and-torn-write-scheme.md]
---

# Interruption Stack (Switch / Interrupt / Return)

> Created via `/new-feature interruption-stack`. This is the foundational MVP mechanic — Task Templates and export both depend on the data this produces. Following `.claude/workflows/design.md` in order. Revised after an independent reviewer pass (2026-07-23) — see git history / conversation for what changed and why. Revised again (2026-07-27, see conversation): the Switch/Interrupt hotkeys start tracking immediately instead of focusing a quick-input popup, naming is now optional at start time (auto-assigned, renamed later), and a new Rename capability was added — see Goals, UX, Technical Constraints, and Acceptance Criteria below.
>
> **Known gap (2026-07-28), tracked as `docs/risks.md` R9:** this doc's `recovered-gap` correction promise (UX, "Gap-recovery state") is not implemented and cannot be performed today — no transition type edits a completed Time Block's times, and no command exposes one. The design decision below stands; what's missing is the mechanism.
>
> **Update, same day:** the Concept revision (`docs/concept/concept.md`) moved timeline reconstruction into MVP scope, so this gap now has a scheduled resolution rather than an open-ended one — the timeline becomes an editing workspace, and `recovered-gap` correction is one case of it. Still blocked on an ADR (transition type carrying explicit start **and** end, ordering under ADR 0004, overlap rules); see `ideas/manual-time-block-entry.md`.
>
> **Also revised by that session:** the stack is no longer strictly hidden — see "Progressive disclosure" below. This doc's core claims are otherwise unaffected. In particular its position that the dashboard is "not meant for rapid interaction — opened deliberately, not part of the fast switch/interrupt/return loop" **survives unchanged**, because the revision confirmed Anchor is capture-first: primary capture stays on hotkeys and the mini widget, and the timeline is a review-and-refine surface, not a data-entry one.
>
> **Vocabulary and metadata pending revision (2026-07-28) — [ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md).** Two things below are now stale wording rather than stale decisions: (a) `completion reason` and its three values are superseded by `EndDetermination` / `CaptureOrigin` / `InterruptionOutcome`, so the acceptance criteria referencing `explicit` and `auto-completed-on-skip` need restating against the new fields — `auto-completed-on-skip` becomes `InterruptionOutcome::Skipped`; (b) "the dashboard's timeline view" is now the **History View**, one of two views over the Timeline alongside the **Timeline Editor**. The behaviour those criteria assert is unchanged. Also newly specified: **a Switch never modifies the interruption stack** — previously true in the implementation but guaranteed by no document.
>
> **Progressive disclosure (2026-07-28).** The default UI still surfaces only the current task and the two return options, exactly as designed below. The full interruption stack is additionally inspectable behind an optional "interruption history" panel, so the user can confirm nothing was lost. This grants **no third return path** — Return to Previous and Return to Original remain the only actions, and the Alternatives below are unchanged.

## Problem

A real workday for the target user (`docs/product/users.md`) isn't one continuous task — it's a primary task interleaved with side tasks and interruptions (calls, messages, spontaneous requests) that need immediate attention but shouldn't cause the original task to be lost or forgotten. Existing time-tracking tools center on a single active timer with no first-class notion of "I was pulled away, and I need to get back to exactly where I was" — see `docs/concept/concept.md` "How it's different." **Corrected 2026-07-28:** that claim is too strong and competitive research falsified it — Toggl's Continue button resumes a previous entry, and Tyme's cluster mode addresses the same fragmentation. The defensible claim, and the one `concept.md` now makes, is narrower: no evaluated tool combines **arbitrary interruption depth, explicit root-task recovery, and persistent skip provenance** as one model. The Problem this doc solves is unchanged; only the competitive claim was overstated.

## Goals

- Entering an interruption and returning from it takes a couple of seconds of interaction, with no more than the two decisions the mechanic requires (interrupt vs. switch; return-to-previous vs. return-to-original). Naming what the new Time Block *is* is deliberately not a required third decision: the Switch/Interrupt hotkeys start tracking immediately with an auto-assigned name, and naming (or renaming to an existing template/past task) can happen at any point afterward, at the user's own pace — see UX and Technical Constraints.
- The record produced is exactly what happened: every Time Block has an accurate start/end and a completion reason (`explicit`, `auto-completed-on-skip`, or `recovered-gap` — see `docs/glossary.md`), with no ambiguity about what was actually finished, skipped, or reconstructed after a gap in tracking.
- Nested interruptions behave correctly at any depth actually tested — the stack never loses, corrupts, or misorders an entry. (Acceptance Criteria test to a representative depth, not literal infinity — see below.)
- Ties to `docs/vision/vision.md` "Success looks like": this feature alone should make "nothing gets lost inside the interruption stack" true.

## Users

Serves the single primary persona in `docs/product/users.md` — the interrupted billable developer — directly and entirely; this is the mechanic their whole workflow centers on. No other segment is in scope (see `docs/product/users.md` "Explicitly not users").

## Alternatives

**Interaction model:**
1. Global hotkeys only, no persistent UI — fastest per-action, but zero at-a-glance awareness of current task or stack depth.
2. Global hotkeys + an always-on-top mini widget (current task, stack depth) — adds passive awareness without leaving the keyboard.
3. **Global hotkeys + mini widget + a full dashboard window** (timeline view, template management, export) — richest, most build surface, but the mini widget alone can't reasonably host history review, template editing, or export.

**Persistence/durability model:**
1. In-memory state with periodic snapshot to disk — simplest, but data since the last snapshot is lost on a crash, directly violating the duration-accuracy goal.
2. A single mutable state file, overwritten on every transition — durable once written, but a crash *during* the write (a torn write) can corrupt the entire file, not just lose the latest transition. Rejected: this is a worse failure mode than the snapshot alternative it was meant to improve on.
3. **Append-only transition log** (each transition appended as its own record; current state is the replay of the log) with periodic compaction/snapshot for fast startup — a torn write can only ever affect the last, incomplete record, which is trivially detected (fails to parse) and discarded on next launch; every prior record is untouched.
4. Write-new-file + atomic rename per transition — equally durable/atomic to (3), but more filesystem overhead per action than a simple append, with no benefit at this interaction frequency.

**End-time inference on gap recovery (crash or sleep/hibernate):**
1. Leave the end time null/unknown, blocking any new tracking action until the user manually enters it — maximally correct, but adds mandatory friction to the next thing the user does, which cuts against "effortless."
2. **Infer end time from the last durable write (transition or heartbeat), mark the entry `recovered-gap`, and surface it in the dashboard for the user to correct at their own pace** — no forced interruption; user chose this explicitly (see conversation, 2026-07-23): a sleep gap doesn't necessarily mean the task stopped (e.g. stepping away to a face-to-face meeting related to the same work), so auto-flagging without forcing a stop/confirm decision is correct, not just convenient.
3. Prompt the user immediately on wake/relaunch — most accurate if the user reliably answers correctly, but adds an interruption to every wake, which the user explicitly rejected in favor of (2).

**Naming at start time (added 2026-07-27):**
1. Hotkey opens a quick-input popup requiring a typed name before the transition commits — the original design. Rejected on revision: this makes naming a mandatory third decision on top of the two the Goals section already limits the mechanic to, and the popup itself steals focus, which the mini widget's own "no focus-stealing" principle already argues against.
2. Hotkey performs the transition immediately with a required non-blank name, falling back to nothing if the user hasn't typed one — inconsistent: still requires *some* value at the exact moment speed matters most.
3. **Hotkey performs the transition immediately; if no name is given, an auto-generated name (`Anchor N`, see Technical Constraints) is assigned, correctable at any later point via Rename.** **Chosen** — user decision (2026-07-27): naming and starting are decoupled entirely. The auto-name is a real, exportable name like any other (not null, no log-format change), so nothing downstream (export, aggregation) needs special-casing for "unnamed."

**Renaming an in-progress task (added 2026-07-27):**
1. No rename capability — a task's name is fixed at Start/Switch/Interrupt and can only be corrected by editing raw log data. Rejected: incompatible with option 3 above, which requires a way to give an auto-named task a real name later.
2. Rename draws suggestions from Task Templates only, matching the existing autocomplete Task Templates already added to quick-input. Simplest, but can't resume a one-off past task name that was never saved as a template.
3. **Rename draws suggestions from both Task Templates and distinct past Time Block names/project/client combinations ("history"), visually tagged by source.** **Chosen** — user decision (2026-07-27): "both." A durable, logged `Rename` transition (see Technical Constraints) changes the active Time Block's identity fields in place — no new Time Block, no stack effect, start time untouched.

**Heartbeat cadence bounding inference accuracy:** option 2 above is only as accurate as the most recent durable write. A user-triggered transition (Switch/Interrupt/Return) could be hours apart during genuinely heads-down work, which would leave `recovered-gap`'s inferred end time hours stale — silently defeating the accuracy this mechanism exists to provide. Three options:
1. No heartbeat — rely on transition writes alone. Rejected: inference error is unbounded (as long as the longest realistic single task), directly undermining R4's mitigation story.
2. **Fixed-interval heartbeat write (every 60 seconds) while any entry is active** — a trivial timestamp-only append, bounding inference error to ~1 minute regardless of how long a task runs, at a negligible cost (one small write/minute, only while something is being tracked, none while idle).
3. Activity-triggered heartbeat (e.g. on keyboard/mouse input) — tighter bound in principle, but requires monitoring input activity, which is exactly the kind of activity-content inference ADR 0001 rules out. Rejected on those grounds, not on accuracy.

## Trade-offs

| | Interaction | Persistence/Durability | End-time inference | Heartbeat cadence | Naming at start | Rename source |
|---|---|---|---|---|---|---|
| **Chosen** | Hotkeys (user-remappable, sane defaults) + mini widget + dashboard | Append-only transition log + periodic compaction | Infer from last durable write, flag `recovered-gap`, correct later | Fixed 60s interval while an entry is active | Immediate start, auto-name (`Anchor N`) if blank | Both Task Templates and past Time Block history |
| Complexity | Higher — two UI surfaces to build and keep in sync off one shared state | Low-moderate — no transaction engine, but requires a replay/compaction step | Low — reuses the same gap-detection mechanism for both crash and sleep/hibernate | Trivial — one more append type, no new subsystem | Low — a pure function over the existing timeline, no new persisted state | Moderate — a combined, source-tagged autocomplete over two data sources instead of one |
| Reversibility | Mini widget/dashboard can be simplified later without changing the underlying data model | Compaction format can change later; log format itself should be treated as a stable on-disk contract once shipped | Correction UI can be made stricter later without changing the data model (the field already exists) | Interval can be tuned later without changing the mechanism | The naming convention string can change later without a data migration — it's just a name like any other | Either source could be dropped later without changing the `Rename` transition itself |
| Risk if wrong | Building the dashboard too early, before the core stack is proven | An unreplayed torn record at the tail could still misrepresent the very last transition if not handled — see Technical Constraints | A long, silent gap (e.g. laptop asleep 10 hours) produces a wildly wrong inferred duration if the user never notices/corrects it — see `docs/risks.md` R4 | Too infrequent → same staleness problem as no heartbeat; too frequent → needless disk writes. 60s chosen as a low-cost default, not measured. | An `Anchor N` task left unrenamed reproduces R2's aggregation fragmentation under a meaningless name — see `docs/risks.md` R8 | A combined list can visually crowd out the more relevant source if templates and history overlap heavily |

## UX

- **Mini widget** (always-on-top, no focus-stealing): shows current task name, current stack depth (e.g. "3 deep"), and updates immediately on any state change. Empty state: "No active task" when nothing is being tracked.
- **Dashboard** (full window): timeline view of all Time Blocks (name, project/client, start/end, duration, completion reason), Task Template management, and export actions (XLSX/JSON). Not meant for rapid interaction — opened deliberately, not part of the fast switch/interrupt/return loop.
- **Switch/Interrupt hotkeys**: no popup, no focus-stealing — pressing the hotkey performs the transition immediately. If no name was given (see Rename below for the dashboard's manual Switch/Interrupt, which also accepts a blank name), the new Time Block is auto-named `Anchor N` (Technical Constraints).
- **Rename** (dashboard, "Active" section, visible whenever a task is running): name/project/client fields pre-filled with the active task's current values, editable and savable at any time without affecting the stack or the task's start time. Autocomplete combines Task Templates and distinct past Time Block name/project/client combinations ("history"), each suggestion tagged by source so a template and a same-named past task aren't visually confused.
- **Return actions**: two distinct hotkeys (Return to Previous, Return to Original) — no prompt needed, since both are unambiguous given the current stack state.
- **Gap-recovery state**: on relaunch after an ungraceful shutdown, *or* on resume from sleep/hibernate while an entry was active, that entry is marked `recovered-gap` with an inferred end time and surfaced distinctly in the dashboard — never silently folded in as `explicit`. No prompt interrupts the user in the moment (on wake or on relaunch); correction happens whenever the user next opens the dashboard. **(Not yet implemented — the correction mechanism this sentence assumes does not exist; see `docs/risks.md` R9 and the note at the top of this doc.)**

## Technical Constraints

- Every transition (start/switch/interrupt/return-previous/return-original/complete) must be durably appended to an on-disk log before being considered committed — the load-bearing constraint behind "a crash won't interrupt the duration."
- Durability must be crash-atomic at the record level: use an append-only log so that a torn write can only ever corrupt the last, in-progress record — which must be detected and discarded on next launch, never allowed to corrupt or block replay of prior records. The concrete on-disk format, checksum framing, and compaction scheme are decided in [ADR 0004](../../decisions/0004-transition-log-format-and-torn-write-scheme.md), not here.
- Gap detection is unified for both causes: (a) no clean-exit marker was found on launch (crash), or (b) an OS-level sleep/hibernate signal fired while an entry was active. Either case ends that entry with completion reason `recovered-gap` and an end time inferred from the last durable write — never silently marked `explicit`, and never blocking the user with a mandatory prompt.
- While any entry is active, a lightweight heartbeat (timestamp-only append) is written every 60 seconds, independent of transitions — this bounds `recovered-gap` inference error to roughly one minute regardless of how long a single task runs, rather than leaving it as stale as the last user-triggered transition.
- Mini widget and dashboard must read from the same underlying persisted state — no divergent in-memory caches that could disagree about current stack depth.
- Global hotkeys must be OS-registered, user-remappable, with sane defaults and conflict detection against other running apps. Target platform is Windows for MVP, hotkeys implemented via Tauri's `global-shortcut` plugin — see [ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md). Whether that plugin actually surfaces a distinguishable conflict error is not yet confirmed; treat as an open implementation question, not a solved one.
- Sleep/hibernate detection (part of the unified gap detection above) requires direct Windows API calls from the Rust core (e.g. handling `WM_POWERBROADCAST`), not something Tauri provides natively — see ADR 0002 Consequences and `docs/risks.md` R6.
- The interruption stack itself (not just completed Time Blocks) must be part of what's persisted and recovered — a restart needs to restore exactly which task is root, which interruptions are stacked above it, and in what order.
- This gap-inference behavior narrows, but does not contradict, ADR 0001's "nothing is inferred" — see the amendment in `docs/decisions/0001-manual-assisted-tracking-for-mvp.md`: that claim is scoped to activity *content* (what the user was doing), not recovery-time *metadata* (when a gap ended), which is explicitly flagged and user-correctable rather than silently trusted.
- A blank name given to Start/Switch/Interrupt (from either hotkey, which never collects one, or the dashboard's buttons, which no longer require one) is replaced with an auto-generated name of the form `Anchor N` before the transition is built — nothing blank is ever durably logged or exportable. `N` is computed purely from the existing timeline: the highest `N` already used by an `Anchor N`-named Time Block that started at or after the current local calendar day's start, plus one. This resets the count once per real day without a separate persisted counter, and needs no change to the on-disk transition log format ([ADR 0004](../../decisions/0004-transition-log-format-and-torn-write-scheme.md) is unaffected — `name` stays a required, non-null string, exactly as already shipped).
- Rename is its own transition type (`Rename`), additive to the existing `TransitionPayload` schema — old logs remain fully replayable, no migration needed. It changes only the currently active Time Block's `name`/`project`/`client` in place: no new Time Block, no stack effect, start time untouched. Requires an active task; rejected otherwise. Like every other transition, it must be durably logged before being considered committed, so a rename survives a crash immediately after it — a `recovered-gap` entry recovered after a rename reflects the renamed identity, not the stale auto-generated one.

## Acceptance Criteria

- User can trigger Switch and Interrupt via configurable global hotkeys (working defaults out of the box, remappable in settings), and a remapped hotkey persists across an app restart.
- A conflicting hotkey assignment (already bound to another action or another app, where detectable) is surfaced to the user rather than silently failing to register.
- A full Switch or Interrupt action triggered by its hotkey completes in a single keypress, with no popup and no required naming step — the two decisions described in Goals (interrupt vs. switch; return-to-previous vs. return-to-original) are the only ones required, matching or beating the prior 3-second budget.
- Triggering Switch or Interrupt with no name given (hotkey, or the dashboard's buttons left blank) assigns the new Time Block an auto-generated name of the form `Anchor N`, where `N` is 1 for the first such task on a given local calendar day and increments for each subsequent one, resetting the next day.
- Renaming the active task (dashboard) changes its name/project/client without altering its start time, its position (if any) relative to the stack, or any other Time Block; a rename immediately before a crash or a sleep/hibernate gap is reflected in the resulting `recovered-gap` entry, not the pre-rename name.
- The rename control's autocomplete surfaces both Task Templates and distinct past Time Block name/project/client combinations, each visually tagged by which source it came from.
- Mini widget reflects current task and stack depth within 150ms of any state change.
- Mini widget and dashboard, when both open simultaneously, never disagree about current task or stack depth.
- Dashboard's timeline view shows every Time Block with correct start/end/duration and its completion reason (`explicit`, `auto-completed-on-skip`, or `recovered-gap`).
- Simulating an ungraceful process kill mid-interruption, then relaunching, results in: the full stack restored exactly as it was, all prior completed Time Blocks intact with correct durations, and the interrupted active entry marked `recovered-gap` with an end time inferred from the last durable write, accurate to within ~60 seconds of the actual kill time (bounded by the heartbeat interval) — zero data loss, and no corruption of any record prior to the killed transition.
- Simulating a torn write (process killed mid-append), then relaunching, results in the incomplete trailing record being discarded and every prior record intact and correctly replayed.
- Simulating a sleep/hibernate cycle with an active entry, then resuming, results in that entry marked `recovered-gap` with an inferred end time, with no prompt shown to the user at wake time.
- Return to Previous and Return to Original both function correctly at a stack depth of at least 10, with skipped entries marked `auto-completed-on-skip` (never `explicit`).

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
