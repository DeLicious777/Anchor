# Milestones

_Last updated: 2026-08-07_

Sequencing reflects the dependency structure in `planning/epics/` — not arbitrary prioritization.

## M1 — Core Tracking Loop

**Epic:** [Interruption Stack](epics/interruption-stack.md)

Outcome: a full workday can be tracked end-to-end (Switch/Interrupt/Return, hotkeys, mini widget, dashboard shell) with durable, crash-safe storage. Nothing usable exists before this ships.

## M2 — MVP Complete

**Epics:** [Task Templates](epics/task-templates.md), [Export](epics/export.md)

Outcome: recurring activities are low-friction to track, and a day/range can be exported to XLSX/JSON in a billing-ready form. Both epics depend only on M1, not on each other — can proceed in parallel or either order.

~~Once M2 ships, the MVP is feature-complete per its "In scope" list.~~ **No longer true (2026-07-28).** The Concept revision added four items to MVP scope — timeline reconstruction, the Timeline Editor, Pause/Continue Session, and the interruption-history panel. M1 and M2 are necessary but not sufficient for MVP completeness.

**That gate closed on 2026-08-07.** All four now have accepted feature docs, and every accepted feature doc has an epic in `planning/epics/` — eight files, one per doc. M3 is defined below. *(The intermediate tracking table that stood here between 2026-08-06 and 2026-08-07, listing which docs were still missing, has been removed now that the answer is "none".)*

## M3 — Editable Timeline

**Epics:** [Pause](epics/pause.md), [Interruption History](epics/interruption-history.md), [Visual Redesign](epics/visual-redesign.md), [Timeline Editor](epics/timeline-editor.md), [Timeline Reconstruction](epics/timeline-reconstruction.md)

Outcome: the four items the 2026-07-28 Concept revision added to MVP scope are built. A day can be corrected as well as captured, the interruption stack can be inspected and resolved, and work can stop without falsifying the record.

**M3 is split by *blocker*, not by feature**, because the design-system assets gate visual work and nothing else. Treating the milestone as one block would idle three pieces of finished design behind an external dependency.

### M3a — Backend integration. **Not releasable, and it closes no epic.**

**This is the correction that matters most in this plan, and the first draft got it wrong.** M3a delivers no complete accepted feature. Every one of these three items has visual acceptance criteria that remain blocked on the design system, so none of them is user-reachable when M3a is done, and **no epic can move to `done` on the strength of it.** Treating "the transition exists" as "the behaviour ships" is precisely the confusion between documentation and implementation that risk **R11** exists to catch — and the first draft of this section committed it, claiming frame dismissal would make `StackError::BlockReferencedByOpenFrame`'s advice true. It will not: that message tells the user to *dismiss* an interruption, and until the Interruption History panel exists there is nothing to dismiss it *with*. A command with no caller is exactly the shape **R9** had for four days.

What M3a is actually for: it removes the domain risk from three features so that when the assets arrive, M3c is surface work against a settled and tested backend rather than surface work plus event-model work.

| Work | Issue | Epic | What it is |
|---|---|---|---|
| Frame-dismissal transition + command | #24 | Interruption History | **Enabling slice.** No entry point until the panel ships |
| `Pause` transition, command, **tests only** | #23 | Pause | **Enabling slice.** The binding is *not* in it — see below |
| Shared view range **store and commands only** | #25 | Timeline Editor | **Enabling slice.** Scope narrowed — see below |

**Order: #24 → #23 → #25.** With none of them user-reachable, ordering is about implementation risk rather than user value. #24 is smallest and reuses the existing `resolve_paused` path. #23 is the state-machine change. #25 is independent of both, being a new store that touches neither.

**The sixth Pause binding is deferred out of M3a, to M3c.** *(Decided 2026-08-07, on evidence.)* The earlier plan said to land the binding but not enable it — which the architecture cannot express. `hotkeys::register_bindings` iterates the actions and calls `global_shortcut().register` on each unconditionally; there is no enabled flag, no skip list, and no dormant-registration path. Adding a `HotkeyAction` variant therefore **makes the accelerator live the moment it ships**, and a live Pause key with no visible paused state realises risk **R19** directly — assumption **A18** names the widget's Paused display as its only mitigation, and that display is asset-blocked.

Inventing a disabled-registration mechanism to work around this would be new architecture introduced by a planning pass, which Planning does not get to do. So M3a lands **the transition, the command and their tests**; the binding, its registration and the `HotkeyBindings` settings-shape change land in M3c beside the Paused display, where the mitigation exists.

**This removes the only durable-format change from M3a.** All three slices are now purely additive: a transition, a command, and a new settings file.

**Scope of #25, decided here:** the **store, its persistence and its get/set commands only.** The range control and History View filtering move to M3c. Filtering the History View before a timeline exists beside it would deliver the entire cost of the change — history hidden by default, a user-visible regression on a shipped surface — with none of its benefit, since `timeline-editor.md` decision 2 justifies the filtering *solely* by two adjacent views needing to agree, and the second view would not exist yet.

### M3b — Visual foundation

**Visual Redesign**, in full. It cannot start before the assets arrive and everything in M3c depends on it. This is the milestone's critical path and its schedule is not under the project's control.

### M3c — Surfaces, once the foundation exists

In dependency order: **Timeline Editor** → **Timeline Reconstruction**'s remaining Add/Move/clamping → **Interruption History**'s panel and **Pause**'s paused-state display, which are independent of each other and of the reconstruction work.

### M3c also completes the three M3a slices

Each M3a item is finished by a surface, and the epic closes then, not before: the Interruption History panel gives dismissal its entry point (and only then does `BlockReferencedByOpenFrame`'s advice become true); Pause's display arrives **together with its sixth binding and the `HotkeyBindings` settings-shape change**, since the architecture cannot land one without the other; and the range control plus History View filtering complete the shared view range.

### Verification

`docs/verification-checklist.md` is the exit gate for **M3c**, not for M3a — M3a has no user-reachable behaviour for a manual end-to-end pass to exercise, and running it there would produce a green result that means nothing. M3a's own gate is the automated suite: replay and snapshot coverage for every new transition. *(The five-field-settings-file compatibility check moves to M3c with the binding.)*

**Sequence, unambiguously:** **Graphify regeneration first**, then #24, #23, #25. The graph is committed and currently describes a repository that no longer exists — eight feature docs, seven accepted ADRs and a substantially grown glossary have landed since it was generated — so regenerating it *after* implementation starts would bake a stale snapshot into the one artifact other tooling reads as current.

Step 5b (correcting an inferred end) and step 7's restricted-block check were added on 2026-08-06 and have never been run. They exercise shipped behaviour and **do not need to wait for M3** — see "Not in M3".

### Not in M3

Live defects and stale board state are tracked as issues, **not** promoted to milestone scope: debt does not become a roadmap item because it is convenient to schedule it there. `tauri.conf.json`'s `"csp": null` (**R13**(c)) is likewise independent of the redesign and should not wait for it.

## Later (not yet epics)

Per `docs/roadmap.md` "Later": cross-platform support, additional clients (CLI/browser/mobile), and anything currently in MVP's "Explicitly out of scope" list. These don't get epics until a future Discovery/Design pass revisits them.

---

**Keeping this current:** when an epic's status changes (e.g. `planned` → `in-progress` → `done`), update this file's outcome framing if the sequencing assumption changes — don't let this drift silently from `planning/epics/`.
