---
status: accepted
date: 2026-08-01
owner: erich
related: [docs/decisions/0004-transition-log-format-and-torn-write-scheme.md, docs/decisions/0005-event-model-time-block-metadata-and-reconstruction-transitions.md, docs/product/features/timeline-reconstruction.md, docs/product/features/interruption-stack.md, docs/product/features/export.md, docs/architecture/constraints.md, docs/principles.md, docs/risks.md, docs/assumptions.md, docs/glossary.md]
---

# 0006: Stable, Persistent Time Block Identity

> **`status: accepted` 2026-08-01**, after two independent review cycles (summarised below) and a third that reviewed the feature consuming it. This was the architectural prerequisite for timeline reconstruction (#15); that feature is accepted alongside it.
>
> **What acceptance commits to.** `ANCHOR_NAMESPACE` and the ASCII-decimal encoding of `seq` are now a durable contract, not a changeable implementation detail — `timeline-reconstruction.md`'s alternative F decided that reconstruction payloads carry the derived `Uuid`, so the first such transition ever written freezes both. Changing either afterwards would orphan every stored reference and leave the app unable to start. **Per this repository's ADR convention, a reversal is a new ADR superseding this one — never an edit here.**
>
> **The one prerequisite this ADR named has since been met.** Risk **R14** — a `seq` consumed by an append that did not fully and durably complete — was closed in `log/writer.rs` on 2026-08-02: `append` now records the file length before writing and restores it on any failure, so a failed append consumes no `seq` and leaves no bytes. That was the last thing gating implementation of this scheme.
>
> **Revised 2026-07-29 after its own independent review.** The decision — derive identity from the creating transition's `seq` — was upheld; three things were not. The review's strongest objection was that a fixed namespace over `seq` gives no *global* uniqueness. Investigating that against accepted project material established that **global uniqueness is not a requirement of this project at all**: `export.md` does not specify an `id` field, the billing path never carries one, multi-user and sync are explicitly out of scope, and no ADR states an identity requirement. The objection was therefore **reframed, not accepted** — the defect was this document overclaiming in Consequences, not the derivation. See the Decision's non-goal, and option G.
>
> **Revised again 2026-07-29 after a second independent review.** That review found the derivation sound and proposed no alternative, but caught that the Decision named the namespace and never the **encoding** — and since UUIDv5 hashes bytes, `seq.to_string()`, `to_be_bytes()` and `to_le_bytes()` give three different ids from the same number. The encoding was as load-bearing as the namespace and was being left to implementation; it is now fixed in the Decision and pinned by test vectors. The same pass recorded that this ADR does *not* decide whether reconstruction payloads carry the derived UUID or the `seq`, and broadened the open writer defect (**R14**) to cover a partial `write_all`, not only a failed `sync_all`.
>
> Two genuine defects were fixed: `seq` uniqueness was asserted from a *document* rather than the code and **does not hold today** (see Verified facts), and the claim that compaction was "unaffected" was wrong — the snapshot must persist ids (see Relationship to existing ADRs).

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
- `seq` is assigned per line, including heartbeats, and is intended to be monotonic ([ADR 0004](0004-transition-log-format-and-torn-write-scheme.md)).

**`seq` uniqueness does not hold today, and this decision depends on it.** An earlier draft listed monotonic `seq` as a verified fact citing ADR 0004 — that is a *document*, not the code, which is exactly the failure [`principles.md`](../principles.md) #8 names and risk **R11** tracks. Checked against `log/writer.rs`, two reuse paths exist:

1. **A `seq` consumed by an append that did not fully and durably complete.** `append` does `write_all` → `sync_all` → `next_seq += 1`, so `next_seq` advances only if *both* succeed, while bytes may already have reached disk either way. A failed `sync_all` leaves a complete, checksum-valid line for that `seq` on disk; a failed or *partial* `write_all` leaves an unterminated fragment — and because the tail truncation above runs at `open`, a mid-session fragment is not repaired before the next append concatenates onto it. `apply_transition` surfaces the error as a string and the app continues with the same writer either way.
2. ~~**The discarded torn tail is never truncated.**~~ **Fixed 2026-07-29** — [`78096a8`](https://github.com/DeLicious777/Anchor/commit/78096a8), issue #18, risk **R5**. `LogWriter::open` now truncates any bytes after the final record boundary, so an append always starts on a fresh line.

   Investigation established this was **not** a `seq`-reuse path at all, and was worse than described: the concatenated line broke replay, so every transition committed after a torn write was permanently lost and `next_seq` stalled — meaning successive *sessions* reissued the same `seq`, which under this ADR's scheme would have given three different pieces of work the same id. That is now closed and regression-tested; `seq` advances normally across a torn write.

**Path 1 was closed on 2026-08-02** — `append` records the file length before writing and rolls back to it on any failure of `write_all` or `sync_all`, so a `seq` is consumed only by an append that durably completed. Tracked as risk **R14**, now `mitigated`. It was a prerequisite of implementing this ADR, and it is met. `seq` uniqueness is already required by ADR 0004's watermark filtering — two lines sharing a `seq` make that filter undefined regardless of identity — so this belongs in the writer, where the invariant lives, and must not be compensated for in the derivation. See Options Considered, G.

**Neither path is a bug this ADR creates.** What it changes is the *consequence*: under `Uuid::new_v4()` uniqueness was free, so a duplicate `seq` was an obscure I/O edge; under any derivation it becomes silent identity corruption, because `stack.rs`'s `resolve_paused` and `derived_status` both use `id` as a lookup key.

## Options Considered

### A. Persist an explicit UUID in every creating transition

The writer generates a UUID and puts it in the payload; replay uses the carried value instead of generating one.

- **Replay behaviour**: correct and stable, for blocks created after the change.
- **Wire-format impact**: five variants change. Two of them — `ReturnPrevious` and `ReturnOriginal` — are currently **unit variants**, serialised as `{"type":"return-previous"}`. Adding a field converts them to struct variants. That is a change of shape, not just an added field.
- **ADR 0004 compatibility**: this genuinely modifies the on-disk contract ADR 0004 declares stable. It is achievable — `Rename` was added additively — but it is a real change requiring an amendment.
- **Backward compatibility**: workable via `#[serde(default)]`, deserialising old lines with no id.
- **Effect on existing history**: blocks created by pre-change log lines carry no id and would fall back to a generated one, leaving them unaddressable — history split into an editable generation and a non-editable one, at an arbitrary date, in a feature whose purpose is repairing the record.

**Rejected** — but on cost, not impossibility. That split is not *forced* by this option: A and B compose, carrying an explicit id going forward and falling back to `seq` derivation for older lines. The honest reason to reject A is that the composition is strictly more machinery than B alone, for no additional guarantee the project requires — two identity sources to keep consistent, plus a wire-format change to five variants, two of which change shape.

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

### G. Derive from the record rather than `seq` alone — `UUIDv5(namespace, seq ‖ timestamp)`

Proposed by the independent review as a fix for cross-lineage collision. Both fields already exist in `TransitionRecord`, so it costs no format change and the same five call sites.

**Rejected — it is technically correct but architecturally unnecessary, and it would mask a bug rather than fix one.** Three reasons, in order of weight:

1. **It solves a problem the project does not have.** Its benefit is global uniqueness, which the Decision above establishes is not a requirement — no consumer of exported ids exists, and `export.md` does not even specify the field.
2. **It would hide `seq` reuse instead of fixing it.** Two lines sharing a `seq` would receive distinct ids, so identity would appear correct while ADR 0004's watermark filter remained undefined for those lines. Compensating for one subsystem's broken invariant inside another subsystem makes the real defect *harder to detect*, and leaves it in place. The correct fix is the writer contract.
3. **It broadens the contract without a demonstrated requirement**, making identity depend on timestamp *serialisation* stability — formatting, precision, timezone rendering — which is a genuine new dependency traded for a hypothetical benefit. [`principles.md`](../principles.md) #1.

The cost of rejecting it: no global uniqueness, and the loss of an incidental guard against the writer bug. Both are accepted knowingly, the second because the guard would be papering over something that must be fixed regardless.

**Also considered and rejected: a per-lineage namespace** — mint a random namespace UUID when a log is created and store it in a log header. That would give global uniqueness *and* `seq` derivation, but requires a log-format change to satisfy a non-requirement.

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

```
ANCHOR_NAMESPACE = becbca30-958b-4568-a9ec-dd5ed1dbf612
```

**The name hashed under that namespace is the ASCII decimal representation of `seq`** — unpadded, unprefixed, no separators. In Rust: `Uuid::new_v5(&ANCHOR_NAMESPACE, seq.to_string().as_bytes())`.

Stating the encoding is not pedantry. `seq` is a `u64` and UUIDv5 hashes a *byte string*, so `b"42"`, `42u64.to_be_bytes()` and `42u64.to_le_bytes()` yield three different UUIDs — the encoding is exactly as load-bearing as the namespace, and exactly as unchangeable once a reference is stored. Decimal is chosen over a byte representation because it matches how `seq` already appears in the log's JSON, so an id can be reproduced by hand from a log line. **Pinned by test vectors, so the contract is held by an assertion rather than by prose:**

| `seq` | `id` |
|---|---|
| `0` | `18885148-20dd-5b7c-a7d3-d7844af7a220` |
| `1` | `24d50f72-3754-5b11-8e57-ec5329a394af` |
| `42` | `7d8aafb1-dfaf-59a2-bab1-d1a0a0f3e7f7` |

Both the namespace and the encoding are fixed here rather than deferred to implementation, because values that must never change belong in the accepted decision.

**What makes them a durable contract is a decision this ADR does not make.** They are permanent only if reconstruction payloads carry the derived *UUID*. If a payload carried the *`seq`* instead, the derivation would stay purely in-memory and remain changeable, and the log would stay readable by eye — one of the two axes ADR 0004 chose JSONL on. This ADR assumes the UUID because `TimeBlock.id` already is one and the frontend already types it as such, but that is a consequence of an existing field, not an argument. **The choice belongs to whichever design specifies the reconstruction payloads**, and it should be made deliberately there rather than inherited from here.

**It has since been made there** — [`timeline-reconstruction.md`](../product/features/timeline-reconstruction.md), alternative F: payloads carry the derived `Uuid`. So the namespace and encoding above *are* a permanent on-disk contract from the first reconstruction transition written, and that doc records the loss of future freedom as a knowing cost rather than inheriting it silently. Noted here rather than argued here: the reasoning belongs to the design that owns the payloads.

### Scope of the guarantee — and an explicit non-goal

**A Time Block's identity is stable within a single append-only log lineage. This ADR does not provide globally unique identities across unrelated Anchor histories. Global uniqueness is explicitly a non-goal** unless a future accepted requirement establishes one.

Stated in the Decision rather than buried in Consequences, so that anyone revisiting this — while implementing sync, import, or multi-device support — sees immediately that the limitation is **intentional, not accidental**.

Concretely: a fresh log, a restored backup, or a second installation will reuse the same ids for entirely unrelated work. The first block in every Anchor log is the same UUID.

**Why that is acceptable, established from accepted project material rather than assumed:**

- **`docs/product/features/export.md` never specifies an `id` field.** Its documented full-fidelity JSON shape is name, project, client, start, end, duration, plus the three metadata fields. The id reaches the output only because `TimeBlock` derives `Serialize` — it is unspecified output, not a contract.
- **The billing path never carries an id at all.** XLSX columns are Name / Project / Client / Duration. [ADR 0003](0003-billable-classification-out-of-scope.md) puts classification in a downstream process fed by those columns.
- **Multi-user accounts and sync are explicitly out of scope** (`docs/product/mvp.md`, `docs/product/users.md`). `docs/vision/vision.md` raises moving beyond personal use as an *open question*, not a requirement.
- **No accepted ADR states an identity or uniqueness requirement.**
- **Import is future work** — `docs/architecture/constraints.md` says "any *future* import."

A globally unique scheme was considered and rejected as solving a problem the project does not have: it would trade a real new dependency (identity resting on timestamp *serialisation* stability, or a log-format change for a per-lineage namespace) for a benefit no accepted requirement asks for. See Options Considered, G.

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

**The most plausible violator is import, and it is already an accepted commitment, not a hypothetical.** `docs/architecture/constraints.md` states that every input path produces transitions and nothing else, naming "any future import" explicitly. A batch import creating N blocks in one transition would break this invariant directly. It is out of scope today; it is the first thing to check against this ADR when it arrives.

**Verified against reconstruction as designed:** the invariant holds. Add creates one block; Move, Resize, Edit Identity and Delete create none; the overlap policy is clamping, not splitting.

**One open tension, recorded rather than resolved.** `timeline-reconstruction.md` makes undo a hard prerequisite for unconfirmed Delete. Undoing a Delete forces a choice this ADR does not make: either the undo transition re-creates the block carrying its **original** id — which breaks "identity derives from the creating transition's `seq`" as a universal rule and makes the scheme a hybrid — or it creates a block with a **new** id and must carry the deleted block's full prior state, which is the replacement-state shape option D was rejected for. Both are arguable; neither is free. **This must be settled when undo is designed (#14), and it may require revisiting this ADR.**

Deliberately *not* pre-built: a `(seq, ordinal)` derivation to leave room for that case. Per [`principles.md`](../principles.md) #1, no case for it exists, and building for a hypothetical is what that principle rejects.

## Relationship to existing ADRs

- **[ADR 0004](0004-transition-log-format-and-torn-write-scheme.md) remains the sole authority for the append-only log format** — record shape, checksum framing, torn-write detection, compaction and the watermark. **This ADR changes none of its decisions**, and adds no field to any record.

  **It does add one normative requirement to the snapshot, and an earlier draft wrongly claimed compaction was "unaffected."** Blocks below the watermark are never replayed — `log/reader.rs` skips those lines entirely — so their identity can come only from the snapshot. Therefore:

  > **The snapshot MUST persist each Time Block's `id`** (or the `seq` it derives from) exactly, alongside the unresolved stack frames ADR 0005 already requires.

  This is the *same gap* ADR 0005 had to close as assumption **A10**: ADR 0004 specified compaction's mechanism and never its payload, and a decision was built on the unstated assumption. One ADR later, in the same document, in the same shape. It is [`principles.md`](../principles.md) #4 verbatim — *only materialised state survives* — and it costs nothing to state now, because compaction is unimplemented. Discovering it after a snapshot format ships costs a migration.
- **This ADR defines how persistent Time Block identity is *derived from* that log.** Identity is a projection, consistent with `docs/architecture/constraints.md`: the event log is the single source of truth and all state is replayed from it.
- **[ADR 0005](0005-event-model-time-block-metadata-and-reconstruction-transitions.md)** is unaffected. Its three metadata fields, the `DerivedInterruptionStatus` projection, and its snapshot-payload guarantee all hold unchanged. Its open items 1–4 concern reconstruction *semantics*, not identity, and are resolved by `timeline-reconstruction.md` rather than here.
- **`timeline-reconstruction.md` must cite this ADR for persistent identity** and retract its claim that reconstruction imposes *"no new requirement."* It does impose one; this is it.

No accepted ADR is reopened.

## Consequences

**Required implementation work** (not part of this ADR):

- Enable the `uuid` crate's `v5` feature — `Cargo.toml` currently has only `v4` and `serde`.
- Choose and fix `ANCHOR_NAMESPACE`.
- **Make `seq` allocation atomic with durable append** in `log/writer.rs` — a `seq` must not be consumed by a write that did not durably complete. **A prerequisite, not a follow-up**: already required by ADR 0004's watermark filtering, and this ADR turns its absence into silent identity corruption. Tracked as risk **R14**. *(The torn-tail half of this is already done — see Verified facts.)*
- Thread the record's `seq` into block construction at the five `TimeBlock::new` call sites in `stack.rs::apply`. Note `apply` currently receives `timestamp` but **not** `seq`, so its signature changes.
- **`apply_transition` dry-runs on a clone before any record exists**, so it must be handed the writer's `next_seq`; the dry-run and the real apply must agree on it, and a rejected or failed append must not consume it. Same seam as the reuse paths above — this is the genuinely non-trivial piece, not the signature change.
- Add tests enforcing **both** the one-block-per-transition invariant and **id uniqueness across a full replay**. The second matters because `resolve_paused` and `derived_status` use `id` as a lookup key: their correctness was previously guaranteed by v4 randomness and is now a consequence of the derivation, so a duplicate resolves the wrong block silently rather than failing.

**Replay determinism.** Replaying a log twice now yields byte-identical state including ids. This is a genuine strengthening beyond what this ADR needs: restart-equivalence tests can assert full equality rather than field-by-field comparison with ids excluded.

**Testing implications — one existing test's premise inverts.** `commands.rs`'s restart test currently reads:

> *"Time Block IDs are freshly random per `TimeBlock::new()` call, so replay naturally produces different IDs than the original run — **by design, nothing relies on stable IDs across restarts**"*

That comment becomes false, and the test's field-by-field comparison — which deliberately omits `id` — should be replaced by an assertion that ids **match** across a restart. The test gets stronger, and this ADR is the reason.

**Impact on reconstruction.** Unblocks it. Move, Resize, Edit Identity and Delete gain a target that survives replay. Reconstruction transitions carry a `Uuid` resolved against blocks whose identity is recomputed identically on every pass.

**Impact on export.** `TimeBlock.id` is already serialised into full-fidelity JSON — incidentally, because `TimeBlock` derives `Serialize`; `export.md` does not specify the field. Today those values are random and change on every restart, making them meaningless. After this change they become **reproducible within one log lineage**: exporting the same range twice from the same log yields the same ids.

**They do not become globally durable references.** A fresh log, a restored backup, or a second installation reuses the same ids for unrelated work — see the Decision's non-goal. That is acceptable only because no consumer depends on them, and the billing path never receives one. **If any consumer ever comes to depend on exported ids, this ADR must be revisited**, because the guarantee it provides is narrower than such a consumer would need.

Grouped exports are unaffected; they carry no per-block metadata.

**What this does not do.** It does not make ids meaningful to a *user*, does not create a stable identity for a *task* (there is still no task entity — `docs/product/mvp.md`'s flat model is untouched), and does not survive a user deleting and re-adding a block, which correctly produces a different block.

## Review findings resolved by this ADR

- **M1** (independent review, 2026-07-29) — *"Block identity is not stable across replay. Four of the five operations are unimplementable as specified."* Resolved in principle here; `timeline-reconstruction.md` must be updated to cite this ADR and retract its "no new requirement" claim.
- **M4**, partially — reconstruction claimed to both *resolve* ADR 0005 items 1–4 and be *gated on* them. Separating identity into its own ADR removes the ambiguity: this ADR owns identity, `timeline-reconstruction.md` owns items 1–4, and the two no longer contend.

Other review findings are untouched and remain open.

---

**Revisit this decision (new ADR) if:**

- a transition ever needs to create more than one Time Block — **import is the most likely trigger**, and it is already an accepted commitment in `docs/architecture/constraints.md`;
- **any external consumer comes to depend on exported ids**, or cross-lineage identity becomes an accepted requirement — sync, multi-device, or import across installations would each do this, and the Decision's non-goal would no longer hold;
- undo of a Delete is designed (#14) and requires re-creating a block under its original id;
- or compaction is redesigned such that `seq` is no longer stable for the lifetime of a block.
