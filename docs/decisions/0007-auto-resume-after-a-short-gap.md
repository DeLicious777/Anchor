---
status: proposed
date: 2026-08-03
owner: erich
related: [docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/product/features/interruption-stack.md, docs/principles.md, docs/risks.md, docs/verification-checklist.md]
---

# 0007: Auto-Resume After a Short Gap

> `status: proposed`. **Supersedes the resolution of [ADR 0005](0005-event-model-time-block-metadata-and-reconstruction-transitions.md)'s open item 9** (2026-07-29, "wake stops auto-starting"). That ADR's other decisions are untouched.
>
> **This is a requirement change, not implementation evidence.** ADR 0005's reasoning was not found to be wrong, and nothing in the code contradicts it. The author has re-weighed the trade it made — stated there as *"an accurate record over a convenient one"* — and wants the convenience back, bounded. Recorded plainly because [`.claude/docs-standards.md`](../../.claude/docs-standards.md) distinguishes the two, and a later reader needs to know which happened here.
>
> **One genuine piece of implementation evidence did surface while writing this**, and it is separate from the requirement change: the anomaly ADR 0005 said it removed was only half removed. See Context.

## Context

### What ADR 0005 decided, and why

On sleep/wake, `power.rs` emits `RecoverGap` and nothing else; the user resumes deliberately. The deciding argument was **consistency**: crash recovery already behaved that way, and *"wake and crash are the same class of event: Anchor lost continuity and cannot know what happened in the gap."*

Two alternatives were rejected, and both rejections still stand on their own terms:

- *"The user was demonstrably at the machine"* — **presence is not resumption.** Dismissing a lock screen says nothing about which task someone intends to work on.
- *Renaming the transition* — a documentation fix for a behaviour problem.

The cost was stated explicitly: the user presses the capture hotkey after a wake instead of finding tracking already running.

### What actually changed

Verification run 1 (2026-08-03, `validated-baseline-1`) exercised this on the running application for the first time. The behaviour is correct and was recorded as passing. It is also, in daily use, the thing the author most wants different: after a crash-and-relaunch the task must be re-started by hand, and in the overwhelmingly common case — the app died and came back within a few minutes — the user *was* still working on the same thing.

**The uncertainty ADR 0005 named is real and does not go away.** What is being re-weighed is how much of it a *short* gap carries. This ADR takes the position that under some bound, resumption is the better default guess, and that being wrong is cheap because the mistake is visible immediately and correctable in one action.

### The evidence: the anomaly was only half removed

ADR 0005 said handling wake and crash differently *"was the anomaly, and this removes it rather than adding a rule."* Checked against the code, it removed the auto-start half and left another in place:

| | gap < 90s | gap ≥ 90s |
|---|---|---|
| **Wake** (`power.rs::resolve_resume_gap`) | **no transition at all** — the block simply continues | `RecoverGap`, no resume |
| **Startup** (`state.rs::init`) | **`RecoverGap` regardless** | `RecoverGap`, no resume |

`power.rs` has a `RESUME_GAP_THRESHOLD_SECS = 90` grace window, chosen to sit "safely past the 60s heartbeat interval, to avoid false positives from ordinary scheduling jitter." `AppState::init` has no equivalent: any leftover active block is closed, however brief the outage.

So a 13-second sleep-wake is a non-event, while a 13-second crash-relaunch closes the block. Verification run 1 hit exactly this: `Anchor 5` was killed 13 seconds after starting, and because no heartbeat had landed, the last durable write *was* the start — producing a **zero-duration block**. Correct under the current rules, within risk **R4**'s bound, and still an artifact nobody chose.

**This also means the project already holds "a short gap implies continuity" as a working principle.** It applies it on one path only, and at 90 seconds. What follows is less a new idea than making that principle explicit, symmetric, and adjustable.

## Options Considered

1. **Keep ADR 0005's resolution.** No auto-resume anywhere. Consistent and maximally cautious about inventing records.
2. **Always auto-resume**, from the moment of recovery, regardless of gap length.
3. **Auto-resume only when the gap is shorter than a threshold.** Below it, assume continuity; above it, require a deliberate resume.
4. **Auto-resume covering the gap** — the resumed block starts at the inferred end, so downtime becomes tracked work.
5. **Prompt on relaunch** — "were you still working on X?"

## Trade-offs

| | 1. No resume | 2. Always resume | 3. Resume under a threshold | 4. Resume covering the gap | 5. Prompt |
|---|---|---|---|---|---|
| Record accuracy | Highest — never invents a start | Poor — an overnight crash resumes a task the user is not doing | Good — bounded exposure, wrong only within the window | **Unacceptable** — bills every hour the app was not running | High |
| Convenience | Lowest | Highest | High for the common case | High | Low — chrome on every recovery |
| Consistency with the 90s grace | Contradicts it (wake already assumes continuity) | Extends it without limit | **Generalises it** | Unrelated | Unrelated |
| Cost when wrong | None | A phantom block that may run for hours unnoticed | A phantom block, visible immediately in the widget | Silently inflated billable total — risk **R4**'s failure mode | None |
| Fit with the persona | Neutral | Neutral | Neutral | Neutral | **Rejected by `users.md`** — no explanatory chrome, no dialogs the expert user did not ask for |

**Option 4 is the one to reject loudly.** It is the only option that can silently inflate a billed total, which is the exact failure `R4`, `A15` and the heartbeat bound all exist to prevent. That an hour-long gap would be counted as work makes it strictly worse than a phantom block the user can see and delete.

**Option 5 fails on `docs/product/users.md`**: the persona is explicitly *"comfortable with hotkeys, not needing onboarding"*, and `timeline-reconstruction.md` reserves prompts for irreversible actions. A dialog on every recovery is the explanatory chrome this product refuses.

## Decision

**One rule, applied identically on both the startup and wake paths**, replacing the current split:

| Gap since the last durable write | Behaviour |
|---|---|
| **< 90 seconds** (`CONTINUITY_THRESHOLD`) | No transition at all. The block continues, as if nothing happened. |
| **90 seconds – 1 hour** (`RESUME_LIMIT`) | `RecoverGap` closes the block at the last durable write, **then a `Start` for the same name/project/client** opens a new block at the moment of recovery. |
| **≥ 1 hour** | `RecoverGap` only. The user resumes deliberately. Today's behaviour. |

Three consequences of stating it this way:

- **The symmetry ADR 0005 established is kept, and extended.** Its deciding argument — wake and crash are the same class of event — is honoured more fully than before, since the 90-second grace now applies to both rather than to wake alone.
- **The zero-duration block from verification run 1 stops happening.** A crash-relaunch inside 90 seconds becomes a non-event on both paths, which is what it always was on one of them.
- **The gap itself is never counted as work.** The resumed block starts at recovery time, not at the inferred end. The timeline shows an honest hole where Anchor was not running.

**`RESUME_LIMIT = 1 hour` is unvalidated**, chosen as a plausible default rather than measured — the same honest framing ADR 0004's `N = 500` and the 60-second heartbeat use, per [`principles.md`](../principles.md) #7. **Revisit it if** phantom resumed blocks turn out to be common in practice (lower it), or if the manual re-start after a lunch-length outage stays annoying (raise it). `CONTINUITY_THRESHOLD` keeps its existing 90-second value and its existing justification.

**The resumed block is `CaptureOrigin::LiveCapture`**, indistinguishable from one the user started. It is live capture of work happening now, not reconstruction of the past, and Capture Rate should count it as captured. No new `CaptureOrigin` variant: nothing consumes such a distinction, and adding one would touch the export contract and the History View to serve a case no accepted requirement names ([`principles.md`](../principles.md) #1).

### What this knowingly gives up

**Anchor now invents a start time.** ADR 0005 named that as [`principles.md`](../principles.md) #3 — *"the state model must never force users to create inaccurate records"* — and the objection is not answered by this ADR, it is **accepted within a bound**. Three things make the trade defensible, and none of them makes it free:

1. The window is one hour, not unlimited.
2. The mistake is **immediately visible** — the widget and dashboard both show a task running the moment it happens, which is not true of a wrong *end* time buried in history.
3. Correcting it is one action (Complete, then Delete if the block is spurious), against a block the user can see.

If phantom blocks prove common, the honest response is to lower the limit or revert to option 1, not to add machinery that guesses better.

## Consequences

- **Replay stays deterministic.** The gap length is computed once, live, from the wall clock, and the resulting `Start` is durably logged like any other. Replay replays the logged transition; it never re-decides. No wall-clock read enters `InterruptionStack::apply`.
- **`power.rs::resolve_resume_gap` must return two transitions again.** Its current doc comment records that the return type was deliberately collapsed to one *"so the second transition cannot be reintroduced by accident — there is no longer anywhere to put it."* That guard was correct for ADR 0005's decision and is being removed on purpose, not defeated.
- **Three tests assert the superseded behaviour** and must change with the decision, not be worked around: `stack.rs::recover_gap_closes_active_with_inferred_end_and_does_not_auto_resume` (still valid — `RecoverGap` alone still does not resume; the *caller* now appends a `Start`), `state.rs::leftover_active_entry_is_closed_as_recovered_gap_with_no_auto_resume`, and `power.rs`'s threshold tests. None is on the protected-invariant list (replay determinism, stable identity, snapshot recovery, A15, append-only boundary, delete/export/replay consistency, live-vs-replay equivalence).
- **`docs/verification-checklist.md` step 4 changes.** It currently reads *"nothing auto-resumes"*, which will be false for gaps under an hour. The checklist gains a case: a short-gap kill should resume, a long-gap kill should not.
- **`validated-baseline-1` describes behaviour this supersedes.** The tag stays accurate for the commit it names. A new verification run is needed after this ships; the baseline is not invalidated, it is dated.
- **`interruption-stack.md` needs its gap-recovery wording revised** — it is `accepted`, and it currently describes recovery as closing the entry with no successor.
- **The interruption stack is untouched.** `RecoverGap` still pushes no frame, so a task interrupted by a gap still has no return path; the resumed block is a new independent block, exactly as a Return produces. That gap remains open and is still ADR 0005's to name — it belongs with Pause (#16).

## Relationship to existing ADRs

- **[ADR 0005](0005-event-model-time-block-metadata-and-reconstruction-transitions.md)** — its open item 9 resolution is **superseded** by this ADR. Everything else in it stands: the three-field metadata model, the reconstruction transitions, the snapshot payload guarantee, and the remaining eight open items' resolutions.
- **[ADR 0001](0001-manual-assisted-tracking-for-mvp.md)** — **unaffected, and worth checking rather than assuming.** Auto-resume is not activity inference: Anchor does not observe what the user is doing and conclude anything. It continues a task the user themselves started, after an outage of its own making. No idle detection, no window-title watching, no screen monitoring. The manual-capture premise holds.
- **[ADR 0004](0004-transition-log-format-and-torn-write-scheme.md)** — untouched. No new transition type, no record-shape change; `RecoverGap` and `Start` both already exist.
- **[ADR 0006](0006-stable-persistent-time-block-identity.md)** — untouched. The resumed block gets its identity from the `Start`'s own `seq`, like every other created block, so the one-block-per-transition invariant holds.

## Follow-up work

- Implement the shared three-zone rule in one place, called by both `state::AppState::init` and `power.rs`, so the two paths cannot drift apart again — the drift this ADR's Context documents is what made a single shared rule worth insisting on.
- Revise `interruption-stack.md`'s gap-recovery description and `docs/verification-checklist.md` step 4 in the same change as the code, not after it.
- Re-run the verification checklist and record `validated-baseline-2`.
