---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/product/mvp.md, docs/product/users.md, docs/concept/concept.md, docs/product/features/task-templates.md, docs/assumptions.md]
---

# 0003: Billable/Non-Billable Classification Is Out of Scope for Anchor's Data Model

## Context

The Vision, MVP, and Target Users docs all describe the underlying user need as splitting a workday into billable and non-billable time. But `docs/concept/concept.md`'s Time Block definition — the accepted data model — has never included an explicit billable/non-billable field; it has only name, start, end, duration, and optional project/client. This mismatch surfaced while designing `docs/product/features/task-templates.md`, when the question of whether a template should carry a default billable flag forced the gap into the open.

## Options Considered

1. **Add an explicit `billable` boolean field to Time Block** — unambiguous, queryable directly from Anchor's own data, but adds a field the author must set (or default via template) for every entry, and duplicates classification logic that likely already exists in whatever billing/invoicing process consumes the export.
2. **Infer billable status from whether project/client is set** (has a client → billable, no client → non-billable) — zero extra input, but conflates "has a client" with "is billable" and would silently misclassify real cases (e.g. unpaid internal work performed under a client project, or billable work not yet attached to a tracked client).
3. **No billable field at all — classification happens downstream**, after the timeline is exported (XLSX/JSON) and transferred into the author's existing billing/invoicing process. Anchor's responsibility ends at accurate project/client attribution.

## Trade-offs

| | Explicit field (1) | Inferred from project/client (2) | Downstream classification (3) |
|---|---|---|---|
| Accuracy | Fully accurate, author-controlled | Systematically wrong for edge cases (unpaid client work, unattached billable work) | Accurate by construction — whatever system already does this classification keeps doing it |
| Data model impact | New field on every Time Block and Task Template | No schema change, but a fragile behavioral rule | No schema change, no new rule |
| Duplicate logic | Yes — Anchor would be re-implementing classification that likely already exists downstream | Yes, and worse — implicitly, not explicitly | No — single source of truth for classification stays wherever it already lived |
| Fit with author's actual workflow | Unconfirmed to be needed | Actively wrong for named edge cases | Matches what the author explicitly stated: exported time is transferred into an existing process that already classifies it |

## Decision

**No billable/non-billable field in Anchor's data model (Option 3).** Anchor's scope is accurate capture and project/client attribution of Time Blocks; classifying that time as billable or non-billable is the responsibility of whatever downstream process (billing/invoicing) the author transfers the exported timeline into. This was a direct author decision (2026-07-23): "not needed because [the downstream billing process] is used and the tracked time will then be transferred."

This narrows — without contradicting — the user need stated in `docs/product/users.md` ("accurately splitting a full workday between billable and non-billable time"): that need is satisfied by accurate project/client attribution feeding an already-existing downstream classification step, not by Anchor performing the classification itself.

## Consequences

- `docs/vision/vision.md` "Success looks like" and `docs/product/mvp.md` "Success criteria" were amended the same day to describe Anchor's job as project/client attribution, not billable/non-billable reflection — both previously implied Anchor did this classification itself, which was never actually true of the accepted Time Block model.
- `docs/product/users.md` "Needs & pain points" was amended to clarify the billable/non-billable need is met via this downstream path, not an in-app feature.
- Task Templates (`docs/product/features/task-templates.md`) carry only name/project/client, with no billable default — consistent with this decision.
- If Anchor is ever used by someone without an existing downstream billing process that does this classification, this decision would need revisiting (new ADR) — that's explicitly out of scope for now per `docs/vision/vision.md`'s non-goals (personal use only).
