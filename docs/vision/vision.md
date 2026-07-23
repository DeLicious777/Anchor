---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/risks.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Vision

## Problem

Software developers and other knowledge workers who bill by effort struggle to keep an accurate record of a real workday. A day isn't one continuous task — it's a primary task interleaved with side tasks, spontaneous calls, Slack pings, and other interruptions. Manually tracking this accurately, without losing track of what you were doing before the interruption, is difficult. Existing time-tracking approaches (single active timer, or passive/automatic inference) don't model the actual shape of the work: an interruption stack that needs to be entered and exited cleanly, every time, without losing or forgetting anything.

## Why now

A personal pain point, supported by recurring observations across real-world development workflows — not a market timing bet. The author experiences this friction directly and repeatedly, which is reason enough to build a first version now, scoped to solving it for one person before considering anyone else.

## Vision statement

Anchor lets people who work in bursts — switching between primary work and interruptions all day — capture an accurate, effortless record of exactly how their time went, and always find their way back to the task they meant to return to.

## Success looks like

- A full workday (e.g. 8 hours for a full-time employee) can be reconstructed after the fact as an accurate sequence of time blocks, correctly attributed to their project/client, with no manual reconciliation. Billable-vs-non-billable classification itself is not something Anchor decides — that happens downstream once the exported timeline is transferred into whatever billing/invoicing process the author already uses; Anchor's job is accurate project/client attribution, not billing classification. See [ADR 0003](../decisions/0003-billable-classification-out-of-scope.md).
- Switching to a new task, or handling an interruption and returning from it, takes a couple of seconds via a global hotkey — with no cognitive overhead about "am I tracking this correctly."
- Nothing gets lost inside the interruption stack: every task that was pushed is either explicitly completed or explicitly returned to, never silently dropped.
- At the end of a day or week, exporting to XLSX/JSON produces data the author would actually trust and use for billing, without hand-editing.

## Non-goals

- Passive or automatic activity detection (calendar, app focus, window titles) — tracking stays fully manual and user-initiated. See [ADR 0001](../decisions/0001-manual-assisted-tracking-for-mvp.md) and `docs/assumptions.md`.
- Idle/gap detection or "did you forget to track this" prompts — explicitly deferred past MVP.
- Multi-user accounts, team visibility, or collaboration features — this is a single-user tool for now.
- Additional clients beyond desktop (CLI, browser extension, mobile) — planned architecturally (shared data model) but not built in the first version.
- Invoicing, PDF generation, timesheets, or third-party integrations (Jira, Harvest, Toggl) — the timeline is the source of truth; these are downstream consumers to build later, not now.
- Treating this as a commercial product from day one — scope is personal use first (see `docs/product/users.md`).

## Open questions

- If/when this moves beyond personal use, what does the path to multi-user or distributable product look like, and does the MVP's architecture (esp. the flat, unsynced timeline) accommodate that without a rewrite?
- Is "no work forgotten" ever going to need gap-detection assistance, or does the stack model alone genuinely cover it in practice? Revisit after real usage.
- Are existing time-tracking tools (Toggl, Harvest, Clockify, RescueTime) genuinely missing the interruption-stack mechanic, or does one of them already solve this well enough? No competitive research has been done yet — see `research/`.

---

**Keeping this current:** revisit this doc whenever an ADR or feature decision seems to strain against it — that's a signal either the vision needs updating or the decision does. Don't let them silently diverge.
