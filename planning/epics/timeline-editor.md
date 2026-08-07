---
status: planned
date: 2026-08-07
owner: erich
related: [docs/product/features/timeline-editor.md, docs/product/features/visual-redesign.md]
---

# Epic: Timeline Editor

A proportional graphical timeline beside the History View — supplying the clamping half of *"the editor clamps; the domain rejects"*, and making a wrong duration visible as a wrong-sized shape rather than a number to be read.

## Source docs

- Feature: [`timeline-editor.md`](../../docs/product/features/timeline-editor.md) — `accepted` 2026-08-06
- Visual foundation: [`visual-redesign.md`](../../docs/product/features/visual-redesign.md)

## Current implementation state

**Not started.** No timeline surface exists. `moveBlock` and `addBlock` exist in `app/src/lib/api.ts` with no caller; `resizeBlock` gained one on 2026-08-06 — the History View's numeric correction, which closed **R9**.

## Remaining work

1. **The shared view range — store, persistence and get/set commands only.** Backend-owned, following the `paths.rs` four-files-one-concern pattern. **Separable and unblocked**, and an *enabling* slice: it ships no visible behaviour.
1a. **The range control and History View filtering.** *(Split out 2026-08-07.)* Deliberately **not** in the unblocked slice: `timeline-editor.md` decision 2 justifies range-scoping the History View solely by the need for two adjacent views to agree, so shipping the filter before the timeline exists would deliver the whole cost — history hidden by default on a shipped surface — and none of the reason. Lands with items 2–4.
2. The vertical 96 px column, the 8-hour Today viewport and fit-to-range mode, scroll-and-pin, the 480 px minimum column height with a window `minHeight`. *(Asset-blocked.)*
3. Proportional rendering with **no minimum extent**, clustering for dense runs, the three-lane marker gutter. *(Asset-blocked.)*
4. Zone-based Add/Move/Resize, clamping with pointer decoupling, the numeric selection panel. *(Asset-blocked.)*

## Dependencies and blockers

- **External:** the design-system assets — spacing scale steps, hue palette, font weights. Explicitly a prerequisite of *implementing*, never of accepting.
- **Item 1 is unblocked today**, and unlocks the Interruption History epic's "Earlier".
- `app/src/routes/+page.svelte` is git-binary (NUL bytes in an `R.uniqBy` key), so this work gets no line-level merges.

## Child issues

- **#25 — shared view range store and get/set commands.** Unblocked; item 1 above. **The range control and History View filtering (item 1a) are explicitly not in it** and land with items 2–4.
- #14 — the epic issue. Retains the asset-blocked surface work.

## Exit criteria

`timeline-editor.md`'s Acceptance Criteria, notably: proportionality with **no exception**; a cluster states its member count and reveals every member when opened; no sub-floor element is ever offered as a pointer target; changing the view range does not change what a subsequent export contains.

## Can it progress without the design-system assets?

**Partly — item 1 only.**
