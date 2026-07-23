---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/product/features/task-templates.md, docs/risks.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Export (XLSX / JSON)

> Created via `/new-feature export`. Depends on `docs/product/features/interruption-stack.md` (Time Block model, dashboard) and `docs/product/features/task-templates.md` (canonical naming). Following `.claude/workflows/design.md`.

## Problem

At the end of a day (or occasionally a longer range), the author needs the tracked timeline turned into billing-usable output without hand-editing — per `docs/vision/vision.md`'s "exporting to XLSX/JSON produces data the author would actually trust and use for billing, without hand-editing." A real workday produces many short, fragmented Time Blocks per task (interruptions split a task's tracked time into several pieces across the day) — exporting these raw fragments directly isn't billing-usable without consolidation and rounding to a standard billing increment, which is exactly the "manual reconciliation" this project exists to eliminate.

## Goals

- Picking a date range (defaulting to the common case — today) and exporting produces a single flat XLSX worksheet, one row per unique task in that range, combined and rounded — ready to bill from without further editing in Excel.
- JSON export is available from the same underlying data, independently controllable for rounding, for future integrations/tooling that need more granular fidelity than the billing-oriented XLSX view.
- The underlying stored timeline (`docs/product/features/interruption-stack.md`) is never modified by export — grouping and rounding are read-only, export-time transformations, always computed fresh from the exact stored data.
- Ties to `docs/vision/vision.md` "no manual reconciliation" and to closing the practical gap in R2 (`docs/risks.md`) that Task Templates only partially addressed.

## Users

Serves the same primary persona as `docs/product/users.md` and the other accepted feature docs — no new segment.

## Alternatives

**Date range selection:**
1. Always export the entire timeline, no range — simplest, but ignores the stated primary use case and forces the user to discard irrelevant rows themselves. Rejected.
2. **Range picker with presets (Today, This Week, custom), defaulting to Today** — matches the stated primary use case (mostly current day) while still supporting less frequent longer-range exports. **Chosen.**
3. Fixed weekly/monthly export only, no custom range — too rigid for an ad hoc range request (e.g. a client asking for a specific week mid-month). Rejected.

**Excel row structure:**
1. One row per raw Time Block, no combination — simplest to build, but directly contradicts the stated need: a day with several short interruptions on the same task would show multiple sub-15-minute fragments, requiring the user to manually combine and round them in Excel — exactly the "manual reconciliation" this feature exists to eliminate. Rejected.
2. Two sheets: raw detail sheet plus an aggregated summary sheet — gives both views, but the author explicitly rejected a second/summary sheet. Rejected.
3. **A single flat worksheet, one row per unique task (name + optional project/client) within the selected range; all matching Time Blocks are summed into one duration before rounding is applied.** **Chosen** — matches the author's explicit description exactly. "No grouping" (per the author) means no *separate summary sheet*, not "never combine rows" — the one sheet that exists does the combining itself.

**Rounding strategy:**
1. No rounding, exact durations only — doesn't match the stated billing convention. Rejected.
2. A fixed rounding interval (e.g. always 15 minutes), not configurable — simpler to build, but the author explicitly wants the interval configurable (5/10/15-minute examples given, "not fixed to a specific value"). Rejected.
3. **User-configurable rounding interval, toggleable on/off, remembered as a persisted export setting across sessions.** **Chosen.** Rounding always rounds *up* to the next interval boundary (ceiling — e.g. 1 minute at a 15-minute interval becomes 15 minutes), applied to the already-combined per-task total for XLSX.

**JSON export shape:**
1. Raw, one entry per stored Time Block, with rounding applied independently per record's duration — **rejected** after review: this diverges numerically from XLSX's summed-then-rounded total for the same data (e.g. three 5-minute fragments of one task, 15-minute interval: XLSX sums to 15 minutes then rounds once → 15 minutes; independent per-record rounding would ceiling each fragment separately → 45 minutes if summed downstream). A ~3x divergence between export formats for identical underlying data, surfaced nowhere to the user, directly threatens "data the author would actually trust... for billing." This was initially proposed as an unconfirmed default design call, then rejected once the author confirmed the actual intent (2026-07-23).
2. Same grouped-by-task structure as XLSX (sum matching Time Blocks first, then round once) — **chosen**, per explicit author decision (2026-07-23): "JSON and other maybe later added export options should use the 'first sum then round' approach." This applies only when rounding is enabled; see Technical Constraints for how full fidelity is preserved when rounding is off.

## JSON rounding-on vs. rounding-off shape

Because rounding now means "sum matching Time Blocks, then round once" everywhere (not just XLSX), JSON's actual shape depends on whether rounding is enabled for a given export:

- **Rounding disabled**: JSON stays a raw list, one entry per stored Time Block (full fidelity — individual start/end/completion reason), since there's nothing to sum.
- **Rounding enabled**: JSON becomes grouped by task (name/project/client) exactly like XLSX — one entry per unique task in range, with a single summed-then-rounded duration — since summing is what "first sum then round" requires. Per-Time-Block granularity (individual start/end/completion reason) is not present in this mode, for either format, by the same logic.

This means XLSX and rounding-enabled JSON are now guaranteed to agree numerically for the same range and interval — the divergence risk above is closed by construction, not by a disclaimer.

**Recovered-gap review enforcement at export time:**
1. Block or warn on export if the selected range contains any unreviewed `recovered-gap` entries — would concretely close risk R4's "nothing enforces review" gap (`docs/risks.md`).
2. **No enforcement — export proceeds regardless of unreviewed `recovered-gap` entries; reviewing before exporting is the author's own workflow discipline, not something Anchor checks.** **Chosen**, per explicit author decision (2026-07-23): "I will have reviewed it before export." This leaves risk R4 open, on the same basis R3 is already accepted elsewhere in this project (see `docs/risks.md`).

## Trade-offs

| | Date range | Excel row structure | Rounding | JSON shape | Gap-review enforcement |
|---|---|---|---|---|---|
| **Chosen** | Range picker, presets, default Today | Single sheet, grouped-then-rounded per task | Configurable interval, toggleable, ceiling rounding | Same sum-then-round logic as XLSX when rounding is on; raw per-record when rounding is off | None — author's own discipline |
| Complexity | Low — a date filter over existing data | Moderate — grouping/summing logic before rounding | Low — a persisted setting plus a ceiling-rounding function | Low — reuses the same grouping/rounding computation as XLSX, just serialized differently | None |
| Reversibility | Presets can be added/changed later without affecting stored data | Grouping key (name/project/client) matches the already-accepted aggregation convention — consistent, but any future move to a first-class Task entity (see R2) would need this logic revisited | Interval/default can change later; ceiling-vs-nearest could be revisited without data changes | A distinct "always raw, even when rounded" JSON mode could be added later without breaking this one, if an integration turns out to need per-fragment rounded data | Could be added later (a warning, not a data-model change) if R4 proves costly in practice |
| Risk if wrong | Presets that don't match real usage patterns are just a UI annoyance, not a data risk | If the grouping key (exact name/project/client match) doesn't catch a rename mid-range, that specific task under-reports rather than double-counts — same failure mode as R2 | Rounding compounds R4: a `recovered-gap` entry with a wrong inferred duration gets rounded and combined into a task total, making the error harder to spot after export than before it | None remaining — XLSX and rounding-enabled JSON now agree numerically by construction | R4 stays open by design — a `recovered-gap` entry billed with the wrong duration is now the author's sole responsibility to catch before export, not Anchor's |

## UX

- Export is triggered from the existing **Dashboard** (`docs/product/features/interruption-stack.md`), not a new top-level surface.
- A **range picker** (Today [default], This Week, custom start/end) determines what gets exported.
- A **rounding control**: an on/off toggle plus an interval selector (e.g. 5/10/15 minutes, or a custom value — a positive whole number of minutes; zero, negative, or fractional values are rejected by the input itself) — both persisted across sessions as the default for the next export until changed.
- Two distinct actions, **Export XLSX** and **Export JSON**, both respecting the same range and rounding settings. XLSX always groups (rounded or not, since a total-duration column is always shown); JSON groups the same way *only* when rounding is enabled, and stays raw per-Time-Block when rounding is disabled (see "JSON rounding-on vs. rounding-off shape" above).
- Resulting XLSX: one worksheet, columns for task name, project, client, and total duration (rounded or exact per the current setting). No `completion_reason` column — XLSX is the grouped, billing-oriented view where an individual Time Block's completion reason isn't meaningful once summed with others; that detail stays in JSON (rounding off).
- Resulting JSON (rounding off): an array of Time Block records (name, project, client, start, end, exact duration, completion reason — or no completion reason if the entry is still active, see Technical Constraints).
- Resulting JSON (rounding on): an array of grouped task totals (name, project, client, summed-then-rounded duration) — same shape as the XLSX rows, serialized as JSON instead.
- A still-active (in-progress) task included in the export range shows its elapsed-so-far duration (start to export time), computed live and never written back to storage — see Technical Constraints.

## Technical Constraints

- The underlying stored timeline (the append-only log, `docs/product/features/interruption-stack.md` / [ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md)) is never mutated by export — grouping and rounding are read-only computations over the stored data, recomputed fresh on every export, never cached back into storage.
- The task-grouping key for XLSX is an exact match on (name, project, client) — the same aggregation convention already established in `docs/concept/concept.md`/`docs/product/mvp.md`, and the same one Task Templates (`docs/product/features/task-templates.md`) partially mitigates drift for. This feature is where that previously-abstract aggregation approach actually gets implemented.
- Rounding is ceiling-based: any non-zero duration rounds up to the next multiple of the configured interval (e.g. 1 minute at a 15-minute interval → 15 minutes; 16 minutes at a 15-minute interval → 30 minutes). Always applied to the *combined* per-task total (sum matching Time Blocks first, then round once) — for both XLSX and rounding-enabled JSON. When rounding is disabled, JSON stays raw (no summing, no rounding); XLSX always sums regardless of rounding, since it always shows a single total-duration column per task.
- A Time Block belongs to whichever range it falls into based on its **start** time — not its end time, and not any notion of "the day it mostly happened on." This fully resolves the "task starting just before midnight" case (it belongs to the day it started). The one remaining genuinely open question is timezone/DST handling around a range boundary — not the inclusion rule itself, which is decided above.
- A still-active entry (no end time yet, still being tracked) that falls within the selected range **is included**, using its elapsed-so-far duration (start to the moment of export) as a live, read-only computation — never written back to the stored timeline. Such an entry has no `completion_reason` yet (it hasn't completed): in JSON with rounding off, this shows up as the field being absent for that one record. In XLSX and rounding-enabled JSON, its provisional duration is simply summed into the task's total like any other matching Time Block — not because of anything specific to being active, but because grouped mode never carries `completion_reason` for any entry, active or not.
- The rounding preference (enabled/disabled, interval value) is a durably persisted user setting, following the same durability principle as the rest of the app ([ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md)) — not something that resets between sessions.
- No enforcement of `recovered-gap` review before export (see Alternatives) — a conscious non-mitigation of risk R4, not an oversight.

## Acceptance Criteria

- Selecting "Today" (default) and exporting XLSX produces a single worksheet with exactly one row per unique (name, project, client) combination among Time Blocks whose **start** falls within the current day.
- Three Time Blocks of the same task (name/project/client), two adjacent and one separated by a different interrupting task in between, all combine into one summed-then-rounded row — not three, not two.
- With rounding enabled at a 15-minute interval, a task whose combined duration is 1 minute exports as 15 minutes; 16 minutes exports as 30 minutes — in both XLSX and JSON.
- With rounding disabled, exported XLSX durations exactly match the unrounded sum of the underlying Time Blocks; XLSX always shows one summed row per task regardless of the rounding setting.
- JSON export with rounding disabled reproduces the exact stored duration for every Time Block, one entry per Time Block, not grouped.
- JSON export with rounding enabled produces the same grouped, summed-then-rounded totals as the equivalent XLSX export for the same range and interval — the two numerically agree, and JSON's shape switches to grouped-by-task (no `start`/`end`/`completion_reason` per record) rather than staying a flat per-Time-Block list.
- A task still actively running at export time, within the selected range, is included with its elapsed-so-far duration computed as of the export moment; the underlying stored timeline shows no end time for it afterward (nothing was written).
- The rounding on/off setting and interval persist across an app restart and are applied by default to the next export until explicitly changed.
- Exporting a range containing unreviewed `recovered-gap` entries completes without any blocking prompt or warning.
- The underlying stored timeline is unchanged, byte-for-byte, after any export — grouping and rounding never mutate stored Time Blocks.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
