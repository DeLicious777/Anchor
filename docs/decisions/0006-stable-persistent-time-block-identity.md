---
status: draft
date: 2026-07-29
owner: erich
related: [docs/decisions/0004-transition-log-format-and-torn-write-scheme.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/product/features/timeline-reconstruction.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/architecture/constraints.md, docs/principles.md, docs/risks.md]
---

# 0006: Stable, Persistent Time Block Identity

> `status: draft`. This is the architectural prerequisite for timeline reconstruction (#15) — that feature cannot be accepted or implemented until this is settled.

## Context

Timeline reconstruction requires four of its five operations — **Move, Resize, Edit Identity, Delete** — to name an existing `TimeBlock`. There is currently no way for them to do so.

**Time Block identity is regenerated on every replay.** `model.rs`'s `TimeBlock::new` assigns `id: Uuid::new_v4()` at construction, and construction happens during replay exactly as it does during live capture. `TransitionRecord` carries only `seq`, `timestamp`, and a `TransitionPayload`, and **no payload variant carries a block id**. So a `Resize { target, … }` written today names a UUID that will not exist after the next restart. Replay fails to resolve it, `log/reader.rs` escalates that to `ReplayError::Inconsistent`, `AppState::init` propagates it — and the app does not start.

**Why this has never surfaced.** `StackFrame.paused_time_block_id` already references a block by id and works correctly across restarts, which looks like a counter-example. It isn't: that reference is **intra-replay**. The frame and the block it names are created in the same `apply` call, in the same pass, so the reference only has to be internally consistent within one run. It never has to match an id from a previous run.

**Reconstruction introduces the first inter-replay reference** — written in run *N*, resolved in run *N+1*. That is a new architectural requirement, and it is why this needs a decision rather than an implementation detail.

Found by independent review, after `timeline-reconstruction.md` asserted the opposite: *"No new requirement beyond ADR 0005's existing snapshot-payload guarantee."* That claim was made without checking whether the operations were addressable at all — [`principles.md`](../principles.md) #8, and risk **R11**.

### Verified facts this decision rests on

- Every `TimeBlock` in the domain is constructed inside `stack.rs::apply` — five call sites, one per creating arm (`Start`, `Switch`, `Interrupt`, `ReturnPrevious`, `ReturnOriginal`).
- `Complete`, `RecoverGap`, `Rename` and `Heartbeat` create none.
- **Every creating transition therefore creates exactly one Time Block.**
- `seq` is per-line, monotonically increasing, and assigned to every line including heartbeats ([ADR 0004](0004-transition-log-format-and-torn-write-scheme.md)).

## Options Considered

### A. Persist an explicit UUID in every creating transition

The writer generates a UUID and puts it in the payload; replay uses the carried value instead of generating one.

- **Replay behaviour**: correct and stable, for blocks created after the change.
- **Wire-format impact**: five variants change. Two of them — `ReturnPrevious` and `ReturnOriginal` — are currently **unit variants**, serialised as `{"type":"return-previous"}`. Adding a field converts them to struct variants. That is a change of shape, not just an added field.
- **ADR 0004 compatibility**: this genuinely modifies the on-disk contract ADR 0004 declares stable. It is achievable — `Rename` was added additively — but it is a real change requiring an amendment.
- **Backward compatibility**: workable via `#[serde(default)]`, deserialising old lines with no id.
- **Effect on existing history**: **this is the disqualifying cost.** Blocks created by pre-change log lines have no carried id, so they fall back to a generated one and remain unaddressable forever. History splits into an editable generation and a non-editable one, permanently, at an arbitrary date. The user's existing record would be partly beyond repair — in a feature whose entire purpose is repairing the record.

**Rejected**, primarily on that last point.

### B. Derive identity deterministically from the creating transition's `seq`

`TimeBlock.id = UUIDv5(ANCHOR_NAMESPACE, seq)`, computed inside `apply` from the record already being applied.

- **Deterministic replay**: replaying the same log always produces the same ids. Identity stops being a property of *when the process ran* and becomes a property of *what the log says*.
- **Compatibility with existing logs**: total. Every line already carries a `seq`, so **every block that has ever existed becomes stably addressable immediately** — no migration, no format change, no generational split.
- **Append-only guarantees**: untouched. Nothing is written that was not written before; the log is read identically. ADR 0004's format, checksums, torn-write handling and compaction are all unaffected.
- **Implementation complexity**: low. One derivation at five call sites, plus enabling the `uuid` crate's `v5` feature.
- **Long-term maintenance**: the derivation becomes a permanent contract — see Consequences. It also depends on an invariant that must be stated and enforced, not assumed.

**Chosen.**

### C. Content or positional addressing

Reference a block by its content — `(start, name)`, or its index in the timeline.

**Rejected, and it is self-defeating rather than merely weak.** Reconstruction's operations mutate exactly the properties that would form the key: Move and Resize change `start`, Edit Identity changes `name`. A `Resize` targeting `(09:00, "Standup")` invalidates its own reference the moment it applies, and replaying two such operations in sequence is undefined. Positional indexing fails for the same reason with the addition of Add and Delete shifting every index after them.

An identifier must be immutable to be an identifier. These properties are the mutable ones by design.

### D. Snapshot or replacement-style references

Instead of naming a block, a reconstruction transition carries the complete intended state of a time range — "the timeline between 09:00 and 10:00 is now X."

**Rejected: it weakens the event model in exchange for avoiding identity.** Each transition stops describing *what the user did* and starts describing *what the result should be*, which is the difference between an event log and a series of snapshots. Concretely, it forfeits the property ADR 0004 exists to provide: that state is the deterministic replay of independent, individually meaningful records. Two edits to overlapping ranges become order-dependent in a way that is no longer expressible as a fold, and the log stops being an audit trail of decisions — the thing `docs/principles.md` #6 relies on, where persistence captures what became true and the event model captures how.

### E. A separate persistent identity registry

A side table mapping stable identifiers to blocks, persisted alongside the log.

**Rejected: it conflicts directly with the event-sourced architecture.** `docs/architecture/constraints.md` states the event log is the single source of truth and all state is a projection replayed from it. A registry is a second durable structure holding state that is not derived from the log, which means it needs its own crash-safety scheme, its own torn-write handling, and a reconciliation story for when the two disagree after an ungraceful shutdown. ADR 0004 chose an append-only log specifically to avoid a second mutable on-disk artifact; this reintroduces one, for a problem option B solves with no new storage at all.

### F. Replay-local incrementing identity

Assign identity from a counter incremented during replay traversal — the *n*-th block created in this pass is block *n*.

**Rejected, and the reason is decisive rather than stylistic.** ADR 0004's replay is **watermark-based**: on startup Anchor loads a snapshot and replays only lines whose `seq` exceeds the snapshot's watermark. Blocks incorporated into the snapshot are never traversed at all in that run. A traversal counter would therefore count only the lines this particular run happened to walk, producing numbering that depends on *when the last compaction occurred* — so the same log yields different identities before and after compaction.

The general principle, which outlives this specific mechanism: **identity must derive from data carried in the event, not from how a given run happens to traverse the log.** `seq` is carried in the record; traversal position is an artefact of the reader.

## Trade-offs

| | A. Explicit UUID | **B. Derive from `seq`** | C. Content-addressed | D. Replacement state | E. Registry | F. Replay counter |
|---|---|---|---|---|---|---|
| Log format change | Yes — 5 variants, 2 change shape | **None** | None | Yes — new payloads | None in log | None |
| ADR 0004 impact | Amendment required | **None** | None | Undermines its rationale | Reintroduces a 2nd artifact | None |
| Existing history addressable | **No — permanent split** | **Yes, immediately** | n/a | n/a | Only if backfilled | No |
| Survives compaction | Yes | **Yes** | n/a | Yes | Needs own scheme | **No** |
| Stable under mutation | Yes | **Yes** | **No — self-defeating** | n/a | Yes | No |
| Implementation cost | Moderate | **Low** | Low | High | High | Low |
| Correctness risk | Low | **Low** | Fatal | Model-level | Sync/crash divergence | Fatal |

## Decision

**Time Block identity is derived deterministically from the `seq` of the transition that created the block: `id = UUIDv5(ANCHOR_NAMESPACE, seq)`.**

`ANCHOR_NAMESPACE` is a fixed, project-wide UUID constant, chosen once and never changed. **It becomes part of the durable contract** the moment any reconstruction transition references an id.

Why this fits Anchor specifically:

- **It preserves the append-only replay model.** No transition is added, no payload changes, no line is written differently. Identity becomes a *reading* of the log rather than an addition to it — which is what `docs/architecture/constraints.md` already says all state is.
- **It requires no log migration.** There is nothing to convert, no dual-format reader, and no window during which old and new lines behave differently.
- **It makes historical logs immediately addressable.** Every block ever recorded becomes a valid reconstruction target the moment this ships.
- **It avoids splitting history into editable and non-editable generations** — the failure that disqualified option A, and a bad one for a feature that exists to let the user repair their record.
- **It keeps `TimeBlock.id` a `Uuid`.** The field is serialised into full-fidelity JSON export and into `StackView`; a new identifier type would ripple into the export contract and the frontend types for no benefit.

### Architectural invariant

> **Every transition that creates Time Blocks creates exactly one Time Block.**

This is **not** a description of today's implementation that happens to be convenient. It is a load-bearing invariant that this identity scheme depends on: `seq` uniquely identifies a created block only while the mapping from creating transition to block is one-to-one. If a transition ever created two blocks, both would derive the same id, and every reconstruction reference to either would be ambiguous.

The invariant must be **enforced by test**, not assumed from inspection.

**If a future feature requires a transition to create multiple Time Blocks, this ADR must be revisited** — with a new ADR superseding it — rather than the identity scheme being silently extended with an ordinal or similar. A scheme extended in passing is exactly how a contract gets broken without anyone deciding to break it.

Deliberately *not* pre-built: a `(seq, ordinal)` derivation to leave room for that case. Per [`principles.md`](../principles.md) #1, no case for it exists, and building for a hypothetical is what that principle rejects.

## Relationship to existing ADRs

- **[ADR 0004](0004-transition-log-format-and-torn-write-scheme.md) remains the sole authority for the append-only log format** — record shape, checksum framing, torn-write detection, compaction and the watermark. **This ADR changes none of it.** ADR 0004 gets a pointer note, as it did for ADR 0005's snapshot-payload guarantee; its decisions stand unamended.
- **This ADR defines how persistent Time Block identity is *derived from* that log.** Identity is a projection, consistent with `docs/architecture/constraints.md`: the event log is the single source of truth and all state is replayed from it.
- **[ADR 0005](0005-event-model-time-block-metadata-and-reconstruction-transitions.md)** is unaffected. Its three metadata fields, the `DerivedInterruptionStatus` projection, and its snapshot-payload guarantee all hold unchanged. Its open items 1–4 concern reconstruction *semantics*, not identity, and are resolved by `timeline-reconstruction.md` rather than here.
- **`timeline-reconstruction.md` must cite this ADR for persistent identity** and retract its claim that reconstruction imposes *"no new requirement."* It does impose one; this is it.

No accepted ADR is reopened.

## Consequences

**Required implementation work** (not part of this ADR):

- Enable the `uuid` crate's `v5` feature — `Cargo.toml` currently has only `v4` and `serde`.
- Choose and fix `ANCHOR_NAMESPACE`.
- Thread the record's `seq` into block construction at the five `TimeBlock::new` call sites in `stack.rs::apply`. Note `apply` currently receives `timestamp` but **not** `seq`, so its signature changes — the one non-trivial piece.
- Add a test enforcing the one-block-per-transition invariant.

**Replay determinism.** Replaying a log twice now yields byte-identical state including ids. This is a genuine strengthening beyond what this ADR needs: restart-equivalence tests can assert full equality rather than field-by-field comparison with ids excluded.

**Testing implications — one existing test's premise inverts.** `commands.rs`'s restart test currently reads:

> *"Time Block IDs are freshly random per `TimeBlock::new()` call, so replay naturally produces different IDs than the original run — **by design, nothing relies on stable IDs across restarts**"*

That comment becomes false, and the test's field-by-field comparison — which deliberately omits `id` — should be replaced by an assertion that ids **match** across a restart. The test gets stronger, and this ADR is the reason.

**Impact on reconstruction.** Unblocks it. Move, Resize, Edit Identity and Delete gain a target that survives replay. Reconstruction transitions carry a `Uuid` resolved against blocks whose identity is recomputed identically on every pass.

**Impact on export.** `TimeBlock.id` is already serialised into full-fidelity JSON. Today those ids are random and change on every restart, making them meaningless to any consumer. After this change they are **stable and reproducible** — an export of the same range yields the same ids every time. This is an improvement, and it carries an obligation: an exported id becomes a durable external reference, so changing the derivation later would break anything that stored one. Grouped exports are unaffected; they carry no per-block metadata.

**What this does not do.** It does not make ids meaningful to a *user*, does not create a stable identity for a *task* (there is still no task entity — `docs/product/mvp.md`'s flat model is untouched), and does not survive a user deleting and re-adding a block, which correctly produces a different block.

## Review findings resolved by this ADR

- **M1** (independent review, 2026-07-29) — *"Block identity is not stable across replay. Four of the five operations are unimplementable as specified."* Resolved in principle here; `timeline-reconstruction.md` must be updated to cite this ADR and retract its "no new requirement" claim.
- **M4**, partially — reconstruction claimed to both *resolve* ADR 0005 items 1–4 and be *gated on* them. Separating identity into its own ADR removes the ambiguity: this ADR owns identity, `timeline-reconstruction.md` owns items 1–4, and the two no longer contend.

Other review findings are untouched and remain open.

---

**Revisit this decision (new ADR) if:** a transition ever needs to create more than one Time Block; or an external consumer comes to depend on exported ids in a way that makes the derivation function costly to keep; or compaction is ever redesigned such that `seq` is no longer stable for the lifetime of a block.
