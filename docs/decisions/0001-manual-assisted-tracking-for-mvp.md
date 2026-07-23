---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/product/mvp.md, docs/risks.md, docs/product/features/interruption-stack.md]
---

# 0001: Manual, Assisted Tracking for MVP (No Passive/Idle Detection)

## Context

Anchor's core value proposition is an accurate, effortless record of a real workday built from Time Blocks (see `docs/concept/concept.md`). Time can be captured in fundamentally different ways: fully manual (the user explicitly starts/switches/interrupts/resumes/completes), fully passive (inferred from calendar, app focus, window titles), or a hybrid (manual switching, with passive detection only to catch untracked gaps after the fact).

## Options Considered

1. **Fully manual, assisted tracking** — user explicitly triggers every state change via hotkey/command palette; Anchor only records timestamps for what it's told.
2. **Fully passive/automatic tracking** — Anchor infers activity from calendar, app focus, window titles, etc., and asks the user to confirm/categorize.
3. **Hybrid** — manual switching for intentional changes, passive detection only to flag untracked gaps (e.g. "no active task for 12 minutes") without inferring what happened during them.

## Trade-offs

| | Manual (1) | Passive (2) | Hybrid (3) |
|---|---|---|---|
| Complexity | Low — no inference engine, no OS-level hooks | High — calendar/app-focus integration, inference logic, privacy handling | Medium — needs gap/idle detection, but no content inference |
| Determinism | Fully deterministic — the record is exactly what the user did | Non-deterministic — inference can misclassify or miss context | Deterministic for tracked time; gap-flagging is a nudge, not an inference |
| Privacy | No background monitoring of activity at all | Requires access to calendar/app/window data — real privacy surface | Only needs "was anything active," not *what* was active |
| Addresses "no work forgotten" | Only for what's actually pushed onto the stack — forgetting to track something isn't caught | Best coverage, since it can catch untracked gaps directly | Partially — flags a gap exists, but doesn't recover what happened in it |
| Fit with "fully under user's control" (Vision) | Strongest fit | Weakest fit — the system decides what counts as activity | Middle — still manual capture, just adds a passive safety net |

## Decision

**Fully manual, assisted tracking (Option 1) for MVP.** Anchor records exactly what the user tells it to record, via Switch/Interrupt/Return/Complete operations — nothing is inferred. This keeps the system deterministic, privacy-friendly, and fully under the user's control, consistent with `docs/vision/vision.md`'s non-goals.

This means "no work forgotten" in the MVP is explicitly scoped to *"tracked work is never lost through the interruption model"* — not *"untracked work is detected and recovered."* That gap is real and intentional; it is logged as risk **R3** in `docs/risks.md` (status: `accepted`) rather than silently assumed away.

## Consequences

- Makes the MVP significantly simpler to build and reason about — no inference engine, no background monitoring, no calendar/OS integration surface.
- Makes the core value proposition ("accurate record... no manual reconciliation") conditionally dependent on the user's own discipline in pressing the hotkey — an explicit, accepted risk (R3), not a guarantee.
- Leaves the door open to Option 3 (Hybrid) later without a rewrite: gap/idle *detection* (not inference) could be layered on top of the existing manual capture model as an optional, non-default feature, per `docs/vision/vision.md`'s note that "intelligent assistance may be introduced as an optional feature in future versions."
- Revisit this decision (new ADR, not an edit) if real personal usage under R3 shows work is regularly lost to forgetfulness that the stack model can't account for.

## Amendment (2026-07-23)

The Decision's "nothing is inferred" was found, during design of `docs/product/features/interruption-stack.md`, to be overbroad: gap recovery (an ungraceful crash, or resuming from sleep/hibernate with an active entry) does infer an end timestamp from the last durable transition write, marking the affected Time Block `recovered-gap`. This scopes the original claim rather than reversing it: **activity content is never inferred** (what the user was doing, or whether an interruption occurred, is always explicit) — only recovery-time *metadata* (when an already-known entry's end occurred) is inferred, and only ever in a flagged, user-correctable way, never silently as `explicit`. The underlying decision (no passive/automatic activity detection) stands unchanged.
