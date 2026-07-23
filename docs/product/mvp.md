# MVP Scope

_Last reviewed: 2026-07-23_

## MVP definition

A desktop application for a single user that lets them track a full workday as a sequence of independent time blocks, using fast, hotkey/command-palette-driven operations, always able to return cleanly from any depth of interruption — and export the resulting timeline for billing and reporting.

## In scope

- **Core interruption model**: Switch, Interrupt, Return to Previous Task, Return to Original Task, Complete — backed by a true nested stack internally, simple UI on top. See `docs/product/features/interruption-stack.md`.
- **Durable persistence with gap recovery**: every transition is written durably (append-only log); an ungraceful crash or a resume from sleep/hibernate is detected and reconciled (`recovered-gap` completion reason, inferred end time, user-correctable) rather than silently lost or silently trusted.
- **Manual, assisted tracking only** — no passive/automatic *activity* detection (what the user was doing is never inferred), no calendar integration. This is distinct from crash/sleep gap recovery above, which infers only a recovery-time timestamp, never activity content — see ADR 0001's amendment.
- **Desktop app, always-available**, with user-remappable global hotkeys, an always-on-top mini widget, and a dashboard for history/templates/export.
- **Task Templates** — reusable presets pre-populating name/project/client for recurring activities.
- **Flat timeline data model** — each time block is an independent entry (name, start, end, duration, optional project/client); aggregation by name/project/client happens at export/report time, not via a stored task entity.
- **Exports**: XLSX and JSON, generated from the timeline as the single source of truth.

_(Feature docs for each of the above land in `docs/product/features/` once the Design workflow runs for each — see `.claude/workflows/design.md`.)_

## Explicitly out of scope (deferred)

- Passive/automatic activity detection, idle detection (i.e. flagging that the user forgot to track something during normal operation), and calendar integration. (Crash/sleep gap *recovery* — reconciling the app's own tracking continuity, not inferring user activity — is in scope; see above.)
- Additional clients — CLI, browser extension, mobile app (architecturally anticipated via a shared data model, but not built now).
- Multi-user accounts, team/collaboration features, sync across users.
- PDF generation, timesheets, invoices, statistics/analytics views.
- Integrations with external tools (Jira, Harvest, Toggl, company-specific systems).

## Success criteria

Ties directly to `docs/vision/vision.md`'s "Success looks like": the author can run a real, full personal workday through Anchor using only hotkey/command-palette interactions, and at the end of the day export an XLSX/JSON timeline that accurately attributes every Time Block to its project/client — with zero manual reconciliation and zero lost or orphaned interruptions. Billable-vs-non-billable classification itself happens downstream (see `docs/decisions/0003-billable-classification-out-of-scope.md`), not within Anchor.

This success claim is conditional on two things the MVP does not resolve, tracked in `docs/vision/vision.md` "Open questions" and `docs/risks.md`: (1) that manual-only tracking (risk R3) doesn't in practice let work go untracked through simple forgetfulness, and (2) that the interruption-stack mechanic is validated against the author's own real workflow only — not against whether existing tools (Toggl, Harvest, etc.) already solve this adequately, which remains unresearched.

---

**Keeping this current:** when a feature doc's status changes to `accepted`, check whether it belongs in "In scope" here. When scope grows, actively ask whether something else should move to "Out of scope" — MVP that only grows isn't an MVP.
