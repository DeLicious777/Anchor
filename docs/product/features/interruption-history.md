---
status: accepted
date: 2026-08-07
owner: erich
related: [docs/vision/vision.md, docs/concept/concept.md, docs/product/users.md, docs/product/mvp.md, docs/principles.md, docs/risks.md, docs/assumptions.md, docs/glossary.md, docs/product/features/interruption-stack.md, docs/product/features/visual-redesign.md, docs/product/features/timeline-editor.md, docs/product/features/timeline-reconstruction.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0006-stable-persistent-time-block-identity.md]
---

# Interruption History

> Created 2026-08-07. Follows `.claude/workflows/design.md`.
>
> **This is the surface `docs/product/mvp.md:25` calls the "interruption-history panel"** — progressive disclosure of the interruption stack, *"which is also where a frame can be explicitly dismissed."* Both halves are in scope here; the name is kept because two accepted docs use it, and is given a precise meaning in `docs/glossary.md` rather than being redefined by implication.
>
> **A citation convention, adopted here and worth generalising.** This document cites Rust items by **symbol name** (`InterruptionStack::apply`'s `ReturnOriginal` arm) rather than by line, except where a line is the only way to point at something. Risk **R11**'s line-number decay was measured directly on 2026-08-06: the Timeline Editor's `+page.svelte` citations went stale twice within two days, the second time from an unrelated three-line edit. Symbol names do not decay when code above them moves.

## Problem

The interruption stack is Anchor's distinguishing mechanic, and it is the one piece of state the product currently asks the user to hold in their head. Four concrete gaps, each verified against the code rather than assumed:

1. **The two return actions do not say what they will do.** `+page.svelte:705-706` renders *Return to Previous* and *Return to Original* as bare buttons, disabled only on an empty stack. Their consequences differ sharply and are invisible: `InterruptionStack::apply`'s `ReturnPrevious` arm pops **one** frame and resolves it `Resumed`; the `ReturnOriginal` arm pops **every** frame, resolving each intermediate one `Skipped` and only the root `Resumed`. At depth 4, one button silently marks three pieces of work as never-resumed. Nothing on screen says which task either button lands on, or how many blocks the second one writes off.

2. **A shipped error message instructs the user to perform an action that does not exist.** `StackError::BlockReferencedByOpenFrame` reads: *"cannot be reshaped or deleted while an unresolved interruption still refers to it — resume or **dismiss** that interruption first."* There was no dismiss when this was written: `TransitionPayload` had fourteen variants, none of which dismissed a frame, and no command exposed one. *(**Updated 2026-08-07 (#24):** `TransitionPayload::DismissFrame` and the `dismiss_frame` command now exist, as an M3a enabling slice. **This problem is not solved.** The message is still false, because nothing calls that command — the only way a user can clear a frame is still to return to it. It closes when the panel designed below ships and supplies the caller.)* This is a textbook **R11** instance living in shipped code rather than in a document, and it is reachable from the Timeline Editor's restricted-block tier and from Delete.

3. **`Pending` is a dead end for the user, and it blocks reconstruction.** `timeline-editor.md` decision 7 makes any block with `derived_interruption_status == Pending` a **restricted block** — no drag affordance, non-editable times — and the History View now does the same. That restriction is correct, but its remedy ("resolve that interruption") has no surface. A block can therefore be uncorrectable with no route to make it correctable, which weakens **R9**'s closure at exactly the point where a day was messy enough to need it.

4. **Progressive disclosure is documented but inverted in the code.** `mvp.md:25` and `interruption-stack.md:36` both say the default UI shows only the current task and the two return options, with the full stack behind an optional panel. `+page.svelte:711-722` renders the entire stack **unconditionally**, as a flat `<ol>` of names, always visible. So this feature is not additive: it **replaces a shipped surface**, and that surface currently shows more by default than the accepted design permits, while showing less that is useful — no timings, no outcome, no way to act.

Evidence is direct inspection of `model.rs`, `stack.rs`, `commands.rs`, `+page.svelte`, and the accepted feature docs. The sole user is the author; no user research is involved or needed.

### The constraint that shapes everything below

**The pairing between an interruption and its resumption is not recorded anywhere that survives compaction.** This was checked before any layout was considered, because it decides what the panel can honestly be.

- `StackFrame` (`model.rs`) carries `paused_time_block_id`, `name`, `project`, `client`. It names the task that was **paused**, never the task that interrupted it, and carries no timestamp.
- A frame is **destroyed** when it is popped. Nothing is written in its place except the paused block's `interruption_outcome`.
- A resumption creates a **new** Time Block with a new id ([ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md)); nothing links it to the block it resumes.
- `Snapshot` persists `InterruptionStack` — `active`, `stack`, `closed`, `issued_anchor_names` — and compaction then truncates the log (`log::snapshot::compact`). The `Interrupt` and `Return*` lines that *do* encode the pairing are gone.

Therefore **a nesting tree of past interruptions cannot be reconstructed after compaction**, and building one from chronological adjacency plus name matching would be inference — barred by [ADR 0001](../../decisions/0001-manual-assisted-tracking-for-mvp.md), and fragile in exactly the way risk **R2** describes, since it would key on name equality.

What *is* exact, persisted, and compaction-proof:

| Fact | Where it comes from |
|---|---|
| The **current** stack, in order, with each frame's paused block | `StackView.stack` |
| **When** each frame was paused, and for how long | its paused block's `end` — the interrupt fixed it |
| Whether a past block was interrupted, and what became of it | `derived_interruption_status` — the canonical projection (ADR 0005) |
| Chronological order and every duration | `start` / `end` |

The panel is designed on that list and nothing else.

## Goals

Tied to `vision.md`'s criterion that the author can run a real workday through Anchor and **trust the result** — this feature serves the *nothing was lost* half.

- **Before pressing either return button, the user can see which task it resumes and what it writes off.** Measurable: at any stack depth ≥ 2, the panel names the block *Return to Previous* resumes, names the block *Return to Original* resumes, and states the count of blocks the latter will mark `Skipped`.
- **Every open frame has a resolution route from this surface** — resumed by the existing actions, or dismissed. This is what makes `BlockReferencedByOpenFrame`'s advice true, and what gives a restricted block a way to stop being restricted.
- **The default view does not show the stack**, matching `mvp.md:25`, while depth stays visible at a glance.
- **Interrupted work is accounted for**: for a given day the user can see which blocks were interrupted and whether each was resumed or skipped, without reading the full History View row by row.
- **The fast path is untouched.** No markup, handler, or command on the hotkey or widget path changes.

Explicit non-goals: **no third return path** (`mvp.md:25`, `interruption-stack.md:36`), no nesting tree, no new persisted field, no change to export output, and no inference about work Anchor did not observe.

## Users

The single segment in `docs/product/users.md`: **the interrupted billable developer**, currently the author alone.

- *"Values speed and low friction above all."* This surface sits on the **deliberate** side — the dashboard is "not meant for rapid interaction" (`interruption-stack.md`). It is judged on whether resolving a tangle is quick *once started*, never on competing with the hotkeys.
- *"Comfortable with hotkeys and a command palette,"* explicitly not needing onboarding. No tutorial, no first-run tour. The panel explains the *data*, never the product.

No new user segment is implied.

## Decisions taken (2026-08-07)

**1. The panel is a disclosure attached to the active-task region, not a third column, a modal, or a tab.**

`visual-redesign.md` **B.3** gives the Timeline the dashboard's main area and the active task **persistent placement**. *(Cited as "C.3" in the first draft, which is the theme mechanism — caught by this document's own review sweep.)* The stack is a property of the active task's return path, so it belongs with the active task; anywhere else and two regions describe the same subject.

The arithmetic rules out a third column outright. The window minimum is **800 px** (`timeline-editor.md` decision 1), of which the Timeline Editor takes a fixed **96 px**, leaving ~704 px for a ten-column History View that already needs `overflow-x: auto`. A third permanent column of any useful width comes directly out of that table. A disclosure costs **zero** horizontal pixels when closed, which is its normal state.

**2. The panel has two parts, and the second is deliberately flat.**

- **Now — the open stack.** Exact, live, ordered, with each frame's paused task, when it was paused, and how long it has been waiting.
- **Earlier — interrupted work resolved today.** A **chronological, non-nested** list of blocks whose `derived_interruption_status` is `Resumed` or `Skipped`, in the panel's current day.

**"Earlier" is flat because a tree would be a lie** — see the constraint above. This is stated in the panel's own wording, not just here, because a list of interruptions is exactly the shape a reader assumes is a flattened tree. Tracked as **R18**.

**3. The always-visible stack list is replaced by a depth indicator plus disclosure.** `+page.svelte:711-722` goes away. Depth remains permanently visible as a number — the widget already treats stack depth as one of its four glanceable facts (`visual-redesign.md`) — and the list moves behind the disclosure. This is a **user-visible behaviour change to a shipped surface**, taken deliberately: the accepted design has said since 2026-07-28 that the default shows only the current task and the two return options, and the code has never matched it.

**4. The panel states the consequence of both return actions, and adds no way to invoke a return that does not already exist.**

For a stack of depth *d*, derived entirely from `StackView`:

- *Return to Previous* resumes the **top** frame's task. Blocks written off: **none**.
- *Return to Original* resumes the **root** frame's task, and marks the other **d − 1** frames' blocks `Skipped`.

At *d* = 1 the two actions are identical and the panel says so rather than showing two identical previews.

**No frame is directly actionable as a return target.** `mvp.md:25` forbids a third return path in terms, and a clickable frame at arbitrary depth is precisely that. The preview is not a path — it changes nothing about which commands exist.

**5. Frame dismissal lives here, writes `Skipped`, and is confirmed.**

[ADR 0005](../../decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md) already decided the semantics and this document does not reopen them: *"Explicitly dismissing a frame writes the existing `Skipped`, because the domain fact is identical — this work was interrupted and never resumed."* There is no `Abandoned` value.

- **Any frame may be dismissed, not only the top one.** Dismissal is not a return: it resolves a frame without changing the active task, so the ordering of the remaining frames is untouched and no "third return path" is created.
- **Dismissing the root frame re-points *Return to Original*, and the panel must say so.** *(Found on this document's own review, which had left it undefined.)* "Original" is not a stored identity — `apply`'s `ReturnOriginal` arm resumes whatever sits at the bottom of the stack when it runs. Dismiss the root and the next-deepest frame silently inherits that role. The domain is right to work this way and is not changed here; what is unacceptable is the *silence*, so the confirmation for a root frame states which task becomes the new Return-to-Original target. At depth 1 that target becomes none, which is the same statement as "the stack becomes empty."
- **It is confirmed**, on the same grounds as Delete (`timeline-reconstruction.md`): it permanently marks real work as never-resumed, MVP has no undo, and the `Skipped` value is what `DerivedInterruptionStatus` publishes to every consumer including export's grouping. The persona rule reserves confirmations for exactly this.
- **Dismissing the last frame empties the stack, and whether that makes `Complete` available depends on what is running.** `apply`'s `Complete` arm checks two things in order: it rejects a non-empty stack, then requires an active task. So emptying the stack is *necessary* and not *sufficient* — with a task active, `Complete` becomes available; while **paused** (`active == None`) it stays unavailable, now for the second reason rather than the first, until work is started or continued. The confirmation states whichever of the two actually applies.
- **The transition exists; the surface does not.** *(Updated 2026-08-07.)* ADR 0005 listed frame dismissal among its new transitions and it went unbuilt until **#24**, which shipped `TransitionPayload::DismissFrame` and the `dismiss_frame` command as an **M3a enabling slice** — domain and command only, with replay and snapshot coverage. **Dismissal is still not user-reachable**, and this decision is therefore designed and half-built rather than delivered: nothing calls that command until this panel provides the caller.

**6. A frame states what it is blocking.** Where a frame's paused block is `Pending` — which is *every* open frame, by `derived_status`'s definition — the panel says that the block cannot be reshaped until this frame is resolved, and resolving it here is what lifts the restriction. This is the missing half of `timeline-editor.md` decision 7 and of the History View's disabled time fields: those surfaces state a restriction, and this is the only surface that can remove it.

**7. Depth is stated numerically and the list scrolls; nothing is truncated silently.** Stack depth is unbounded in the domain — `Interrupt` has no depth limit. The panel shows the count as a number so depth is never established by counting rows, and the list scrolls past its visible height rather than capping. **No "and N more" summarisation**, which would hide exactly the frame the user is looking for.

**8. The panel is scoped to the same view range as the History View and Timeline Editor for its "Earlier" part, and is never range-scoped for its "Now" part.** The open stack is current state, not history: a frame opened yesterday is still open now and must appear regardless of what range is selected. `timeline-editor.md` decision 2 established the shared **view range** over the two history surfaces; "Earlier" joins them, "Now" cannot. Mixing them was the first draft's error and would have hidden a live frame behind a date filter.

## Alternatives

### A. Where the panel lives

1. **A third permanent column** beside the Timeline Editor and History View — everything visible at once, no interaction to reach it. **Rejected on arithmetic:** at the 800 px window minimum, 96 px is already committed to the timeline and the History View's ten columns already overflow. A third column is paid for entirely by the table.
2. **A modal or dialog** — unlimited room, and rejected: it hides the timeline and the active task at the moment the user is trying to relate the stack to them, and modality is chrome `users.md` rules out for an expert sole user.
3. **Its own tab** — rejected for the reason `visual-redesign.md` B.2 rejects extending tabs: it fragments a surface the user opens to see everything at once, and it would put the stack at the same level as the day itself.
4. **A disclosure attached to the active-task region.** **Chosen.** Zero cost when closed, adjacent to the thing it describes, and it satisfies `mvp.md:25`'s "optional panel" literally.

### B. What "history" means here

1. **A nesting tree of all interruptions.** The intuitive reading of the name, and **impossible**: the interruption→resumption pairing is not in the projection and compaction destroys the log lines that carry it. Reconstructing it from chronological adjacency plus name equality would be inference (barred by ADR 0001) keyed on a field **R2** already identifies as unreliable. Rejected on evidence, not on cost.
2. **The open stack only** — completely honest, and too narrow: `interruption-stack.md:36` states the panel exists "so the user can confirm nothing was lost," and once a frame is resolved it vanishes from the stack entirely. A panel that only shows open frames cannot answer "did I ever get back to that?"
3. **Open stack, plus a flat chronological list of resolved interruptions.** **Chosen.** Both parts are exact and both survive compaction. The flatness is a stated property, not an omission.

### C. Whether dismissal needs its own confirmation

1. **No confirmation** — consistent with the fast path, and rejected: the fast path is the hotkeys, and this is the deliberate surface. Dismissal writes `Skipped` permanently with no undo.
2. **A general undo instead** — better in the abstract, and out of scope: MVP has no undo anywhere, and inventing one for a single action would be the largest thing in this document.
3. **An inline confirmation stating the consequence.** **Chosen.** Same pattern the History View's Delete already uses, including its full-width row rather than an inline control, for the reason recorded there: at 800 px an inline confirmation can be pushed off-screen and clicked unread.

### D. What replaces the always-visible list

1. **Keep it and add the panel alongside** — no behaviour change, and rejected: two surfaces would show the same stack, and the accepted design explicitly says the default shows neither.
2. **Remove it entirely, depth included** — cleanest, and rejected: depth is one of the four facts `visual-redesign.md` requires the *widget* to show at a glance, and the dashboard hiding what the widget shows would be incoherent.
3. **Depth stays as a number; the list moves behind the disclosure.** **Chosen.**

## Trade-offs

| | Placement | Content | Dismissal | Default visibility |
|---|---|---|---|---|
| **Chosen** | Disclosure on the active-task region | Open stack + flat resolved list | Any frame, confirmed, writes `Skipped` | Depth only; list behind disclosure |
| Complexity | Low — no layout competition | **Moderate — two data shapes, two scoping rules** | **High — requires a new transition and command** | Low — a removal plus a counter |
| Reversibility | High | Moderate — the flat list's framing is load-bearing | **Low — a transition is an on-disk fact once written** | High |
| UX impact | Costs nothing when closed | "Nothing was lost" becomes answerable | A stuck frame stops being permanent | Default matches the accepted design at last |
| Risk if wrong | The disclosure is never opened and the panel is dead weight | The flat list is read as a flattened tree (**R18**) | A mis-dismissal permanently marks work skipped (**R17**) | Depth alone proves too little to prompt opening it |

**Not a trade-off, recorded so it is not mistaken for one:** every display in this document is derived from `StackView` as it is already serialised. The one thing this feature adds to the record is the dismissal transition, which ADR 0005 already decided.

## UX

Owned by the ux-designer. Stack semantics are `interruption-stack.md`'s and are not restated.

### Default state

- The active-task region shows the current task and the two return actions, **exactly as today**, plus **stack depth as a number**.
- **The disclosure's control is a labelled control that carries the depth, not the bare number.** *(Corrected on review. The first draft made the number itself the control, which fails `visual-redesign.md` E.3: these gestures are `infrequent`, so they must be **findable rather than memorable**, and a bare integer is not a visible affordance — it reads as a readout. The depth stays visible either way; what changed is that something on screen says it can be opened.)*
- **It remains openable at depth 0**, because "Earlier" is independent of the stack and is usually non-empty on a day with any interrupted work. *(Also corrected on review: the first draft called the control "inert" at depth 0, which contradicted decision 2 one section earlier — it would have made the resolved-interruption record unreachable on exactly the days it is most worth reading, once every frame has been resolved.)*
- **Nothing about the panel animates or expands on its own.** A stack change is not a reason to open a panel over the user's work.

### Now — the open stack

- Frames are listed **deepest first**, matching the order the returns will unwind them, so the first row is what *Return to Previous* acts on.

  *(Noted on the second review pass: deepest-first **is** most-recent-first, since `Interrupt` pushes. So "Now", "Earlier" and the History View all read newest-first, and the surface has one ordering rule rather than three that happen to coexist. This was true by accident in the first draft and is recorded so a later change to one list is understood as a change to all three.)*
- Each frame shows: the paused task's name (with project/client where set), **when it was paused**, and **how long it has been waiting** — the latter derived as `now − pausedBlock.end`, never stored.
- The **root** frame is marked as *Return to Original*'s target.
- Each frame states that its paused block cannot be reshaped until the frame is resolved (decision 6).
- Each frame carries exactly one action: **Dismiss**. No return control, at any depth (decision 4).

### The return preview

- Stated above the list, in terms of the actual tasks: which block *Return to Previous* resumes, which block *Return to Original* resumes, and **how many blocks the latter marks skipped**.
- At depth 1, one statement, noting that the two actions coincide.
- At depth 0 the preview is absent rather than empty — there is nothing to say.

### Earlier — interrupted work, resolved

- A chronological list, **most recent first**, of blocks in the current view range whose status is `Resumed` or `Skipped`, each showing its identity, its span, and which of the two it was.

  *(Ordering settled on review, which found the first draft using oldest-first while the History View immediately beside it sorts most-recent-first. Two adjacent lists of the same day's blocks in opposite orders is the same defect `timeline-editor.md`'s Alternative L rejected when it refused to let two views of one dataset disagree. The day-narrative reading that motivated oldest-first is not worth a contradiction on one screen.)*

- **Membership is by the block's `start`**, exactly as `export.md:95` decides for export and `timeline-editor.md` decision 2 for the view range — one membership rule across the product. *(Stated after review found it undefined: a block interrupted just before midnight and resumed after it would otherwise be assigned by whichever rule the implementer picked.)*

- **A block skipped by *Return to Original* and one explicitly dismissed are indistinguishable here, and that is by decision, not omission.** ADR 0005 rejected an `Abandoned` value because the domain fact is identical — *this work was interrupted and never resumed*. The panel must not imply a distinction the record does not carry, and no future version may add one without a new ADR.
- **Its flatness is stated on the surface**: this is a record of what was interrupted and what became of it, not of what interrupted what. Anchor does not keep the latter.
- `Skipped` and `Resumed` are distinguishable **without colour**, on the same grounds `visual-redesign.md` applies to provenance.
- Empty state: a plain statement that nothing in this range was interrupted. No call to action.

### Dismissal

- **Confirmed inline, in its own full-width row**, naming the task and stating three things: the block will be recorded as **skipped**, there is **no undo**, and — if it is the last frame — that the stack becomes empty. Whether *Complete* becomes available with it depends on whether a task is running: it does when one is, and does not while paused, where nothing is active to complete. The confirmation says which.
- On success the frame disappears from **Now** and the block appears in **Earlier** as `Skipped` — the same projection update every other command already triggers, so both windows agree by construction.
- A rejected dismissal surfaces the domain's own error, untranslated.

### Motion

Inherits `visual-redesign.md`'s 150–200 ms ease-out for the disclosure itself. The waiting-time figures update on the existing one-second tick; nothing else animates.

## Technical Constraints

**Inherited, not open:**

- **Dismissal writes `Skipped`** — ADR 0005, decided. No `Abandoned` value, and this document does not reopen it.
- **`derived_interruption_status` is the only permitted reading** of interruption state. Reading `interruption_outcome` directly is a bug: absent means *never interrupted* **or** *interrupted and unresolved*, and only the live stack separates them (ADR 0005, risk **R1**).
- **`Complete` is rejected while the stack is non-empty, and separately requires an active task** — two preconditions, checked in that order. Emptying the stack clears the first and never the second.
- **The active block is never a frame's paused block** — a frame's target is always already closed.
- **Blocks are addressed by stable id** ([ADR 0006](../../decisions/0006-stable-persistent-time-block-identity.md)), which is what makes a frame's reference resolvable across sessions.
- **No inference.** ADR 0001.

**This surface's own:**

- **The dismissal transition is built; this panel supplies its only caller.** *(Updated 2026-08-07, #24.)* `TransitionPayload::DismissFrame` and the `dismiss_frame` command resolve the named frame's block to `Skipped` through the same `resolve_paused` path the returns use, remove that frame, and leave `active` untouched — keyed on the paused block's id, never a stack index. That was a domain change and belongs to the state machine and its tests, which is why it shipped separately. **What remains here is the caller**: until this panel ships, the command has none, and the accepted feature is not delivered.
- **"Earlier" depends on the shared view range, which is accepted but unbuilt.** *(Added on review, which caught the paragraph above claiming dismissal was the feature's "only implementation dependency" — it is not.)* `timeline-editor.md` decision 2 creates that range; today no dashboard range exists at all, since `resolveRange` serves `doExport` alone and the History View is unfiltered. **The two parts of this panel are therefore separately shippable, and should be sequenced that way:** "Now" plus dismissal depends only on the new transition, and is what closes the `BlockReferencedByOpenFrame` and restricted-block gaps; "Earlier" waits for the view range. Building "Earlier" first would either invent a second range control or ship unfiltered, and both contradict decision 8.
- **Frame identity in the UI is the paused block's id, not a stack index.** An index is invalidated by any concurrent change; the id is durable by ADR 0006. A dismissal command keyed on position could resolve the wrong frame if the stack moved between render and click.
- **"Now" is never range-scoped; "Earlier" always is** (decision 8). One query cannot serve both, and using the range for both would hide a live frame.
- **Waiting time is computed in the UI from `pausedBlock.end`, never persisted and never sent to the domain.** A `Utc::now()` inside the state machine would make replay depend on when it runs.
- **`StackError::BlockReferencedByOpenFrame`'s message becomes true only when this panel ships.** *(Updated 2026-08-07.)* Its advice to *"resume or dismiss"* needs **both** halves: the transition, which #24 built, and a user-reachable caller, which only this panel supplies. The transition alone is **not** sufficient — a user reading that message still cannot act on it, so the message stays false until the caller exists. Either it ships here and the text stands, or the text must be corrected; it may not keep pointing at something a user cannot reach. Tracked below as an acceptance criterion rather than left to notice.
- **`app/src/routes/+page.svelte` is classified binary by git** — two NUL bytes in an `R.uniqBy` key — so this feature gets no textual diffs or line-level merges on the file it modifies. `visual-redesign.md` notes the rewrite may resolve this with a non-NUL delimiter.
- **Implementation is blocked on `visual-redesign.md`'s inputs** — the spacing scale's steps, the hue palette and its size, the font weights — which are not in this repository. This document may be accepted against the accepted contract; building it may not begin before those arrive. The **dismissal transition is not so blocked**, being domain work with no visual surface.

## Acceptance Criteria

**Default state and disclosure**

- With the panel closed, the dashboard shows the current task, the two return actions, and stack depth as a number — and **no frame list anywhere**, at any depth.
- Stack depth displayed on the dashboard equals `StackView.stack.length` at all times, and equals the depth the mini widget shows.
- The panel never opens by itself, including when a frame is pushed or popped while it is closed.
- The disclosure is operable at **depth 0** and shows "Earlier" there; it is not disabled by an empty stack.
- The control that opens the panel is labelled, not a bare number, and is reachable without knowing it exists beforehand (`visual-redesign.md` E.3's `infrequent` class).

**The open stack**

- Frames are listed deepest-first, and the first row's task is the one *Return to Previous* resumes.
- Each frame shows a paused-at time equal to its paused block's `end`, and a waiting time equal to `now − end` that advances once per second.
- Every frame is present. For a stack of depth *d*, the panel renders exactly *d* rows, with no truncation, summarisation, or "and N more" at any depth; the list scrolls instead.
- With the stack empty, "Now" states that nothing is waiting, and the return preview is absent rather than blank.

**The return preview**

- At depth ≥ 2 the panel names the block *Return to Previous* resumes, names the block *Return to Original* resumes, and states a skipped count equal to **depth − 1**.
- After performing *Return to Original* from depth *d*, exactly *d − 1* blocks that were `Pending` read `Skipped` and exactly one reads `Resumed` — matching the count the preview stated beforehand.
- At depth 1 a single statement is shown, noting the two actions coincide.
- The panel offers **no control that resumes a frame**. The only actions reachable from a frame row are Dismiss and inspection.

**Dismissal**

- Dismissing a frame sets its paused block's status to `Skipped`, removes exactly that frame, leaves every other frame's order unchanged, and does not change the active task.
- A frame at any depth can be dismissed, not only the top one.
- Dismissal is confirmed before anything is written, and the confirmation names the task, states that there is no undo, and — when it is the last frame — states that the stack becomes empty, together with whether *Complete* becomes available (it does with a task active, and does not while paused).
- Cancelling a confirmation writes **no** transition.
- Dismissing the **root** frame at depth ≥ 2 is confirmed with a statement naming the task that becomes the new *Return to Original* target, and afterwards *Return to Original* resumes exactly that task.
- After dismissing the last frame **with a task active**, `Complete` is enabled and succeeds, where it was rejected before.
- After dismissing the last frame **while paused**, the stack is empty and `Complete` is still unavailable — the domain rejects it for the absent active task, not for the stack — and it becomes available only once work is started or continued.
- A dismissed frame's block appears in "Earlier" as `Skipped` without a manual refresh, in both windows.
- **`StackError::BlockReferencedByOpenFrame`'s text is true**: every action it names — resume, dismiss — is reachable from the running application. *(This criterion exists because it was false when this document was written.)*
- Dismissing the frame that blocks a restricted block makes that block reshapeable: its Timeline Editor drag affordance and its History View time fields both become available, with no restart.

**Earlier**

- Lists exactly those blocks whose `derived_interruption_status` is `Resumed` or `Skipped` — never `Pending`, never `NeverInterrupted` — **most recent first, in the same order as the History View beside it**, with membership decided by each block's `start`.
- A block skipped by *Return to Original* and one dismissed explicitly render identically, both as `Skipped`.
- Changing the view range changes "Earlier" and **does not change "Now"**. A frame opened before the range's start is still listed under "Now".
- `Resumed` and `Skipped` remain distinguishable with colour removed, in both themes.
- The surface states that the list is not a nesting record.

**Non-regression**

- No transition type other than dismissal, no stored Time Block, and no export output differs before and after this feature ships, proven by an unchanged export from identical input.
- A full Switch/Interrupt/Return cycle on the hotkey and widget paths is unchanged — no markup, handler, or command on either is touched.
- Replaying a log containing dismissals reproduces the same projection as the live run that wrote them, and a snapshot taken after a dismissal restores it identically.

---

**Keeping this current:** if an ADR later changes how this feature is built, update Technical Constraints and cross-link the ADR — don't leave this doc describing a superseded approach.
