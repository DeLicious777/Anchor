---
status: accepted
date: 2026-07-24
owner: erich
related: [docs/product/features/interruption-stack.md, docs/decisions/0002-desktop-app-framework-and-platform.md, docs/risks.md, docs/architecture/constraints.md, docs/glossary.md]
---

# 0004: Transition Log Format, Torn-Write Detection, and Compaction

## Context

`docs/product/features/interruption-stack.md` requires an append-only, crash-atomic transition log (every Switch/Interrupt/Return/Complete, plus a 60-second heartbeat while an entry is active), where a torn write can only ever affect the last record ("detected — fails to parse/checksum — and discarded on next launch"), and periodic compaction for fast startup. [ADR 0002](0002-desktop-app-framework-and-platform.md) chose the append-only-log approach over alternatives and explicitly deferred "exact on-disk log format" and the torn-write framing/checksum scheme as smaller, more reversible implementation choices — this ADR makes those choices. ADR 0002 also flagged this log format as something that "should be treated as a stable on-disk contract once shipped," which is why it gets an ADR rather than being decided silently in code.

Relevant constraints already in force: Rust core (`docs/architecture/constraints.md`, ADR 0002), low memory footprint and startup speed as top priorities, a solo maintainer without confirmed prior Rust experience (risk R6, `docs/risks.md`).

## Options Considered

**Record encoding:**
1. **JSON Lines (JSONL)** — one JSON object per transition/heartbeat, newline-terminated.
2. Length-prefixed binary framing — each record as `[4-byte length][payload][4-byte CRC32]`.
3. Embedded SQL database (e.g. SQLite via a Rust crate) as the log store instead of a flat file.

**Torn-write detection:**
1. Parse-validity only — a line that fails to parse as JSON, or has no trailing newline, is the torn tail; discard it.
2. Parse-validity plus a per-line checksum (e.g. CRC32) embedded in each JSON record — catches the rarer case of a torn write that happens to leave a syntactically-valid-but-truncated JSON value.

**Compaction trigger:**
1. Never compact — always replay the full history from the start of the log.
2. Compact on clean shutdown only (a snapshot of current state is written, then the log is truncated to empty).
3. Compact on clean shutdown **or** after N transitions accumulate since the last compaction, whichever comes first — bounds log growth even across a single very long-running session that's never cleanly closed.

## Trade-offs

**Record encoding:**

| | JSON Lines (1) | Binary framing (2) | SQLite (3) |
|---|---|---|---|
| Complexity for a Rust-newcomer solo maintainer | Low — a JSON serde crate plus line splitting, both extremely well-trodden in Rust | Higher — hand-rolled binary framing is exactly the kind of subtle-bug-prone code a newcomer is more likely to get wrong (off-by-one on length prefixes, endianness, etc.) | Low to use, but re-opens a decision ADR 0002 already made (rejected "full transactional database per event" as more machinery than a single-user local app needs) without new justification |
| Debuggability | High — a human can open the log file directly and read what happened | Low — requires a custom tool or debugger to inspect | Medium — requires a SQLite browser, but structured querying is easy once opened |
| Torn-write behavior | A torn write leaves an incomplete trailing line, straightforward to detect on parse | A torn write can leave a partial length prefix or partial payload — also detectable, but with more edge cases to get right by hand | Handled by SQLite's own transaction/journal machinery — arguably the most "solved" option here, but at the cost of re-opening the ADR 0002 decision |
| Size/performance | Slightly larger on disk than binary, slightly slower to parse — irrelevant at this app's actual record volume (a personal time tracker, at most a few thousand records) | Most compact/fastest — an advantage this app has no real need for | Comparable to JSONL in practice at this scale |
| Consistency with ADR 0002 | Directly extends the already-chosen append-only-log design | Also extends it, just with a different encoding | Contradicts the "no transactional database" reasoning in ADR 0002 without a new reason to revisit it |

**Torn-write detection:**

| | Parse-validity only (1) | Parse-validity + checksum (2) |
|---|---|---|
| Coverage | Misses the rare case where a torn write happens to leave a syntactically-valid-but-truncated JSON value (e.g. a crash right after closing a nested object but before the line's remaining fields) | Catches that case too — a checksum mismatch flags corruption even when the truncated content still happens to parse |
| Complexity | Lower — no checksum computation/verification code needed | Slightly higher, and only viable if the framing avoids a self-reference problem (see Consequences: checksum is computed over bytes outside the JSON object itself, not a field inside it) |
| Fit given risk R6 | Simpler, safer for a Rust newcomer | Still low-risk *if* implemented via the append-outside-the-object framing below — the naive "checksum as a JSON field" approach would reintroduce exactly the hand-rolled-framing risk this ADR rejects binary encoding for |

**Chosen: parse-validity + checksum**, specifically because the coverage gap in option 1 is a real, if rare, data-integrity gap for a project whose whole premise is trustworthy record-keeping — but only using the framing described in Consequences that avoids the self-reference problem a first-pass review of this ADR identified.

**Compaction trigger:**

| | Never (1) | Clean shutdown only (2) | Clean shutdown OR N transitions (3) |
|---|---|---|---|
| Startup cost over time | Grows unboundedly with total lifetime usage — directly conflicts with ADR 0002's startup-speed priority | Bounded across restarts, but a single very long-running session (app left open for days/weeks) never compacts | Bounded regardless of session length |
| Complexity | None | Low — one compaction path | Low — one extra counter, same compaction path reused |
| Risk if wrong | Startup eventually becomes noticeably slow — not measured, but this is the failure mode ADR 0002 was trying to avoid entirely | Fine for typical usage (the app is closed most nights), fails only for the atypical long-session case | See below re: the N value not being measured either |

**Chosen: option 3.** The **N = 500** threshold itself is **not measured, chosen as a low-cost default** — the same honest framing `docs/product/features/interruption-stack.md` already used for its 60-second heartbeat interval ("chosen as a low-cost default, not measured"). The counting rule is explicit to avoid the ambiguity a first-pass review flagged: **only user-triggered lifecycle transitions (start/switch/interrupt/return-previous/return-original/complete) count toward N — heartbeat lines do not.** Without this exclusion, a single heads-down 8-hour day with zero manual transitions would still accumulate ~480 heartbeat lines and could trigger compaction from heartbeat volume alone, which isn't what this trigger is for.

## Decision

**JSON Lines (JSONL), with parse-validity plus a checksum (framed outside the JSON object, not as a field within it) for torn-write detection, and compaction on clean shutdown or after 500 user-triggered transitions (excluding heartbeats) since the last compaction, whichever comes first.**

JSON Lines wins primarily on the Rust-newcomer-complexity axis (risk R6) and debuggability. SQLite is rejected for reopening ADR 0002's already-settled rejection of a transactional database without new justification. The checksum addition and its framing are detailed in Consequences, specifically to avoid the self-reference bug a first-pass review of this ADR caught.

**Revisit this decision (new ADR) if:** measured cold-start time (snapshot load + log replay) exceeds 300ms in practice on the author's own hardware, or if real usage shows the 500-transition threshold needs adjusting (e.g. compaction firing far more or less often than once every few days of typical use).

## Consequences

- **Line format avoids the checksum self-reference problem**: each line is `<JSON object><TAB><CRC32 checksum in hex><newline>`, where the checksum is computed over the exact bytes of the JSON object substring — *not* a field inside that object. On read, a line is split on the last tab; the JSON substring's bytes are checksummed and compared against the trailing hex value *before* attempting to parse it as JSON. This means detection never requires serializing the object twice or manually excluding a field from its own checksum — the checksum lives entirely outside the structure it protects.
- Each JSON object has a `type` field (transition kind: start/switch/interrupt/return-previous/return-original/complete/heartbeat), the relevant Time Block or stack data, a timestamp, and a monotonically increasing **sequence number — every line gets one, including heartbeats.** This is a distinct counter from the "500 user-triggered transitions" compaction-trigger count below: the sequence number is per-line and exists purely for watermark/replay filtering; the compaction-trigger count only increments on lifecycle transitions and exists purely to decide when to compact. Conflating the two was an earlier draft's mistake — heartbeats need a sequence number to be filterable at all, even though they don't count toward triggering compaction.
- **Snapshot writes use write-to-temp-file-then-atomic-rename** — the exact technique `docs/product/features/interruption-stack.md` (line 38) considered and rejected for *per-transition* writes due to overhead, but that overhead concern doesn't apply here since compaction is rare (every few hundred transitions or once per session), not per-action.
- **Crash-safety through compaction is watermark-based, not truncation-order-dependent**: every snapshot records the sequence number of the last *line* (of any type, including a heartbeat if that happened to be the most recent line) it incorporates. On startup, Anchor loads the snapshot (if any) plus all log lines, and replays only log lines whose sequence number is greater than the snapshot's recorded watermark — regardless of whether the log was actually truncated after that snapshot was written. Because heartbeats carry a sequence number like everything else, a heartbeat written after the last compaction is correctly included on replay (preserving the ~60-second `recovered-gap` accuracy bound even across a compaction), while heartbeats before the watermark are correctly excluded (superseded by the snapshot's own recorded last-known-alive value for the active entry, if any). This means a crash at *any* point in the compaction process (during snapshot write, after snapshot write but before truncation, or during truncation) is safe: at worst, the next startup re-verifies some already-incorporated lines against the watermark and correctly skips them. This closes the new failure mode a first-pass review identified (a torn or partial snapshot combined with an already-truncated log) — see `docs/risks.md` R7.
- On launch, Anchor reads the most recent snapshot (if any) plus every log line with a sequence number past its watermark, verifying each line's checksum and JSON validity; the first line that fails either check is treated as the torn tail (per `docs/product/features/interruption-stack.md`) and everything from that point on is discarded — every prior line is unaffected.
- Log truncation after a successful, durable snapshot write is a disk-space optimization, not a correctness requirement — the watermark makes replay correct even if truncation is skipped or interrupted.
- Because JSON string values are escaped by any compliant serializer (including Rust's `serde_json`), a literal raw tab or newline byte cannot appear inside the JSON substring's bytes — the tab-delimited checksum framing above is safe to split on "the last tab" without ambiguity, provided the implementation uses a standard JSON serializer and doesn't hand-construct lines in a way that could bypass this escaping.
- The on-disk JSONL schema (the `type` field, per-type payload shape, and the checksum framing above) is now the "stable on-disk contract" ADR 0002 anticipated — changing it later (e.g. adding a field) needs to remain backward-compatible with logs written under this version, or needs its own migration approach, not a silent format change.
- This does not change anything already decided in `docs/product/features/interruption-stack.md` or ADR 0002/0003 — it only fills in the implementation detail those decisions explicitly deferred.
