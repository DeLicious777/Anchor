---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/risks.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Task Templates

> Created via `/new-feature task-templates`. Depends on `docs/product/features/interruption-stack.md` (`status: accepted`) for the Time Block model and quick-input UX it extends. Following `.claude/workflows/design.md`.

## Problem

Recurring activities (a daily standup, a sprint retro, ongoing work for a specific client) require re-entering the same name/project/client every time via the quick-input (`docs/product/features/interruption-stack.md` UX), adding friction that compounds across a day and a week. This friction plausibly erodes tracking discipline over time — directly related to risk R3 (`docs/risks.md`, manual-tracking depends on user discipline) — and inconsistent manual entry of the same recurring activity's name/project/client is exactly the failure mode risk R2 describes (export-time aggregation fragmenting on naming drift).

## Goals

- Starting a recurring activity takes one selection (template) instead of manually typing name/project/client each time.
- Templates reduce *entry-time* drift (typos, inconsistent naming) for a recurring activity's name/project/client, partially mitigating R2's aggregation-fragmentation risk (`docs/risks.md`) going forward. This is a partial mitigation, not a resolution: editing a template later still doesn't retroactively update already-recorded Time Blocks (see Technical Constraints), so template renames can still reproduce the same fragmentation R2 describes.
- Ties to `docs/vision/vision.md` "Success looks like": lower-friction capture supports "no manual reconciliation" by reducing a source of inconsistent data entry.

## Users

Serves the same primary persona as `docs/product/users.md` and `docs/product/features/interruption-stack.md` — no new segment.

## Alternatives

**Template invocation:**
1. Dedicated hotkeys per template (e.g. one key bound directly to "Daily Standup") — fastest for a small fixed set, but doesn't scale past a handful of hotkeys and adds hotkey-management complexity on top of `interruption-stack.md`'s already-required hotkey remapping/conflict-detection UI.
2. **Autocomplete/search within the existing quick-input** (type a few letters, pick the matching template) — no new UI surface, reuses the quick-input already specified in `interruption-stack.md`; scales to any number of templates without hotkey-space pressure. **Chosen** — user decision (2026-07-23): templates are invoked purely through quick-input autocomplete, no dedicated per-template hotkeys.
3. A separate "template picker" popup/menu, distinct from the quick-input — adds a new UI surface and a new decision point ("do I type or do I open the picker") for no clear benefit over (2).

**Template field scope:**
1. Name/project/client only, matching the Time Block fields already defined in `docs/concept/concept.md` — no new fields introduced.
2. Name/project/client plus a billable/non-billable flag — **rejected**: no such field exists on Time Block; billable classification is explicitly out of scope for Anchor's data model (see [ADR 0003](../../decisions/0003-billable-classification-out-of-scope.md)) since it happens downstream once the exported timeline is transferred into the author's existing billing process.
3. Name/project/client plus a default duration or reminder — not requested, and would add scope (a scheduling/reminder concept) beyond what any accepted doc calls for. **Rejected** as unrequested scope expansion.

## Trade-offs

| | Template invocation | Template field scope |
|---|---|---|
| **Chosen** | Autocomplete in existing quick-input | Name/project/client only |
| Complexity | Low — extends an existing UI surface rather than adding one | Lowest possible — zero new fields, zero new Time Block schema changes |
| Reversibility | Dedicated hotkeys could be added later for a small "favorites" subset without changing the underlying template data | Adding a field later (e.g. if billable tracking is ever brought in-app) doesn't invalidate existing templates — they'd just lack the new field until edited |
| Risk if wrong | If the user ends up with many templates, autocomplete search quality (fuzzy match, ranking) becomes load-bearing for speed — not yet specified, see Technical Constraints | Renaming a template after it's been used doesn't propagate to past Time Blocks (see Technical Constraints) — this reproduces R2's fragmentation risk on a template rename, just less often than free-text entry. Not eliminated, only reduced. |

## UX

- **Dashboard** (`interruption-stack.md`'s existing dashboard window) gains a Task Template management section: create, edit, delete templates (name, optional project, optional client).
- **Quick-input** (triggered by Switch/Interrupt hotkeys, per `interruption-stack.md`): typing autocompletes against both free text and existing templates; selecting a template pre-fills name/project/client, which the user can still edit before confirming — templates are a starting point, not a lock.
- Empty state: dashboard's template section shows "No templates yet" with a create action, when none exist.
- Two templates may share the same name with different project/client (e.g. "Standup" for two different clients) — autocomplete must show project/client alongside the name so these are visually distinguishable, not just the name alone.
- No change to the Return-to-Previous/Return-to-Original flows — templates only affect how a *new* Switch/Interrupt is named, not how the stack unwinds.

## Technical Constraints

- Templates are stored separately from the Time Block timeline (a template is a reusable preset, not a tracked event) — persisted durably (same append-only/durable-write principle as `interruption-stack.md`, since losing a template silently would reintroduce the naming-drift risk R2 this feature exists to mitigate), but not part of the timeline replay itself.
- Quick-input autocomplete must rank/filter templates in a way that stays fast (no perceptible lag) as the template count grows across real usage — exact matching/ranking algorithm is an implementation detail, not specified further here.
- Editing an existing template's name/project/client does **not** retroactively change any already-recorded Time Block that used the old values — per the accepted flat-timeline model (`docs/concept/concept.md`, `docs/product/mvp.md`), Time Blocks are independent entries with no persistent link back to the template that created them. This is a direct consequence of the already-accepted "no persistent Task entity" decision, not a new one, but worth stating explicitly here since it's easy to assume otherwise.
- Framework/persistence mechanics (Tauri/Rust, append-only log) follow [ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md) — no new architecture decision needed for this feature.

## Acceptance Criteria

- Creating, editing, and deleting a template in the dashboard persists durably and survives an app restart.
- Selecting a template via quick-input autocomplete pre-fills name/project/client into a new Switch or Interrupt, editable before confirming.
- Editing or deleting a template has no effect on Time Blocks already recorded using its previous values.
- With at least 20 templates created, autocomplete filtering responds in under 100ms per keystroke in the quick-input.
- No dedicated hotkey UI exists for individual templates — template selection is exclusively via quick-input autocomplete.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
