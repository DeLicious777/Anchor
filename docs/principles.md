# Design Principles

_Established 2026-07-28, during the Concept revision's `grill-with-docs` session._

These are not aspirations. Each one was derived from a decision that was actually made, and each has already been used to reject something concrete. They exist so future decisions can be evaluated against a consistent philosophy rather than judged in isolation.

Distinct from [`architecture/constraints.md`](architecture/constraints.md): a constraint is a non-negotiable boundary that forecloses options. A principle is a lens for choosing among options that remain open.

---

## 1. Every feature must solve a clearly stated problem

Not "it feels like a natural operation." A feature earns its place by naming the user problem it solves and showing that no existing primitive already solves it.

**Rejected under this principle:** Split and Merge. Both are plausible timeline gestures; neither had a problem statement. Merge turned out to duplicate what export already does (grouping and summing equivalent work), while adding adjacency rules, duration inconsistency, and two kinds of provenance laundering. Split was reachable via resize + add.

**Admitted under it:** Pause, which acquired a concrete problem statement — *stop tracking without unwinding or abandoning an open interruption stack* — and passed.

## 2. Compete on design philosophy, not feature count

Timelines, exports, tags, and reports are features; competitors have them and will keep adding them. What is hard to converge on accidentally is an opinionated model of how work should be captured, interrupted, reconstructed, and trusted.

**Applied:** after research found ManicTime and Memtime already shipping "local-first timeline + billing-grade export," the Concept stopped claiming a market gap and reframed as build-vs-buy. Anchor exists because those tools capture *automatically*, which [ADR 0001](decisions/0001-manual-assisted-tracking-for-mvp.md) deliberately rejected — not because they lack features.

## 3. The state model must never force users to create inaccurate records

When the model demands a fabricated event to reach a legal state, **the model is wrong, not the user**.

**Found three times by applying it:** (a) `Complete` is rejected while the interruption stack is non-empty, so stopping work mid-stack required either leaving the clock running or fabricating returns. (b) A stack frame could only be resolved by returning to it, so abandoning a task required fabricating a resumption — closed by an explicit dismissal writing `Skipped`. (c) **The fix for (a) initially reproduced (a).** Pause alone left the app at `active == None` with a non-empty stack, from which no transition was legal, so unwinding required starting a task the user hadn't done. (a) is closed only by Pause **plus** making `ReturnPrevious` and `ReturnOriginal` legal with no active task. Caught by second-pass verification, and the sharpest evidence for this principle: applying it once is not enough, because the remedy can recreate the condition.

## 4. Only materialised state survives

[ADR 0004](decisions/0004-transition-log-format-and-torn-write-scheme.md) compacts by writing a snapshot of current state and truncating the log. Anything that must outlive compaction has to be written into state; anything left implicit in the event stream is lost.

**Applied:** `CaptureOrigin` lives on the `TimeBlock` rather than being derived from transitions. **Also applied in reverse:** `Pending` is *not* persisted on the block, because the unresolved stack frame already carries it and the snapshot preserves the stack.

_Status note: compaction is not yet implemented ([`log/reader.rs`](../app/src-tauri/src/log/reader.rs)), so this is a designed constraint rather than an observed one. It still governs, because ADR 0004 is accepted._

## 5. Make important distinctions explicit in the model, not implicit in behaviour

Where principle 4 answers *where* information lives, this one answers *whether the distinction should exist at all*. If a difference matters, give it a field — don't leave it inferable from adjacency, ordering, or live state that evaporates.

**Applied:** `EndDetermination`, `CaptureOrigin`, and `InterruptionOutcome` are three fields because they answer three questions. Merge was removed rather than given precedence rules, because precedence rules are exactly this principle inverted.

## 6. Persistence captures what became true; the event model captures how

Different user actions may legitimately produce the same persisted state when they express the same domain outcome.

**Applied:** Return to Original and explicit dismissal both write `InterruptionOutcome::Skipped`. They are different routes to the same fact — *this work was interrupted and never resumed*. Adding an `Abandoned` value would have encoded the route into the outcome, conflating the two models.

## 7. When evidence is unavailable, commit provisional targets with explicit revisit triggers

The failure mode is not choosing a number without evidence — it is choosing one without admitting it. State the number, label it unvalidated, and name the condition that would change it.

**Precedent:** ADR 0004's `N = 500` compaction threshold and `interruption-stack.md`'s 60-second heartbeat, both explicitly "chosen as a low-cost default, not measured."

**Applied:** [`vision.md`](vision/vision.md)'s Capture Rate (≥90%) and Capture Latency (≤1s) targets.

## 8. Verify a claim against documentation *and* implementation before it becomes load-bearing

Not "keep the docs in sync" — that treats the failure as drift, and it usually isn't. Sometimes the documentation was wrong, sometimes the implementation was ahead of it, sometimes both were incomplete while a decision was being made on top of them. The common cause is that nothing establishes which source is authoritative before something depends on it. This holds even before code exists: the claim is then checked against a documented architectural decision instead.

**Found six instances in one session (2026-07-28), only because someone went looking:** an `accepted` doc promising `recovered-gap` correction with no mechanism (R9); `explicit` documented as "user-finished" while the code wrote it at six sites for four meanings; Switch's stack behaviour undocumented while a Vision promise depended on it; ADR 0004 specifying compaction's mechanism but never its payload; compaction unimplemented while several docs reasoned about what survives it — and, most expensively, **Pause decided as "Complete + Start" when the code rejects `Complete` with a non-empty stack.** That decision was made *wrongly* because nobody checked.

Tracked as risk **R11**, whose candidate mitigation is this principle applied as a gate: before any design document reaches `accepted`, every implementation-dependent claim in it is verified. Note R11 does not close when any single instance is fixed.

---

**Keeping this current:** a principle earns a place here only once it has rejected or admitted something concrete. If a principle has never changed a decision, it is a slogan — remove it. If a decision contradicts a principle, one of the two is wrong; resolve it rather than letting both stand.
