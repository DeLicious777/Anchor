---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/mvp.md, docs/product/features/interruption-stack.md, docs/product/features/task-templates.md, docs/risks.md, docs/decisions/0003-billable-classification-out-of-scope.md]
---

# Export (XLSX / JSON)

> Created via `/new-feature export`. Depends on `docs/product/features/interruption-stack.md` (Time Block model, dashboard) and `docs/product/features/task-templates.md` (canonical naming). Following `.claude/workflows/design.md`.

> **Revised (2026-07-29) — [ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md), open item 7.** Terminology and one incorrect acceptance criterion; **no behaviour changed and no decision was revisited.** Three things:
>
> 1. **`completion_reason` is superseded by three fields.** Full-fidelity JSON (rounding off) carries all three — `end_determination`, `capture_origin`, `interruption_outcome` — because `docs/vision/vision.md` requires Capture Rate to be computable from exported data and this is the only artifact that can carry it. It deliberately does **not** carry `DerivedInterruptionStatus`: that projection is computed against the *current* interruption stack, so embedding it would make an export of last Tuesday yield different values depending on when it ran — unacceptable in an artifact that must be reproducible for billing. The projection is for display surfaces only.
> 2. **Log vs. timeline wording corrected.** This doc used "the underlying stored timeline (the append-only log…)" as if the two were the same thing. They are not, and the distinction is now load-bearing: **the transition log is the source of truth; the timeline is a projection replayed from it** (`docs/architecture/constraints.md`). Export reads the projection and writes to neither.
> 3. **One acceptance criterion was factually wrong** — "unchanged, byte-for-byte, after any export." See its replacement in Acceptance Criteria.
>
> Unchanged and still correct: grouped output (XLSX, and JSON with rounding on) carries no per-block metadata, and **none of the three fields may ever become an aggregation key** — that would silently change billed totals (`docs/risks.md` R2). Each export mode has one job: grouped output is intentionally lossy and billing-oriented; full-fidelity output is the analysis artifact.

## Problem

At the end of a day (or occasionally a longer range), the author needs the tracked timeline turned into billing-usable output without post-processing it elsewhere — per `docs/vision/vision.md`'s "produces data the author would actually trust and use for billing **without post-processing in Excel or any other external tool**" (citation updated 2026-07-28; the criterion previously read "without hand-editing," which the reconstruction workspace made false as written — editing inside Anchor is now intended, editing *outside* it is what this feature prevents). A real workday produces many short, fragmented Time Blocks per task (interruptions split a task's tracked time into several pieces across the day) — exporting these raw fragments directly isn't billing-usable without consolidation and rounding to a standard billing increment, which is exactly the "manual reconciliation" this project exists to eliminate.

## Goals

- Picking a date range (defaulting to the common case — today) and exporting produces a single flat XLSX worksheet, one row per unique task in that range, combined and rounded — ready to bill from without further editing in Excel.
- JSON export is available from the same underlying data, independently controllable for rounding, for future integrations/tooling that need more granular fidelity than the billing-oriented XLSX view.
- Export never writes: it appends nothing to the transition log and mutates no Time Block. Grouping and rounding are read-only, export-time transformations over the timeline projection, always computed fresh from the replayed state.
- Ties to `docs/vision/vision.md`'s "minimal manual effort, entirely inside Anchor" (revised 2026-07-28) and to closing the practical gap in R2 (`docs/risks.md`) that Task Templates only partially addressed.

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
3. **User-configurable rounding interval, toggleable on/off, remembered as a persisted export setting across sessions.** **Chosen.** Rounding always rounds *up* to the next interval boundary (ceiling — e.g. 1 minute at a 15-minute interval becomes 15 minutes), applied to the already-combined per-task total for XLSX. *(**Made explicit 2026-08-06, after the implementation disagreed with this sentence in a real export.** "Always" means **any** total greater than zero, including one shorter than a second: a 622 ms Time Block bills one full interval, not nothing. Only a total of *exactly* zero stays zero. The code truncated durations to whole seconds before testing for zero, so sub-second work reached the zero guard already destroyed and was billed as 0 — a silently wrong billed total — `docs/risks.md` **R2**'s shape arriving through a different mechanism than R2 describes — and an **R11** instance in that this rule was accepted and never verified against the code. Fixed by summing and rounding in milliseconds; the guard itself was always correct.)*

**JSON export shape:**
1. Raw, one entry per stored Time Block, with rounding applied independently per record's duration — **rejected** after review: this diverges numerically from XLSX's summed-then-rounded total for the same data (e.g. three 5-minute fragments of one task, 15-minute interval: XLSX sums to 15 minutes then rounds once → 15 minutes; independent per-record rounding would ceiling each fragment separately → 45 minutes if summed downstream). A ~3x divergence between export formats for identical underlying data, surfaced nowhere to the user, directly threatens "data the author would actually trust... for billing." This was initially proposed as an unconfirmed default design call, then rejected once the author confirmed the actual intent (2026-07-23).
2. Same grouped-by-task structure as XLSX (sum matching Time Blocks first, then round once) — **chosen**, per explicit author decision (2026-07-23): "JSON and other maybe later added export options should use the 'first sum then round' approach." This applies only when rounding is enabled; see Technical Constraints for how full fidelity is preserved when rounding is off.

## JSON rounding-on vs. rounding-off shape

Because rounding now means "sum matching Time Blocks, then round once" everywhere (not just XLSX), JSON's actual shape depends on whether rounding is enabled for a given export:

- **Rounding disabled**: JSON stays a raw list, one entry per stored Time Block (full fidelity — individual start/end plus all three metadata fields), since there's nothing to sum.
- **Rounding enabled**: JSON becomes grouped by task (name/project/client) exactly like XLSX — one entry per unique task in range, with a single summed-then-rounded duration — since summing is what "first sum then round" requires. Per-Time-Block granularity (individual start/end and the three metadata fields) is not present in this mode, for either format, by the same logic.

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
- Resulting XLSX: one worksheet, columns for task name, project, client, and total duration (rounded or exact per the current setting). No metadata columns — XLSX is the grouped, billing-oriented view, where an individual Time Block's metadata isn't meaningful once summed with others; that detail stays in JSON (rounding off).
- Resulting JSON (rounding off): an array of Time Block records (name, project, client, start, end, exact duration, plus `end_determination`, `capture_origin`, and `interruption_outcome`). A still-active entry has no `end_determination` yet — see Technical Constraints.
- Resulting JSON (rounding on): an array of grouped task totals (name, project, client, summed-then-rounded duration) — same shape as the XLSX rows, serialized as JSON instead.
- A still-active (in-progress) task included in the export range shows its elapsed-so-far duration (start to export time), computed live and never written back to storage — see Technical Constraints.

## Technical Constraints

- **Export performs no writes.** It appends no record to the transition log ([ADR 0004](../../decisions/0004-transition-log-format-and-torn-write-scheme.md)) and mutates no Time Block in the timeline projection. Grouping and rounding are read-only computations, recomputed fresh on every export, never cached back into storage. _(Wording corrected 2026-07-29: this previously called the timeline "the append-only log" and cited ADR 0002. The log and the timeline are distinct — the log is the source of truth, the timeline is a projection replayed from it — and the log format is decided in ADR 0004, not 0002.)_
- **Full-fidelity JSON emits records in ascending `start` order.** _(Added 2026-07-29; requirement identified in `timeline-reconstruction.md`'s design pass, recorded here because this doc owns export behaviour.)_ It currently emits in the order blocks were closed, which happens to be start-order today only because the state machine always closes the active block at the instant the next one starts. Timeline reconstruction breaks that coincidence permanently: a block added at 16:00 for a 09:00 span is closed last and would emit last. Sorting is a read-time transformation over a copy and does not touch the **no-writes** invariant above. Grouped output is unaffected — it has no per-block records to order. The History View already sorts by `start` and needs no change.
- The task-grouping key for XLSX is an exact match on (name, project, client) — the same aggregation convention already established in `docs/concept/concept.md`/`docs/product/mvp.md`, and the same one Task Templates (`docs/product/features/task-templates.md`) partially mitigates drift for. This feature is where that previously-abstract aggregation approach actually gets implemented.
- Rounding is ceiling-based: any non-zero duration rounds up to the next multiple of the configured interval (e.g. 1 minute at a 15-minute interval → 15 minutes; 16 minutes at a 15-minute interval → 30 minutes). Always applied to the *combined* per-task total (sum matching Time Blocks first, then round once) — for both XLSX and rounding-enabled JSON. When rounding is disabled, JSON stays raw (no summing, no rounding); XLSX always sums regardless of rounding, since it always shows a single total-duration column per task.
- A Time Block belongs to whichever range it falls into based on its **start** time — not its end time, and not any notion of "the day it mostly happened on." This fully resolves the "task starting just before midnight" case (it belongs to the day it started). The one remaining genuinely open question is timezone/DST handling around a range boundary — not the inclusion rule itself, which is decided above.
- A still-active entry (no end time yet, still being tracked) that falls within the selected range **is included**, using its elapsed-so-far duration (start to the moment of export) as a live, read-only computation — never written back to the stored timeline. Such an entry has no `end_determination` yet — an end time that hasn't happened has no determination — so in JSON with rounding off that field is absent for that one record. In XLSX and rounding-enabled JSON, its provisional duration is simply summed into the task's total like any other matching Time Block — not because of anything specific to being active, but because grouped mode never carries per-block metadata for any entry, active or not.
- The rounding preference (enabled/disabled, interval value) is a durably persisted user setting, following the same durability principle as the rest of the app ([ADR 0002](../../decisions/0002-desktop-app-framework-and-platform.md)) — not something that resets between sessions.
- No enforcement of `recovered-gap` review before export (see Alternatives) — a conscious non-mitigation of risk R4, not an oversight.

## Acceptance Criteria

- Selecting "Today" (default) and exporting XLSX produces a single worksheet with exactly one row per unique (name, project, client) combination among Time Blocks whose **start** falls within the current day.
- Three Time Blocks of the same task (name/project/client), two adjacent and one separated by a different interrupting task in between, all combine into one summed-then-rounded row — not three, not two.
- With rounding enabled at a 15-minute interval, a task whose combined duration is 1 minute exports as 15 minutes; 16 minutes exports as 30 minutes — in both XLSX and JSON.
- **A task whose combined duration is greater than zero never exports as zero.** With rounding enabled at 15 minutes, a task that ran for 622 ms exports as 15 minutes, and ten separate 900 ms blocks of one task export as 15 minutes rather than 0 — the sum is taken over exact durations, and only a total of exactly zero stays zero. *(Added 2026-08-06 from a real export that billed 45 minutes where 60 was owed. Both halves are regression-tested, including the exact five blocks that exposed it.)*
- **"First sum then round" means the sum is over exact durations, not over per-block truncated seconds.** Truncating each block before summing loses up to a second per block and under-reports in proportion to how fragmented the day was — worst on precisely the interrupted days this product exists for.
- With rounding disabled, exported XLSX durations exactly match the unrounded sum of the underlying Time Blocks; XLSX always shows one summed row per task regardless of the rounding setting.
- JSON export with rounding disabled reproduces the exact stored duration for every Time Block, one entry per Time Block, not grouped.
- JSON export with rounding enabled produces the same grouped, summed-then-rounded totals as the equivalent XLSX export for the same range and interval — the two numerically agree, and JSON's shape switches to grouped-by-task (no `start`/`end` and no per-block metadata) rather than staying a flat per-Time-Block list.
- Full-fidelity JSON export emits records in ascending `start` order, including after a block has been added covering a span earlier than blocks recorded before it.
- A task still actively running at export time, within the selected range, is included with its elapsed-so-far duration computed as of the export moment; it still has no end time afterward, because export wrote nothing.
- The rounding on/off setting and interval persist across an app restart and are applied by default to the next export until explicitly changed.
- Exporting a range containing unreviewed `recovered-gap` entries completes without any blocking prompt or warning.
- **Export itself performs no writes**: an export appends no record to the transition log and mutates no Time Block. Grouping and rounding never write back.

  _(Replaced 2026-07-29. The previous criterion read "the underlying stored timeline is unchanged, byte-for-byte, after any export" — **factually false**: while a task is active, the 60-second heartbeat legitimately appends to the transition log, and it may do so mid-export. The log file's bytes therefore can and do change across an export, through no fault of export's. The existing test passes only because no heartbeat timer runs in it, which means it was asserting something narrower than it claimed. The invariant that actually matters, and that export can actually guarantee, is about what **export** does — not about what unrelated concurrent processes may do.)_

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
