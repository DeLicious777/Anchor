---
status: draft
date: 2026-07-28
owner: erich
related: [docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/risks.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Vision

> **Under revision (2026-07-28), was `accepted` since 2026-07-23.** Reverted to `draft` by the Concept revision session that established Anchor as *capture-first, timeline-assisted*. Two success criteria below were absolute in a way the product no longer is — see "Success looks like." Returns to `accepted` only after the Discovery workflow's Stage 5 reviewer pass.

## Problem

Software developers and other knowledge workers who bill by effort struggle to keep an accurate record of a real workday. A day isn't one continuous task — it's a primary task interleaved with side tasks, spontaneous calls, Slack pings, and other interruptions. Manually tracking this accurately, without losing track of what you were doing before the interruption, is difficult. Existing time-tracking approaches (single active timer, or passive/automatic inference) don't model the actual shape of the work: an interruption stack that needs to be entered and exited cleanly, every time, without losing or forgetting anything.

## Why now

A personal pain point, supported by recurring observations across real-world development workflows — not a market timing bet. The author experiences this friction directly and repeatedly, which is reason enough to build a first version now, scoped to solving it for one person before considering anyone else.

## Vision statement

Anchor lets people who work in bursts — switching between primary work and interruptions all day — capture an accurate, effortless record of exactly how their time went, and always find their way back to the task they meant to return to.

## Success looks like

- A full workday (e.g. 8 hours for a full-time employee) can be reconstructed after the fact as an accurate sequence of time blocks, correctly attributed to their project/client, **with minimal manual effort and entirely inside Anchor** — the user never reconciles their work in another tool. Billable-vs-non-billable classification itself is not something Anchor decides — that happens downstream once the exported timeline is transferred into whatever billing/invoicing process the author already uses; Anchor's job is accurate project/client attribution, not billing classification. See [ADR 0003](../decisions/0003-billable-classification-out-of-scope.md).
- Switching to a new task, or handling an interruption and returning from it, is effectively instantaneous via a global hotkey — with no cognitive overhead about "am I tracking this correctly." **Measured as Capture Latency**: time from invoking a capture action to the new work becoming the active tracked task, inclusive of the durable write that `docs/product/features/interruption-stack.md` requires before a transition counts as committed. **Provisional target ≤1s** — not measured, chosen as a design target (same convention as ADR 0004's `N = 500` and the 60-second heartbeat). If the durable-write path makes it unattainable, that is an architectural finding to review, not a requirement to quietly relax.
- **Capture stays the primary path, and that is measurable.** **Capture Rate**: percentage of tracked work minutes whose Capture Origin is live capture, over a rolling working week. **Provisional target ≥90%** — an explicit design hypothesis, not an evidence-based optimum; re-evaluate after roughly four weeks of regular real-world usage. This exists so that risk R10 (a reconstruction workspace eroding capture discipline) can actually be falsified rather than argued about.

  **Adjusted blocks still count as live capture.** Pressing the hotkey at 09:00 and correcting 09:05 to 11:00 that evening is successful capture with imperfect timing — not work reconstructed from memory. Excluding such blocks would measure editing habits rather than capture discipline, letting a one-second correction erase the evidence that the task was captured as it happened. Capture quality is a *separate* question, answered by a second metric rather than by overloading this one: **Adjustment Rate** — percentage of live-captured minutes subsequently adjusted. Same unit, so a two-minute correction and a four-hour one don't weigh the same. Read together they diagnose: high capture / low adjustment is the goal; high capture / high adjustment means capture happens but timing is unreliable; low capture / low adjustment means work is being reconstructed rather than captured. Capture Rate remains the success criterion; Adjustment Rate exists to interpret it.

  **Anchor must make this metric *computable*, not display it.** The requirement is that full-fidelity exported data (`docs/product/features/export.md`'s ungrouped JSON) carries enough per-block metadata to derive the number objectively. Whether it is then computed by a script, a test, a future dashboard, or by hand is an implementation detail. Stated this way deliberately: requiring a *displayed* figure would add an analytics surface whose purpose is validating the product rather than helping the user work, and `docs/product/mvp.md` keeps analytics views out of scope. Grouped exports stay intentionally lossy and billing-oriented; full-fidelity export is the analysis artifact.
- Nothing gets lost inside the interruption stack: every task that was pushed is eventually **resumed or skipped**, and the record says which — never silently dropped. (Revised 2026-07-28. Previously "explicitly completed or explicitly returned to," which the model does not deliver: a frame can also be dismissed without returning, and that is a legitimate outcome rather than a gap. The wording now describes the model rather than the model being bent to preserve the wording.)
- At the end of a day or week, exporting to XLSX/JSON produces data the author would actually trust and use for billing **without post-processing in Excel or any other external tool**. Transferring the result into an enterprise time-booking system (e.g. Produra) is a copy, not a reconciliation.
- The Timeline is a **reconstruction workspace**, not a passive log: the day can be refined inside Anchor — adding work that went untracked, correcting an inferred end time, fixing attribution — and every block still represents actual work that happened. Editing is an intended capability, not evidence of failure. The MVP edit surface is five operations (add, move, resize, edit identity, delete); split and merge were considered and removed for lacking problem statements.

**Why two of these changed (2026-07-28).** They previously read "with no manual reconciliation" and "without hand-editing." Both were absolute in a way that contradicted the product Anchor is becoming: a rich timeline editor whose whole purpose is hand-editing. The intent behind them was never "the user never touches their data" — it was "the user never has to leave Anchor and fix things up in Excel." That is what they now say.

## Non-goals

- Passive or automatic activity detection (calendar, app focus, window titles) — tracking stays fully manual and user-initiated. See [ADR 0001](../decisions/0001-manual-assisted-tracking-for-mvp.md) and `docs/assumptions.md`.
- Idle/gap detection or "did you forget to track this" prompts — explicitly deferred past MVP.
- Multi-user accounts, team visibility, or collaboration features — this is a single-user tool for now.
- Additional clients beyond desktop (CLI, browser extension, mobile) — planned architecturally (shared data model) but not built in the first version.
- Invoicing, PDF generation, timesheets, or third-party integrations (Jira, Harvest, Toggl) — the timeline (projected from the transition log, which is the actual source of truth — corrected 2026-07-28) is what everything derives from; these are downstream consumers to build later, not now.
- Treating this as a commercial product from day one — scope is personal use first (see `docs/product/users.md`).

## Open questions

- If/when this moves beyond personal use, what does the path to multi-user or distributable product look like, and does the MVP's architecture (esp. the flat, unsynced timeline) accommodate that without a rewrite?
- Is "no work forgotten" ever going to need gap-detection assistance, or does the stack model alone genuinely cover it in practice? Revisit after real usage.
- Are existing time-tracking tools (Toggl, Harvest, Clockify, RescueTime) genuinely missing the interruption-stack mechanic, or does one of them already solve this well enough? **First evidence gathered 2026-07-28** (Concept revision session), narrowing but not closing this:
  - **The timeline half is precedented.** Toggl Track's Calendar view already supports click-drag to create an entry with start/end pre-populated, dragging an entry's edges to change start or end, and dragging a block to move it — the same interaction model as Anchor's reconstruction workspace. The timeline is therefore *not* a differentiator; see `docs/concept/concept.md` "How it's different."
  - **The stack half has precedent too, but thinner.** `task-stack` (tomrochette.com) is a system-tray app holding a persistent, reorderable task stack one hotkey away, with push-on-interruption and pop-on-done. It is not a billing/time-tracking product and has no export or project/client attribution, but the stack mechanic itself is not novel.
  - **Answered, and not the way this doc assumed.** A second research pass found **ManicTime** and **Memtime** already shipping local-first timeline plus billing-grade export on Windows, so the combination is *not* a gap. The original assumption is recorded as **invalidated** in `docs/assumptions.md` (2026-07-23 row); its narrower replacement — *no evaluated tool combines arbitrary interruption depth, root-task recovery, and persistent skip provenance* — is the row dated 2026-07-28, and remains an assumption, not a conclusion. `docs/concept/concept.md` was reframed from differentiation to build-vs-buy as a result.

---

**Keeping this current:** revisit this doc whenever an ADR or feature decision seems to strain against it — that's a signal either the vision needs updating or the decision does. Don't let them silently diverge.
