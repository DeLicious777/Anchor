---
status: accepted
date: 2026-08-06
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/principles.md, docs/risks.md, docs/assumptions.md, docs/glossary.md, docs/product/features/visual-redesign.md, docs/product/features/timeline-reconstruction.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0006-stable-persistent-time-block-identity.md, docs/decisions/0007-auto-resume-after-a-short-gap.md, ideas/adjustable-timeline-view.md, ideas/manual-time-block-entry.md]
---

# Timeline Editor

> Created 2026-08-04. Follow `.claude/workflows/design.md`.
>
> **Independently reviewed five times** — four on 2026-08-04, a fifth on 2026-08-06. `.claude/workflows/design.md` budgets for "however many rounds it takes," and this doc took five.
>
> - **Pass 1 — 22 must-fixes.** Six changed decisions; three were false claims about shipped code (table below).
> - **Pass 2 — 8 new must-fixes, every one created by pass 1's two largest fixes**: the numeric fallback (G.3) and the shared range (decision 2). The range gained the Alternatives entry it never had (**Alternative L**) and **its chosen option changed** — export is no longer coupled.
> - **Pass 3 — 6 new must-fixes, and a named pattern**: *each round's fix lands in the paragraph the review pointed at and stops there.* Handle geometry was fixed without asking whether the same arithmetic hit the block **body**; the range option changed without sweeping Technical Constraints; two docs were amended and two others carrying the same retracted sentence were not.
>
> **Round 4 began with a repository-wide sweep rather than an edit**, which is what the pattern called for. Two decisions moved as a result: membership gained a companion rule, **occupancy governs the canvas** (decision 2), because start-membership alone drew occupied time as free space; and the rendered floor rose from 12 px to 24 px. **Pass 4 then killed the floor entirely** — it found that a floor and a constant scale are jointly unsatisfiable for a *run* of short blocks, a debt every draft had missed by reasoning about one block in isolation. Proportionality is now absolute and *interaction* degrades instead, via clustering (decision 5).
>
> **Pass 5 (2026-08-06) changed no decision. It found one design fault and four documentation faults, and every one of the four was caused by shipping code rather than by the design.** **R9** was closed in the same session by wiring `resize_block` into the History View's edit row, which falsified this doc's Problem statement ("impossible to correct anywhere"), its Goals note, and Alternative G.3's "**not** a second editing surface" — all three amended and marked below. That change also shifted every `+page.svelte` line number cited here, two days after pass 1 corrected them; all are re-verified against the current file. Independently, decision 6 and Alternative H were still describing the **distortion mark** that decision 5 abolished, as were `glossary.md` and **R15**. The design fault: decision 1 introduced a window `minHeight` that no acceptance criterion tested, leaving the 1 px-per-minute scale — which every duration claim here rests on — unfalsifiable.
>
> **This is the graphical surface, not the domain.** The reconstruction operations it performs — Add, Move, Resize, Edit Identity, Delete — are designed in [`timeline-reconstruction.md`](timeline-reconstruction.md) and **already implemented and shipping** as commands. This document decides how a spatial surface drives them. It consumes that doc; it does not restate or redefine it.
>
> **Its visual foundation is [`visual-redesign.md`](visual-redesign.md)**, which is `accepted` and defers six things here: drag affordances and hit targets, how a clamp is communicated, zoom and time-range controls, minimum rendered block size, and orientation. Those are this doc's subject. What that doc fixed — a 24×24 CSS px hit floor at `comfortable` density, semantic tokens, non-colour encoding of provenance — is inherited.

## What the first review changed

Recorded because three of these were claims about the codebase that were wrong, and `principles.md` #8 makes that a finding rather than a nit.

| Claim as first written | What the code says |
|---|---|
| "`Add`, `Move` and `Resize` exist as commands with **collision clamping**" | The domain **rejects** overlaps (`stack.rs:63, 416`) and its own comment refuses to rely on editor clamping (`:369`). Clamping is unbuilt — supplying it is this feature's job. The corrected statement is a *stronger* argument for this feature. |
| "the timeline **follows the dashboard's range** — reuses an existing control" | There is no dashboard range. `resolveRange` (`+page.svelte:366`) is called only by `doExport` (`:404`); the History View renders `R.reverse(R.sortBy(start, view.closed))` (`:479`) — all history, unfiltered. The range had to be **created**, which is a behaviour change to a shipped surface (decision 2). |
| "`visual-redesign.md` E.3 classifies reconstruction as infrequent" | E.3 states the rule and deliberately does not apply it; that doc's acceptance criterion lists both classes and reconstruction appears in **neither**. Classified here instead (decision 8). |

Also corrected: `end_determination` renders at `+page.svelte:748-752` with its italic rationale at `:924-931`, not `:589-590` — a stale citation inherited from `visual-redesign.md:33`, which was accurate on 2026-08-01 and is now wrong in that doc too. **Fixed there in the same pass.** *(Re-cited again on 2026-08-06: the R9 wiring shifted every line below ~130 in that file, so these numbers moved a second time within two days of being corrected. Recorded because it is the strongest available evidence that **R11** is structural — these citations are not wrong through carelessness, they decay, and only a sweep against the current file catches it.)*

## Problem

The Timeline is the canonical projection of a day's work (`docs/glossary.md`), but the only surface rendering it is the **History View** — a table. A table is a good detail view and a poor shape view, and three accepted commitments depend on a shape view that does not exist:

1. **Wrong durations are hard to find in a table.** `docs/risks.md` **R4** is that a `SystemInferred` end silently mis-bills. The History View renders `end_determination` as the word `inferred` in italic (`+page.svelte:748-752`, styled `:924-931`) — honest, but it does not tell you *which* inferred end is wrong.

   *(**Narrowed 2026-08-06, and this is a genuine reduction in this feature's justification, not a rewording.** Until that date this point ended "and impossible to correct anywhere": `resizeBlock` existed in `app/src/lib/api.ts:96` with no caller, so **R9**'s mechanism gap was open and this surface was argued as where correction "becomes reachable at all." It is not. The History View's edit row now carries both boundaries and R9 is closed — which was always the right sequencing, since `timeline-reconstruction.md:16` had already argued the R8/R9 operations "can ship on the surface that already exists," and making a mis-billing risk wait on an unbuilt surface would have inverted the priority. **What survives is detection, not correction.** A numeric field can fix an end you already know is wrong; it cannot tell you which one is. That is the gap a proportional rendering closes, and it is a narrower claim than this document made for four review passes.)*

   **The size-based detection argument is narrower than earlier drafts claimed, and is corrected here.** A *routine* recovery is bounded to roughly one minute by the 60-second heartbeat (`risks.md` R4), which is about one pixel — invisible at any scale, and not what a proportional rendering finds. What it does find is the **unbounded** case: a crash or a sleep that ADR 0007 closes with an inferred end, where the error is hours and the block is visibly the wrong size or the hole beside it visibly the wrong shape. Stated precisely because the first draft asserted the general claim, and `interruption-stack.md`'s own "wildly wrong inferred duration" wording sits in tension with R4's one-minute bound — a disagreement between two accepted docs that this feature should not quietly build on.
2. **Half of an accepted invariant has no implementation.** `timeline-reconstruction.md` fixes the rule *"the editor clamps; the domain rejects."* The domain half ships: `stack.rs` returns `OverlapsExistingBlock` and its test is named `the_domain_rejects_every_overlapping_span_not_just_the_editor`. **The clamping half does not exist**, because the editor does not exist. Today every overlapping reconstruction is a rejection the user must interpret, where the design says it should have been a gesture that simply stopped.
3. **`visual-redesign.md` makes the Timeline the dashboard's primary subject** and notes the IA is "implementable today with only the History View" (`:199`) — deliberately, so the redesign could ship first. That was sequencing, not a claim that one view suffices.

Evidence is direct inspection of `+page.svelte`, `stack.rs`, `commands.rs`, and the accepted feature docs. The sole user is the author; no user research is involved or needed.

## Goals

Tied to `docs/vision/vision.md`'s criterion that the author can run a real workday through Anchor and **trust the result** — this feature serves the *trust* half.

- **A wrong duration is detectable by looking rather than reading, in Today mode.** Measurable: in Today mode, two blocks whose durations differ by 15 minutes or more render at visibly different extents, unless one of them is inside a cluster. *(Scoped to Today mode deliberately — see decision 4, which concedes this does not hold in range mode. The earlier phrasing, "wrong by more than a configured threshold," named a configuration that does not exist anywhere in the product.)*
- **Every reconstruction operation this surface owns — Add, Move and Resize — is reachable from it**, and the clamping half of the accepted invariant is supplied. Edit Identity and Delete are deliberately *not* here: they are row actions and already ship on the History View. This feature adds **zero** transitions, payloads, or persisted fields to the record. *(Amended 2026-08-06: **Resize now also ships on the History View**, numerically, which closed R9 ahead of this surface. That does not make it redundant here — a table cannot show the neighbour a boundary clamps against, which is why Resize is *judgeable* only on a timeline (`glossary.md`). **Add and Move remain reachable from nowhere else.**)*
- **Correction stays the exception path.** `timeline-reconstruction.md`: reconstruction "must never become the fast path" (`:42`).
- **The day's shape is legible at a glance**, including a day fragmented by many short interruptions.
- **The fast path is untouched.** No markup, handler, or command on the hotkey or widget path changes.

Explicit non-goals: no new reconstruction capabilities, no change to export output, and no activity inference ([ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md)).

## Users

The single segment in `docs/product/users.md`: **the interrupted billable developer**, currently the author alone.

- *"Values speed and low friction above all."* This surface is on the **deliberate** side of the split — the dashboard is "not meant for rapid interaction" (`interruption-stack.md`, Dashboard/UX section) — so it is judged on whether a correction is quick *once started*, not on whether it competes with the hotkey path.
- *"Comfortable with hotkeys and a command palette,"* explicitly not needing onboarding. No tutorial, no first-run tour, no empty-state guidance.

No new user segment is implied.

## Decisions taken (2026-08-04)

**1. Side by side, vertical timeline.** A vertical column beside the History View, both permanently visible, no toggle. Time runs top to bottom, so **the duration-carrying dimension is height** — every "minimum size" and "extends past a boundary" statement below is about the vertical axis. This settles the orientation question `visual-redesign.md` deferred here: **vertical, not user-selectable.** Two rendering modes plus a persisted setting is cost that "beside the table" does not earn back.

The timeline column is **96 CSS px wide**, fixed; the History View takes the remainder and keeps its existing `overflow-x: auto` (`+page.svelte:937-939`). The main window gains a **`minWidth` of 800** — its current default, which today is only a starting size and can be dragged below the point where the split works.

**The column's minimum *height* is 480 CSS px, and the window gains a `minHeight` to guarantee it.** *(Added on the fourth review, which found four passes had argued about pixel sizes while the number they all depended on — how tall the column is — appeared in no document, no ADR and no config. `tauri.conf.json` sets 800×600 and **no** `minHeight`.)* 480 px over an 8-hour viewport is **one pixel per minute**, which is the scale every duration statement in this document is implicitly quoted at, so it is now stated instead of assumed. Below it, clustering becomes the normal rendering rather than the dense-day rendering. *(Both numbers are unvalidated (`principles.md` #7): 96 px is the smallest column that fits a block plus its marker gutter at `comfortable` density. Revisit if either the gutter or a German label needs more.)*

**2. A shared *view* range across the timeline and the History View. Export keeps its own range.** *(Rewritten twice. See Alternative L, where this is argued.)* There is no dashboard range today — `resolveRange` serves `doExport` alone (`+page.svelte:289, 327`), and the History View is unfiltered (`:402`). So this decision creates a **view** range, and deliberately does not extend it to export.

- **A block belongs to a view range by its `start`**, exactly as `export.md:95` already decides for export. One membership rule across the product, and the case that rule exists for — a block starting just before a boundary — resolves identically on both surfaces.
- **Membership governs the *set*; occupancy governs the *canvas*.** *(Added on the third review, which found start-membership alone creates a phantom.)* A block that started before the range but is still running inside it is **not in the set** — so it is absent from the History View, correctly, and matches export. But its time **is occupied**, and a timeline that drew that interval as empty would be lying about the one thing a timeline is for: free space is the Add zone, so the user would draw into it and the domain would reject with `OverlapsExistingBlock` against a block nowhere on screen. That breaks the founding invariant that *"a correctly working editor should never trigger the rejection"* (`timeline-reconstruction.md:177`).

  So the canvas renders **any block whose span intersects the viewport**, and one that is out of range renders as **context**: visible, occupying its true interval, never free space, and never editable. This is also what resolves the overnight active block — started yesterday, running now — which start-membership would otherwise exclude from today while `timeline-reconstruction.md:201` requires it be rendered as occupied to the present moment. Context blocks are non-editable, and the active block is restricted anyway, so the two rules agree rather than compete.

  **Export needs no equivalent rule** and does not get one: export has no spatial adjacency, so nothing there can be misread as free.

  **A context block is a third class, and it must not look like the second.** *(Added on the fourth review.)* The surface now has three: **editable**, **restricted** (in range, blocked by domain state — active or open-frame, decision 7), and **context** (out of range, blocked by the view). The last two render identically unless something distinguishes them, and their remedies are opposite — *widen the range* versus *resolve the interruption* — so Alternative J's rationale, that a restriction is known before effort is spent, fails if the user cannot tell which one they are looking at. Three rules follow:

  - **The distinction is carried by the block's own treatment, not by the marker gutter.** The gutter reports on the *record* (provenance); this reports on the *view*, and the two must not share a channel or a restricted block's provenance becomes unreadable.
  - **All three classes are selectable.** Selection reveals exact times, which is decision 5's detail surface and is how a context block explains itself. **Selectable is not editable** — only the first class edits.
  - **Restricted-ness stays read from the projection; context-ness is computed in the UI from the view range**, and the two are deliberately separate. This is *not* the "third restricted class" Technical Constraints warns about: that warning is about the domain adding one and the editor failing to read it. Context-ness is not a domain fact at all — the domain has no opinion about what the user is looking at.
- **The view range persists across restarts** like every other durable preference. It is a view preference, and because it no longer reaches export, a persisted narrow range cannot pre-narrow a bill.
- **Persistence preserves selection intent, not resolved instants.** *(Confirmed by the author for #25 on 2026-08-07.)* The durable value is `today`, `this-week`, or `custom` with inclusive start/end calendar dates. `today` and `this-week` therefore resolve again relative to the current date after a restart instead of silently becoming a historical custom range; custom dates reopen unchanged.
- **Mode binding:** a range resolving to **exactly the current day** is Today mode; anything else is range mode. So `today` gives Today mode, `this-week` gives range mode, and a custom single-day range gives Today mode for that day — the scale rule follows the *shape* of the range, not the preset's label. An incomplete custom range (`resolveRange` returns `null`, `:375`) leaves both views showing their previous valid range rather than emptying them.

**This is still a user-visible behaviour change to a shipped surface** — the History View stops showing all history by default — taken deliberately rather than absorbed, and it invalidates `interruption-stack.md`'s "History View of **all** Time Blocks", which is amended in the same pass.

**3. A fixed 8-hour viewport in Today mode**, anchored 15 minutes before the day's first tracked block, or 15 minutes before the current time when nothing has been tracked. The pixels-per-minute scale is constant, which is what Goal 1 depends on.

**A day longer than 8 hours therefore does not fit, and the viewport scrolls.** *(Added after review — the first draft said only that the window "advances," which left a 07:00–19:00 day undefined and would have made Goal 4 false for any long day.)* The window auto-follows the present moment **only while the user has not scrolled it**; any manual scroll pins it until the mode is re-entered. Auto-following must never move the viewport during a drag.

**"Today mode" is really single-day mode, and the past-day case needs its own rules.** *(Added on the fourth review, which found decision 2 routing a custom single-day range into a mode defined entirely in terms of the present moment.)* For a day that is not today:

- the anchor rule is unchanged — 15 minutes before **that day's** first tracked block;
- there is **no present-moment marker and no auto-follow**, because neither exists in that range;
- a past day with **no tracked blocks at all** defaults to an **08:00–16:00** local window. Arbitrary, and labelled as such (`principles.md` #7) — there is no data to anchor to and no non-arbitrary answer. **Revisit** if the author's actual working window turns out to sit elsewhere.

The name is kept because today is overwhelmingly the common case and "single-day mode" would obscure that, but the mode is selected by the range's *shape*, not by whether it is today.

*(8 hours and 15 minutes are user-chosen and unvalidated. Revisit if a normal working day routinely exceeds the window, or if the 15-minute lead-in proves too small to start a drag above the first block.)*

**4. Range mode renders the entire selected range, rescaled to fit — and Goal 1 does not hold there.** *(Materially revised after review.)* Decision 3 rejects fit-to-contents rescaling as destroying the size signal. Range mode *is* fit-to-contents rescaling, and the first draft's defence — that comparison happens within a view — does not rescue it: across a week, short blocks collapse into clusters, so a 5-minute and a 50-minute block are not compared at all — one of them is not individually rendered.

So the honest statement, rather than a defence: **range mode is for navigation and overview, not for duration verification.** Goal 1 is a Today-mode property. This is an accepted limitation, and it is survivable because selecting any block still reveals exact times — but nobody should read a range-mode block's size as evidence about its duration. Tracked as risk **R15**.

**5. There is no minimum rendered extent. Every block renders at its true duration and its true position, always. Where adjacent blocks become too small to target reliably, the run is replaced by a cluster.**

*(Decided on the fourth review, which killed the floor outright. Two earlier drafts set one — 12 px, then 24 px — and both were wrong for the same unexamined reason.)*

**The arithmetic that ended it.** A floor plus a constant scale are jointly unsatisfiable for a *sequence* of short blocks, and every draft reasoned about one inflated block in isolation. Take this doc's own Goal 4 scenario — 12 blocks, six under 15 minutes — in a 500 px column: true extent 312 px, rendered extent 425 px. **113 px of debt, 22% of the column.** Twelve blocks all under five minutes: 62 px of time drawn as 288 px. The surplus has to be absorbed by overlapping blocks, displacing them, or stretching the axis — and all three were rejected:

- **Displacement** makes a block's *position* lie about its start, so free space stops mapping to free time. That is a larger instance of exactly the lie decision 2's context-block rule was added to prevent.
- **Overlap** makes pointer targets ambiguous in dense runs, destroying the argument the 24 px floor was raised to establish in the first place.
- **Local stretching** falsifies decision 3's constant scale, which Goal 1 depends on.

**So proportionality is absolute and interaction is what degrades.** A block's extent and position always tell the truth. What gives instead is *targetability*, and it gives visibly:

- A block's **hit target** is 24×24 and may overhang into adjacent **free** time, which covers the common case of one short block among long ones.
- **Where two adjacent blocks' targets would collide, the run renders as a cluster** — a single element occupying the run's true combined span, showing how many blocks it contains, opened by zoom or by the numeric detail surface.

**The 24×24 target floor is explicitly inapplicable to a block whose proportional geometry makes it impossible**, and that exception is recorded rather than assumed. `visual-redesign.md` D.2 sets 24×24 as "a floor, not a target" for "any element a pointer must hit" — written before any surface existed where an element's size is *dictated by data*. The cluster is what honours the rule: it is the element the pointer hits, and it is above the floor. **No sub-floor element is ever offered as a target.**

**This reverses a rejection made in this doc's own first alternatives round, and the reversal is the point.** "Stacked density indicator" was rejected then because hiding individual blocks means a hidden block with a wrong inferred end is worse than a small one. That objection was correct and still stands — it is simply now the *least bad* of four options rather than worse than a floor that turned out to be impossible. Two things blunt it: a cluster **states its member count**, so nothing is silently absent, and the alternative was a floor that lied about size on every one of those same blocks. Tracked as **R16**, rewritten.

**Consequence for the marker gutter:** there is no distortion mark, because nothing is distorted. The gutter carries three lanes, not four.

**6. A marker gutter, not marks inside blocks.** *(New after review, which found the first draft's claim that provenance marks are "inherited in the same form they take in the History View" to be impossible — that form is the italicised *word* `inferred`, and a 12 px block has no room for a word.)*

A **marker gutter** runs **inside** the 96 px column, along its trailing edge, aligned to each block, with one lane per channel. *(Inside, not alongside — the second review caught decision 1 sizing the column to include the gutter while this decision described it as outside. Inside is the one that makes A16's 96 px number mean anything.)* It carries `SystemInferred` ends, origin, adjusted-ness — the three `visual-redesign.md` requires on **two independent channels** without colour. *(**Three lanes, not four, corrected on the fifth review.** This sentence still added "and the distortion mark" two paragraphs after decision 5 abolished it along with the rendered floor — the same sentence decision 5's own closing line already said was wrong. `glossary.md` and **R15** carried the same ghost and were swept with it.)* Marks live in the gutter rather than inside blocks because block extent is exactly what cannot be relied upon: a block only a few pixels tall has room for neither a legend nor, often, a target of its own, and range mode makes that the common case. This is also what answers the "mark legibility at density" risk, which a mark drawn inside the block would have failed.

**Lane geometry, supplied by the design system 2026-08-08.** Three lanes of **7 px**, separated by **2 px** gaps — **25 px** of the 96 px column, leaving 71 px for the block body. Lane 1 carries the `SystemInferred` end as a 1.5 px vertical stroke, solid versus dashed; lane 2 carries origin as a 5 px dot, filled for live capture and outline for manual entry; lane 3 carries adjusted-ness as a 5 px tick, present only when true. Below 8 px of block height the dashed stroke stops reading as dashed and collapses to a single 3 px tick; below 5 px the dot and tick may overflow the block's own vertical extent **within the lane**, which is safe precisely because lanes are never pointer targets.

**What the lanes show against a cluster** *(decided 2026-08-08 — the case neither decision 5 nor this one covered, since the gutter is specified per block and a cluster is not one).* **Lanes 1 and 3 are presence flags, so they show `any`:** dashed if *any* member has a `SystemInferred` end, ticked if *any* member is adjusted. **Lane 2 is categorical, so it shows agreement or `mixed`:** filled if every member was live-captured, outline if every member was manual, and a distinct **mixed** glyph otherwise.

The alternative — showing nothing until the cluster is opened — was rejected, and on this feature's own terms. **R4** and **R9** are about spotting a wrong inferred end, and **R16** already concedes that clustering hides blocks; its mitigation is that a cluster declares itself. A cluster that reveals nothing forces the user to open every cluster to find one inferred end, which is the same cost as having no signal at all. `any`-of costs exactly one mark per lane — no more space than a single block — because two of the three lanes were already booleans.

**7. Restricted-ness is read from the projection, never recomputed.** `timeline-reconstruction.md`'s tier table makes the **currently active** block and any block with an **open interruption frame** identity-only. On a table that is invisible; on a drag surface it means some blocks do not respond, which is indistinguishable from a broken UI.

A block is restricted when `derived_interruption_status == Pending` or `id == active.id`, **both read from `StackView`**, which already serialises them (`commands.rs` — `ClosedBlockView.derived_interruption_status` and `StackView.active`) and which the History View already renders (`+page.svelte:756`). The variant itself is `model.rs`'s `DerivedInterruptionStatus`. *(The mechanism is named after review, which correctly pointed out that the first draft's prose denied duplicating a domain rule while its own trade-off table listed "the affordance drifts from the tier rule" as a risk. Reading the projection removes the risk rather than accepting it.)*

**8. Timeline gestures are `infrequent` under `visual-redesign.md` E.3's rule, and must therefore be findable rather than memorable.** *(Decided here after review — the first draft cited that doc as already classifying them. It does not: E.3 states the rule, and that doc's acceptance criterion lists both classes without reconstruction in either.)* Add, Move and Resize are correction operations on an exception path; they are not among the seven bound capture actions. Findable means a visible affordance at the point of use — not a tooltip, not a legend, and not a modifier the user must know about.

## Alternatives

### G. How one surface distinguishes Add, Move and Resize

1. **Modifier keys** — rejected: it makes three operations memorable rather than findable, against decision 8.
2. **An explicit mode toggle** — unambiguous, and rejected anyway: it puts a mode between the user and every correction, and a wrong mode is silently wrong.
3. **Zone-based, with selection promoting handles and numeric fields.** **Chosen.** Free space initiates **Add**; a block's body initiates **Move**; a selected block's leading/trailing edge handle initiates **Resize**.

   **The geometry problem, and why selection alone does not solve it.** Three stacked zones — two handles plus a body — need 72 px, which most blocks on a fragmented day do not have. Promoting handles on selection helps, but the first draft's resolution (handles extend into adjacent empty space) fails in the case that actually matters: **abutting blocks leave no empty space, and abutting is the normal shape of live-captured work**, not an edge case — the state machine closes each block at the instant the next starts (`timeline-reconstruction.md:176`). Review was right that this made Resize unavailable for precisely the short, fragmented blocks R4 and R9 exist to fix.

   **So selection opens a detail panel carrying editable numeric fields**, and that panel is the fallback. Its rules, all of which the second review found undefined:

   - **Two fields and one action, mapping 1:1 onto the two commands.** Editing **start** or **end** individually is a **Resize**. A separate explicit *"Move to…"* action sets a new start and preserves duration — a **Move**. **Intent is never inferred from the delta.** `timeline-reconstruction.md` alternative E keeps Move and Resize separate precisely because they answer different questions about how the state came to be; guessing which one a numeric edit meant would reintroduce the "operation misrepresents itself" failure that argument rejects.
   - **Restricted blocks get read-only fields.** A block that cannot be reshaped spatially cannot be reshaped numerically either. Otherwise Alternative J's whole rationale — the restriction is known *before* effort is spent — is defeated by the fallback, and the user fills in fields whose commit the domain will reject with `BlockIsActive` or `BlockReferencedByOpenFrame`. The fields still *display* exact times, which is decision 5's detail-on-selection and is unaffected by restriction.
   - **Numeric entry clamps exactly as a drag does**, and shows it. "The editor clamps; the domain rejects" is an invariant about the editor, not about drags — leaving it undefined on the surface that is now the universal fallback would leave half the accepted invariant unimplemented.
   - **Granularity is one minute**, for the fields and for deciding whether a gesture is a no-op. Stated because "ends where it began" is otherwise undecidable: pixel-equality and time-equality differ, and in range mode a pixel can be many minutes. *(Re-worded on the fifth review, which found this justified by "at the floor" — a floor decision 5 removed. The point itself never depended on it, only on a pixel being coarser than a minute, which range mode guarantees and Today mode's 1 px/minute makes exactly borderline.)*
   - **The boundary that was not edited is echoed back verbatim, sub-minute component included.** *(Added on the third review.)* This is not a detail: `Resize { target, start, end }` has no start-optional form (`model.rs`), so a pure end correction must re-send a start. Live-captured boundaries carry seconds, and re-sending a minute-rounded start would silently move a block by up to 59 seconds in a field the user never touched — and against an abutting predecessor, would turn a legal edit into a clamp or a rejection for a reason that is invisible. The panel therefore *displays* minutes and *transmits* the stored value for any boundary the user did not change.

   **Why these fields belong here as well as on the History View**, whose selected-row edit form (`+page.svelte:777-824`) now carries the same two boundaries: because they are the same selection this surface already maintains, showing the same block, in the same act of inspecting it — not a detour to another view. *(**Rewritten 2026-08-06.** This read "here and **not** on the History View" and called them "**not** a second editing surface." Both claims died when Resize shipped on that row to close **R9**: there are now two numeric routes to the same command, and defending the old wording would be precisely the unverified-claim failure this document's own table records three times. The argument that survives is the one that was always doing the work — the History View cannot show you the neighbour you are clamping against, which is what makes a timeline the right place to **judge** a boundary. Correcting a boundary you have already judged is fine anywhere, and R9 asked only for that.)* *(Raised by the second review as a genuine threat to `mvp.md`'s justification for the Editor being a hard prerequisite — "move and resize are meaningless against the tabular History View." That claim needed narrowing rather than defending: they are not *meaningless* there, they are **unjudgeable** there. `mvp.md` is amended to say so.)*

   **Zone precedence**, since a promoted handle overhangs a short block:

   1. A selected block's handles take precedence over **Add** where they overhang free space.
   2. They take precedence over an **abutting neighbour's body** (its Move zone) where they overhang it — selection is explicit, so the selected block wins.
   3. **Where two handles would collide** — a block whose two 24 px handles need 48 px it does not have — **no handles are offered at all**, and the numeric panel is the only path. It is geometrically impossible to satisfy the 24×24 rule with two handles on a short block, and pretending otherwise would ship an acceptance criterion that cannot pass. A block small enough to sit inside a **cluster** is reached by opening the cluster first.

### H. How a clamp is communicated

`timeline-reconstruction.md:135` requires the clamp be visible — *"a silent clamp reads as a broken UI."* That is the requirement; the form is this doc's.

1. **A message or toast** — rejected twice: it is explanatory chrome `users.md` rules out, and it arrives mid-gesture where it cannot be read.
2. **Rubber-band resistance** — rejected: it shows a state that cannot be committed, so on release the result differs from what was on screen a moment earlier.
3. **Pointer decoupling plus boundary emphasis.** **Chosen.** The block stops dead at the boundary while the pointer continues past, and the shared boundary is emphasised for as long as the gesture pushes against it. The growing gap between pointer and block edge *is* the signal — it turns "nothing is happening" into "something is stopping this."

   The emphasis may not be hue-only, on the same usability grounds as the gutter's marks (decision 6) and for the same reason it is not an F.3 matter. *(Re-pointed on the fifth review, which found this citing "decision 5's mark" — decision 5 no longer has one.)*

### I. Whether the active block renders live

1. **Render it ending at its last durable write** — rejected: it would show the active block shrinking away from the present moment, and `timeline-reconstruction.md:201` requires it be rendered *"as occupied up to the present moment"*, since it is a collision boundary whose end is `now`.
2. **Render it live, growing to the present moment.** **Chosen.** Consistent with `export.md:87,109`, which already treats elapsed-so-far as a live, never-persisted computation.

   **Redraw cadence: once per second.** *(Both the bound and its justification were wrong in the first draft, which said "at most once per second" in one place and "at least once per second" in another, and justified it by a drag approaching the growing boundary — an interaction that cannot happen, since no block may end in the future and a reconstructed block lies entirely before the active block's start (`timeline-reconstruction.md:181, 203`).)* The real justification is simpler: the present moment is the one thing on this surface that must not look stale. **Unvalidated; revisit if the redraw is perceptible as jitter, or if a slower cadence proves indistinguishable.**

   **It is live but not editable** — the tier table makes it identity-only, and its rendered end is a projection to `now`, not a recorded value, so there is nothing there to drag.

### J. How a block that cannot be reshaped says so

**Not previously identified as a fork; found during this design pass.**

1. **Let the drag fail and surface the domain's error** — rejected: it reports the restriction only after effort has been spent on a gesture that was never going to work.
2. **Hide or grey out restricted blocks** — rejected outright: these are real blocks with real durations, and the active one is the most important thing on the surface. Suppressing them to express a permission breaks Goal 1.
3. **Rendered in full, but presenting no drag affordance.** **Chosen.** The block reads normally — same extent, same position, same gutter marks — but no handles appear on selection and the body offers no move affordance. The restriction is communicated *before* the gesture.

   **This does not duplicate a domain rule**, and decision 7 is what makes that true rather than merely asserted: restricted-ness is *read from the projection the domain already publishes*, not recomputed. The domain still rejects the operation if it arrives by any route, and the editor still surfaces that rejection. Same relationship as "the editor clamps; the domain rejects."

### K. Whether a recovery hole is distinguishable from an untracked gap

**Raised because [ADR 0007](../../decisions/0007-auto-resume-after-a-short-gap.md):86 makes a claim about this surface** — *"The timeline shows an honest hole where Anchor was not running."* On this surface a hole is free space, and free space is the Add zone, so an outage and an untracked gap are geometrically identical.

1. **Mark recovery holes distinctly** — attractive, and rejected: the projection carries no representation of an outage. `RecoverGap` is a transition, not a projected entity, so surfacing one would require a domain change this doc's no-behavioural-change constraint forbids.
2. **Do not distinguish them.** **Chosen.** ADR 0007's requirement is satisfied by the *absence of a block*, which is what "honest hole" means — Anchor is not claiming time it did not capture. Both kinds of hole are equally correctable by the same Add gesture, which is the right outcome either way, and **nothing prompts** in either case ([ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md)).

### L. What the range control governs

**Added on the second review**, which found decision 2 presenting a binary — one range across everything, or two adjacent views deliberately disagreeing — while never naming the option between them. Under `design.md`'s funnel that was a missing stage, and on the lowest-reversibility decision in the doc.

1. **One range across timeline, History View and export.** One mental model, and rejected on a specific failure: it makes a **navigation action silently change what gets billed.** Narrowing the timeline to inspect an hour would narrow the next export to that hour, and `export.md` is the billing artifact — this is `docs/risks.md` **R2**'s shape (a silently wrong total) arriving through a UI convenience. Persisting the range would make it survive restarts, so the app could reopen pre-narrowed.
2. **No shared range; the timeline gets its own and the History View stays unfiltered** — no behaviour change to anything shipped, and rejected: two views of one dataset, side by side, showing different subsets is a defect wherever else it appears, and this doc's own timeline/History-View agreement criterion forbids it.
3. **A shared *view* range over the timeline and the History View; export keeps its own.** **Chosen.** The two adjacent views always agree, which is the whole point of the coupling, and the surface where being wrong costs money keeps the explicit, deliberate range selection it has today. Export is not a view — it is an act with an outcome, and it should not inherit scope from where the user happened to have scrolled.

   The cost is honest: **two range controls exist**, and a user who narrows the view and then exports gets everything, not what they were looking at. That is the safe direction of the two — an export that is too broad is visible and correctable, an export that is silently too narrow is neither.

## Trade-offs

| | Layout & orientation | Range | Viewport | Minimum extent |
|---|---|---|---|---|
| **Chosen** | Side by side, vertical, 96 px wide, 480 px min height, not user-selectable | Shared **view** range over timeline + History View; export keeps its own | 8h scrollable in Today; fit-to-range otherwise | No floor — true extent always; dense runs cluster |
| Complexity | Low | **Moderate — a behaviour change to a shipped surface, plus a canvas/set distinction** | Moderate — two scale rules and a scroll/pin rule | **Moderate-high — clustering, and a cluster-open interaction** |
| Reversibility | Moderate | **Moderate** — it is a two-view display preference, so reverting is a filter removal; export was never coupled and is unaffected | Moderate | Moderate — clustering is a rendering rule, but the interaction it implies is not trivially removed |
| UX impact | Timeline always visible; table narrower | The two adjacent views always agree; export stays deliberate | Today stays focused; history stays navigable | Nothing ever lies about size or position; dense runs need one extra step |
| Risk if wrong | 800 px proves too tight and both views are cramped | History View filtering hides work the user expected to see; two range controls read as duplication | A pinned viewport hides the present moment without saying so | A wrong inferred end hides inside a cluster and is never spotted (**R16**) |

**Deliberately *not* chosen, and worth stating because it is the intuitive option:** one range governing export too. It would make a navigation action silently change what gets billed — see Alternative L.

| | Gestures | Clamp feedback | Active block | Restricted blocks | Recovery holes |
|---|---|---|---|---|---|
| **Chosen** | Zone-based; selection promotes handles **and numeric fields** | Pointer decoupling + boundary emphasis | Live at 1s; not editable | Rendered fully, no affordance, read from projection | Not distinguished |
| Complexity | Moderate | Low | Low | Low | None |
| Reversibility | High | High | High | High | High — marking them later is additive |
| UX impact | Three operations, no modes, and a size-independent fallback | A clamp reads as a limit, not a fault | The present moment is always visible | A restriction is known before effort is spent | An outage is corrected the same way as any gap |
| Risk if wrong | The numeric panel becomes the real editor and the spatial surface is decoration | Emphasis too subtle at range-mode density | 1s proves perceptible as jitter | `StackView` gains a third restricted class and the editor is not updated | A user re-adds work Anchor already recovered, double-counting it |

**The two risks this design knowingly takes on**, both now in `docs/risks.md`:

- **R15 — range mode cannot verify durations.** Not a defect to be fixed but a stated limitation of decision 4.
- **R16 — clustering hides individual blocks**, and a hidden block with a wrong inferred end is worse than a small one. Adopted knowingly as the least bad of four options once the floor proved impossible.

**Not a trade-off, recorded so it is not mistaken for one:** none of these decisions touches the transition log, the domain's validation, or export output. Every one is presentation. If any appears to require a new transition or persisted field, that is a signal to stop and raise it.

## UX

Owned by the ux-designer. Operation semantics are `timeline-reconstruction.md`'s and are not restated.

### Layout, scale and states

- A **96 px vertical column** beside the History View, both permanently visible, time running top to bottom.
- **Today mode** — a fixed 8-hour window, anchored 15 minutes before the day's first tracked block or before the current time. Scrolls; auto-follows the present moment only until the user scrolls, then pins. Never auto-scrolls during a drag.
- **Range mode** — the selected range rendered in full, rescaled to fit, no scrolling needed to reach either end.
- **The current mode is identifiable from the surface itself**, since the two modes have different scales and nothing else distinguishes them.
- **Empty (nothing tracked)** — the window renders with its time axis and a present-moment marker, not a blank panel. No empty-state tutorial and no call to action: the axis *is* the affordance, because drawing on free space is how Add works.
- **Loading** — none. The timeline reads the same projection the History View already renders.
- **Rejected operation** — the domain's own error is surfaced, untranslated and unsoftened.

### Blocks and the marker gutter

- **Every block renders at its true duration and true position. There is no exception.** Where adjacent blocks are too small to target separately, the run renders as a **cluster** occupying the run's true combined span and stating how many blocks it holds.
- The **marker gutter** runs inside the column along its trailing edge, aligned per block, one lane per channel: `SystemInferred` end, origin, adjusted-ness. All three remain distinguishable with colour removed, in both themes. **Gutter lanes are never pointer targets** — they report, they do not receive input — so the 24×24 rule does not reach them, and three reporting lanes fit the 96 px column alongside the block.
- Selecting a block reveals exact start, end and duration. On a **reshapeable** block these are editable fields plus a *Move to…* action, and resize handles are promoted where geometry allows. On a **restricted** block they are read-only — the restriction is stated before effort is spent, not after.
- **Interruption nesting is not represented.** Every block is a peer in start order, matching the accepted flat model (`docs/product/mvp.md`). Nesting is read from the History View's `DerivedInterruptionStatus`.

### Gestures

| Where the drag starts | Operation |
|---|---|
| Free space | **Add** — the start point must fall in free space; the end clamps at the next occupied boundary |
| A block's body | **Move** — both boundaries translate, duration preserved |
| A selected block's edge handle | **Resize** — one boundary moves; takes precedence over Add where it overhangs |

- **Add opens naming** with the same autocomplete `Rename` uses — Task Templates plus past task history, source-tagged (`timeline-reconstruction.md:132`). The new block is `CaptureOrigin::ManualEntry`.
- **A clamp decouples the block from the pointer** and emphasises the blocking boundary while the gesture pushes against it.
- **A gesture that ends where it began is a cancelled gesture**, and the editor invokes no command — the same as any other cancel. *(Reframed after review. The first draft made this a rule about no-op transitions, which would have put a record-affecting decision in the UI: `timeline-reconstruction.md:182,193` makes the adjusted flag monotonic with no no-op exception, so a no-op Move submitted by any other caller **does** mark the block adjusted. Not sending a command for a gesture the user did not complete is an editor concern; what happens to a command that **is** sent remains entirely the domain's, unchanged and unduplicated.)*
- **Restricted blocks present no drag affordance** — the active block, and any block whose interruption frame is still open.
- **Nothing prompts.** Anchor never suggests a gap looks like untracked work.
- **Edit Identity and Delete are not on this surface** — they are History View row actions and already ship there. **Resize ships there too** (2026-08-06, closing **R9**) and is still offered here, because only here can a boundary be judged against its neighbour.

### Motion

Inherits `visual-redesign.md`'s 150–200ms ease-out for state changes. **Drag feedback is excluded from it** — that doc anticipated this, noting the drag case "gets its own value rather than this one being stretched." Dragging tracks the pointer with no easing; interpolating a direct-manipulation gesture makes the surface feel detached from the input.

## Technical Constraints

**Inherited, not open:**

- **The domain is fixed and shipping**, and **rejects** overlaps rather than clamping. Supplying the clamping half is this feature's job.
- **`Move` carries only `start`** — duration is preserved by construction, so a move gesture cannot change a duration even if the UI tried.
- **Spans are half-open**, so abutting blocks are legal and a drag to an exact boundary is not a collision.
- **Blocks are addressed by stable id** ([ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md)) and survive replay and compaction. This is what makes a spatial surface safe to build at all.
- **The tier rule** — active and open-frame blocks are identity-only.
- **No behavioural change to the record.** No transition type, persisted field, or export output changes.

**This surface's own:**

- **Restricted-ness is read from `StackView`** — `derived_interruption_status == Pending || id == active.id` — never recomputed. If the domain ever adds a third restricted class, this must read it rather than mirror it.
- **The active block's collision boundary is re-evaluated by the domain at commit time.** The editor renders it growing to `now`; the domain validates against the transition's own timestamp. A `Utc::now()` inside the state machine would make replay depend on when it runs, so the editor must not push it toward one by sending a stale boundary.
- **Two view modes mean two scale computations, not two renderers.** Pixels-per-minute is one input to one rendering path; two paths would drift and cluster inconsistently, so the same day would show different block counts in the two modes.
- **Clustering is computed from the rendered scale, not stored.** Which blocks fall into a cluster changes with zoom and mode; nothing about it is persisted, and it has no export consequence.
- **The view range persists in its own store**, following the one-file-per-concern pattern `paths.rs` already establishes (`settings.json`, `export_settings.json`, `templates.json`) and the same reasoning `visual-redesign.md` used for `theme.json`. The store persists selection intent — `today`, `this-week`, or `custom` with inclusive calendar dates — never timezone-resolved instants. It is **backend-owned**, because both views must agree and a Rust-owned value gives that for free. It is new state shared by exactly two surfaces — the timeline and the History View — and **never by export**, which keeps the range control it has today (Alternative L). It **does** persist across restarts, following the existing per-concern settings pattern; that is decided here, not left to implementation, because a persisted range changes what the app shows on launch.
- **The canvas is not the set.** Rendering asks whether a block's span intersects the viewport; membership asks whether its `start` falls in the range. An implementation that uses one query for both will either hide occupied time or filter the History View wrongly.
- **`app/src/routes/+page.svelte` is classified binary by git** — two NUL bytes in an `R.uniqBy` key (`:460`) — and is the file this feature extends, so it gets no textual diffs or line-level merges. `visual-redesign.md` notes the rewrite may resolve this with a non-NUL delimiter; if that has not happened by the time this is built, it should happen here.
- **Implementation is blocked on `visual-redesign.md`'s inputs** — the spacing scale's steps, the hue palette and its size, the font weights — which are not in this repository. This document may be accepted against the accepted contract; building it may not begin before those arrive.

## Acceptance Criteria

**Viewport and range**

- In Today mode the viewport spans exactly **8 hours**, anchored **15 minutes** before the day's first tracked block, or 15 minutes before the current time when no block exists.
- A day spanning more than 8 hours is fully reachable by scrolling; the viewport auto-follows the present moment until the user scrolls, then stays pinned, and never moves during a drag.
- In range mode the entire selected range renders with no scrolling required to reach either end.
- The current view mode is identifiable from the surface without changing the range control.
- For any view range, the timeline and the History View contain **the same set of in-range blocks**, with membership decided by each block's `start` — the same rule `export.md:95` already applies.
- A block that started before the range but occupies time inside the viewport renders on the timeline as **context** — visible, non-editable, and **not offered as free space** — while correctly appearing in neither the History View nor an export of that range. Drawing an Add across an interval occupied by a context block is impossible from the UI, so the domain's overlap rejection is never reached by a correct gesture.
- Changing the view range **does not change what a subsequent export contains**. Export's own range control is unaffected, and narrowing the view then exporting yields the export range's blocks, not the view's.
- The view range survives an app restart; the export range behaves exactly as it does today.
- A range resolving to exactly one day renders in Today mode; any other range renders in range mode. An incomplete custom range leaves both views on their previous valid range rather than emptying them.

**Rendering**

- Within a single view, two blocks of equal duration render at equal extent, and one of twice the duration at twice the extent — **with no exception**. Proportionality is never traded away.
- In Today mode, two blocks whose durations differ by **15 minutes or more** render at visibly different extents whenever both are rendered individually rather than inside a cluster. *(Goal 1's actual test. It is now unconditional on size, because nothing is inflated — the only way a block escapes it is by being clustered, and a cluster declares itself.)*
- No block is ever rendered at an extent or position other than its true one, in either mode.
- A cluster occupies exactly its members' combined span, states its member count, and reveals every member individually when opened.
- All three provenance channels, the cluster indication, and the clamp emphasis remain distinguishable **with colour removed**, in both themes.
- Every element a pointer must hit is at least **24×24 CSS pixels** — satisfied by a block's hit target overhanging into adjacent free time, or, where neighbours are too close for that, by the run becoming a cluster whose own target is above the floor. **No sub-floor element is ever offered as a target**, so the rule is never violated: it is met by withholding or by aggregating, never by inflating.
- A short block rendered individually is still **movable** by body drag and **selectable**; only resizing falls back to the numeric panel. A block inside a cluster is reached by opening the cluster. The spatial surface does not become decoration for short blocks.
- The main window cannot be resized below **800 px** wide, and at exactly 800 px both views render without clipping.
- **At the window's minimum height the timeline column renders at least 480 CSS px**, which is the 1 px per minute that decision 1 quotes every duration statement in this document at. *(Added on the fifth review. Decision 1 said the window "gains a `minHeight` to guarantee it" and no criterion tested it, so the number the entire scale rests on was unfalsifiable — the same shape of gap that review found in decision 1 itself, one level down. Stated as a property of the column rather than a window pixel count, because the chrome around it is not fixed by this document and a hard-coded total would go stale the first time it changed.)*

**Gestures**

- Dragging a block's body produces a `Move`, a handle a `Resize`, and free space an `Add` with `CaptureOrigin::ManualEntry`. *(The Move-preserves-duration case is a regression guard, not a live risk: `TransitionPayload::Move { target, start }` carries no end, so duration preservation is structural.)*
- Add opens the naming step with Rename's autocomplete before any transition is written.
- A drag reaching a neighbouring boundary stops there, the pointer visibly separates from the block edge, the boundary is emphasised while the gesture continues, and releasing commits exactly what was on screen at release.
- A gesture ending where it began invokes **no command at all** — and a no-op `Move` submitted directly to the domain still marks the block adjusted, proving the editor did not take over a domain rule.
- Every **reshapeable** block, including one reached by opening a cluster, is editable via the selection panel's numeric fields. A **restricted** block's fields are read-only and still display its exact times.
- Editing `start` or `end` in the panel produces a `Resize`; the explicit *Move to…* action produces a `Move` with duration preserved. No numeric edit is ever resolved into a command by inferring intent from the size of the change.
- A numeric entry that would overlap a neighbour clamps to that neighbour's boundary and shows that it clamped, exactly as a drag does.
- Times are **entered and displayed** at one-minute granularity, and a gesture is judged a no-op at that same granularity. A boundary the user did not edit is transmitted at its stored precision, seconds included — never re-rounded.
- A selected block's handles take precedence over Add where they overhang free space, and over an abutting neighbour's body where they overhang that.
- The active block renders occupied to the present moment, updates once per second, and presents no handles and no move affordance.
- A block with an open interruption frame presents no drag affordance, while Edit Identity on that same block still succeeds from the History View.
- An operation rejected by the domain surfaces the domain's own error, unrewritten.
- Every successful gesture appends **exactly one** transition; every rejected or cancelled one appends **none**.

**Goals not otherwise covered**

- **Goal 3 is tested by two metrics with directional expectations, not one.** *(Rewritten twice. The original named **Capture Rate**, which adjusted blocks still count toward, so Move and Resize could not move it — a user could correct fifty blocks a day and pass. The replacement named **Adjustment Rate** alone, which fails in the opposite direction: correcting a wrongly-inferred end *is* a Resize of a live-captured block, so every **R9** correction — the headline reason this feature exists — raises it. A criterion that fails when the feature works is not a test of the goal.)*

  Over a rolling working week:
  - **Capture Rate stays at or above its provisional ≥90% target** (`glossary.md`). *(Corrected again on the fourth review. The previous wording — "must not fall" — was unusable for two reasons: `Add` is the only producer of `ManualEntry` and is gated on this very surface, so the pre-ship baseline is **100% by construction** and any legitimate use fails the criterion; and it contradicts `timeline-reconstruction.md`, which states that Capture Rate **falling** when a day is reconstructed is "the metric reflecting reconstruction rather than hiding it." A fall is the metric working. A fall **through the target** is the goal failing, and only a threshold can tell those apart.)*
  - **Adjustment Rate may rise, and a rise is not a failure.** It measures capture *accuracy*, and this surface exists to let the user fix inaccurate captures — so every **R9** correction raises it by design.
  - **The failing signal is Capture Rate below target**, with or without a rising Adjustment Rate: work being reconstructed rather than captured. `glossary.md`'s own reading of the pair says exactly this — "low capture / low adjustment means work is being reconstructed rather than captured."
- For a day of at least 12 blocks, half of them under 15 minutes, Today mode renders every block within the 8-hour window with no scrolling, and **every block is reachable** — directly if rendered individually, or by opening the cluster that holds it (Goal 4). *(Restated twice: from "is legible", the unmeasurable quantifier that got Goal 1's "configured threshold" rejected on pass one; and then from "each is individually selectable", which the no-floor decision made false — clustered blocks are reachable, not individually selectable.)* Passing this does **not** mean every duration is readable at a glance: a clustered block's size is not shown until the cluster is opened, which is risk **R16** and is why the criterion tests reachability rather than proportionality.

**Non-regression**

- No transition type, stored Time Block, or export output differs before and after this feature ships, proven by an unchanged export from identical input.
- A full Switch/Interrupt/Return cycle on the hotkey and widget paths is unchanged — no markup, handler, or command on either is touched.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
