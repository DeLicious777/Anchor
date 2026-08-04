# Release Candidate Verification Checklist

A manual end-to-end pass over the **running application**. Not a substitute for the automated suite — a complement to it.

## Why this exists

The test suite is strong (147 tests) and covers each layer well. But almost every serious defect this project has found was **not** a logic bug inside a layer. It was an assumption **between** layers, and every one of them was found by someone exercising the thing rather than by a test:

| Defect | Where it hid |
|---|---|
| Silent data loss after a torn write (**R5**) | writer ↔ replay |
| A `seq` consumed by an append that never completed (**R14**) | writer ↔ identity |
| Gap recovery losing its bound after compaction (**A15**) | snapshot ↔ startup |
| ADR 0006 accepted but never implemented | document ↔ code |
| Theme persistence naming a `settings.json` path that does not exist | document ↔ code |
| The dashboard described as having gap correction it never had | document ↔ code |
| The page scrolling sideways at the app's own default window size | domain ↔ UI |

That is `docs/principles.md` **#8** and risk **R11** in practice: *verify a claim against documentation **and** implementation before it becomes load-bearing.* Automated tests assert what we thought to assert. This pass exists to surface what we did not.

**How to use it.** Work top to bottom in one sitting, on a **fresh data directory**, without restarting except where told. Each step names the **seam** it exercises and the **failure signature** to watch for — so a wrong result is interpretable rather than just "that looked odd."

**Record the outcome**, including a clean run. "All 14 steps passed on <date>, build <sha>" is itself evidence, and it is the thing a future regression gets compared against.

## Before starting

- Build and run the real desktop app (`npm run tauri dev` from `app/`), **not** the dev server in a browser — the browser has no backend and no persistence.
- Locate the data directory (Tauri's `app_data_dir`; on Windows, typically `%APPDATA%/<bundle-id>/`). It should contain `transitions.jsonl`, and after step 12, `snapshot.json`.
- **Start from empty.** Move any existing data directory aside rather than deleting it.

---

## The pass

### Capture

**1. Start a task, let it run a minute or two.**
Seam: hotkey/UI → command → log → projection.
Watch: the widget and dashboard agree immediately; the elapsed timer advances; `transitions.jsonl` gains one line you can read.

**2. Switch to a second task.**
Watch: the first block closes with an **exact** end (`End source` = `exact`), not an inferred one.

**3. Interrupt into a third task, then Return to Previous.**
Seam: stack semantics → derived projection.
Watch: while interrupted, the paused block shows `pending` — *not* `never-interrupted`. Both are an absent `InterruptionOutcome`, and confusing them is exactly what `DerivedInterruptionStatus` exists to prevent. After returning, it reads `resumed`.

**4. Leave a task running and kill the app ungracefully** (Task Manager, not the close button). Relaunch.
Seam: replay → gap recovery → **A15**.
Watch: the leftover block is closed with `End source` = **inferred**, and its end is roughly when you killed it — **not** the moment you relaunched. An end equal to relaunch time means the inference bound was lost, which silently bills every minute the app was closed (**R4** at maximum severity).
Watch: **the task restarts by itself**, as a new block starting at relaunch — the gap must remain a visible hole, never counted as work ([ADR 0007](decisions/0007-auto-resume-after-a-short-gap.md)).

**4b. Repeat, but wait more than an hour before relaunching** (or leave it overnight).
Watch: the entry is closed exactly as above and **nothing restarts** — beyond an hour Anchor stops guessing that you came back to the same task.

**4c. Repeat, relaunching within a few seconds.**
Watch: **no gap event at all** — the task is simply still running, with no closed block and no inferred end. A zero-duration block here means the continuity threshold is not being applied on this path, which is the defect ADR 0007 fixed.

### Reconstruction

**5. Edit Identity on a finished block.** Change name, project and client.
Watch: `Capture` moves to an `*-adjusted` variant but keeps its origin — a `live-capture` block becomes `live-capture-adjusted`, never `manual-entry`. Start, end and interruption status are unchanged.

**6. Edit Identity on a block whose interruption is still unresolved**, then return to it.
Seam: block ↔ stack frame atomicity.
Watch: the resumed task carries the **corrected** identity. A stale name here means the frame's copy desynced, and no later transition would ever fix it.

**7. Try to Delete that same unresolved block.**
Watch: it is **rejected**, with the backend's own wording. The block stays, the frame stays, and the app keeps working. This is what stops replay from later failing with `PausedBlockNotFound`.

**8. Delete a resolved block. Read the confirmation, then cancel.**
Watch: the confirmation names the block, says there is no undo, and is **fully readable without scrolling sideways** at the default window size. Cancelling changes nothing — and `transitions.jsonl` gains **no** line.

**9. Delete it again and confirm.**
Watch: the row disappears; neighbours keep their times; a gap is left rather than closed.

**10. Start a new unnamed task.**
Seam: deletion ↔ name allocation (**#19**, **R8**).
Watch: it does **not** reuse a number already used today — including the number belonging to the block you just deleted, or one you renamed away from.

### Export

**11. Export XLSX and JSON, and open both.**
Seam: projection → export → billing artifact.
Watch: the deleted block is **absent from both**. Edited names appear in their corrected form. Durations match what the History View shows. Full-fidelity JSON is ordered by **start**, not by when blocks were closed.
Watch: exporting writes nothing — `transitions.jsonl` is no longer than before, apart from any heartbeat that legitimately landed during it.

### Persistence

**12. Close the app with the close button** (a clean shutdown — this is what triggers compaction).
*Closing **either** window exits Anchor as of 2026-08-04 (#22). Before that the widget kept the process alive, so closing the dashboard looked like a clean shutdown and silently was not — which is why runs 1 and 2 both failed to produce a snapshot here.*
Watch: `snapshot.json` appears, and `transitions.jsonl` is now **empty**. If the snapshot is missing while the log is empty, stop: that is total data loss and the ordering guarantee has been violated.

**13. Relaunch.**
Seam: snapshot → replay → identical projection.
Watch: the History View is **exactly** as you left it — same blocks, same names, same times, same capture origins, same interruption statuses. Everything below the watermark now comes from the snapshot rather than the log.

**14. Edit Identity on a block that existed before the compaction, then restart once more.**
Seam: **A14** — identity surviving a truncated log.
Watch: the edit resolves against the right block and survives. This is the one step that proves a block below the watermark is still addressable; if identity came only from the log, there would be nothing left to name.

---

## The evidence bundle

**Capture this before starting, not after.** Two reasons: the commit must be the one you actually ran (not whatever `HEAD` is by the time you write it up), and in a 🔴 the operator is mid-incident and will not reliably remember to collect environment facts. Filling the header in first costs ten seconds and makes every run comparable to every other.

Run this from the repo root — it emits the header ready to paste:

```powershell
$tz = Get-TimeZone; "commit:   $(git rev-parse --short HEAD)"; "branch:   $(git rev-parse --abbrev-ref HEAD)"; "os:       $((Get-CimInstance Win32_OperatingSystem).Caption) $([System.Environment]::OSVersion.Version)"; "timezone: $($tz.Id) (UTC$(if($tz.BaseUtcOffset.TotalHours -ge 0){'+'})$($tz.BaseUtcOffset.TotalHours)), DST now: $($tz.IsDaylightSavingTime([DateTime]::Now))"; "started:  $(Get-Date -Format 'yyyy-MM-dd HH:mm K')"; "locale:   $((Get-Culture).Name)"
```

A bundle is:

| Item | When | Why it earns its place |
|---|---|---|
| Commit SHA + branch | always | Makes a later failure a regression against a known point rather than an open question |
| OS and version | always | |
| **Timezone, current DST state, locale** | always | Not metadata for completeness. `next_default_name` resets on *local* midnight, gap recovery compares instants across a shutdown, and the widget is sized against non-English string lengths. A regression that only reproduces near a DST boundary or in a non-English locale is diagnosable only if this was recorded |
| Date and time the run started | always | Pins the run against the DST/midnight boundaries above |
| Outcome 🟢 / 🟡 / 🔴 | always | |
| The generated XLSX and JSON from step 11 | always | The product's primary artifact. Keeping it lets a future export be diffed against a known-good one rather than eyeballed |
| **Archived data directory** | 🟡 and 🔴 | The only copy of the state that produced the result. See the Abort steps — do not relaunch first |

Keep bundles wherever suits; they are deliberately not committed, since they contain real captured work.

## Concluding the pass

Every run ends in exactly one of three outcomes. Record it — a pass without a recorded conclusion is indistinguishable from a run nobody finished.

### 🟢 Pass — all fourteen steps succeeded

Record **date, commit SHA, OS and build version**, plus anything unusual about the environment (timezone, DST boundary, non-English locale, unusual display scaling). The environment matters because several invariants here are timezone- or clock-sensitive: `next_default_name` resets on *local* midnight, and gap recovery compares instants across a shutdown.

This is the artifact that makes future failures interpretable. Without a dated clean run, a later failure raises "did this ever work?"; with one, it is a **regression against a known-good commit**, which is a far cheaper question to answer.

**Tag the commit `validated-baseline-N`** (sequential, like ADRs), with a message naming the date and outcome. The project already marks *accepted architecture*; this marks something different and worth distinguishing:

> **Accepted** means we believe this architecture is correct.
> **Validated** means we have exercised the complete system and observed it behaving correctly.

Everything after a validated baseline can be measured against it. Nothing before one can.

### 🟡 Finding — one or more steps failed, but data integrity is intact

Finish the remaining steps if it is safe to do so — a single pass often surfaces more than one thing, and stopping early wastes the setup. Then open an issue per finding, link it here, and classify each one:

- **Implementation evidence** — an accepted document claims behaviour the application does not have, or vice versa. Amend the document from the evidence; do **not** work around it in code (`.claude/docs-standards.md`, "Treating accepted decisions during implementation").
- **Product bug** — the documents and the code agree, and the behaviour is simply wrong. Fix the code.

The distinction matters more than it looks: this project has found both, and treating the first as the second is how architecture quietly drifts away from what is written down.

### 🔴 Abort — a data-integrity failure

**Stop immediately. Do not continue the checklist, and do not relaunch the application.**

Step 12's condition is the clearest example: `snapshot.json` missing while `transitions.jsonl` is already empty. Any state that looks irrecoverable qualifies.

**Relaunching destroys the evidence, and this is not a precaution — startup mutates state by design.** `LogWriter::open` truncates any bytes past the final record boundary, and `AppState::init` appends a `RecoverGap` transition whenever replay leaves something active. Both are correct behaviours that exist to make the app recover; both also overwrite exactly what an investigation would need to read. So, in order:

1. **Do not start the app again.**
2. Copy the whole data directory somewhere safe — `transitions.jsonl`, `snapshot.json`, and its `.tmp` sibling if present. A leftover `snapshot.json.tmp` is itself a strong signal, since a successful write renames it away.
3. Note what the last action was before the failure, and whether the shutdown was clean.
4. Open an issue with the copied files attached or their contents quoted.

An abort is the most valuable result this checklist can produce, and the easiest to accidentally erase.

## Recording the result

Add a dated line below for each full pass, including clean ones.

| Date | Commit | OS / build | Outcome | Notes |
|---|---|---|---|---|
| 2026-08-03 | `936281f` | Win 11 Pro 10.0.26200 · de-DE · W. Europe (UTC+1, DST on) | 🟢 **14/14 PASS** | First validated baseline — tagged `validated-baseline-1`. See below. |
| 2026-08-03 | `122d3ab` | Win 11 Pro 10.0.26200 · de-DE · W. Europe (UTC+1, DST on) | ⏳ **partial — 4a/4b/4c only** | ADR 0007's three gap zones verified against the running binary. **Not** a baseline: steps 1–3, 5–11 and 14 not re-run. |

### Run 2 (partial) — 2026-08-03, commit `122d3ab`

ADR 0007 changed gap recovery, so step 4 was re-run in all three of its new zones. The preconditions were constructed by seeding `transitions.jsonl` with a backdated `start` (checksum reproduced and verified against a line the app itself wrote) and launching the real desktop binary — the *decision* under test is what the running `AppState::init` does, and that is what was observed.

| Zone | Expected | Observed |
|---|---|---|
| **4c** — 12s gap | no transition at all | ✅ log holds only the seeded `start`; the block carried on |
| **4a** — 10min gap | close at last durable write, restart at recovery | ✅ `recover-gap → 16:45:47`, then `start` at `16:55:56` |
| **4b** — 9h gap | close only | ✅ `recover-gap → 07:57:30`, nothing restarted |

The middle row is the guarantee worth naming: the gap closed at the last durable write while the resumed block starts ten minutes later, so the outage stays a visible hole rather than billable time.

**4c is the fix for run 1's artifact.** That same 12-second outage previously wrote a `recover-gap` whose `inferred_end` equalled the start, producing a zero-duration block. It now writes nothing.

**Why this is not `validated-baseline-2`.** Steps 1–3, 5–11 and 14 require driving the native window, which this run could not do. They are unchanged code paths that passed in run 1, but the premise of this checklist is that seams fail where nobody looked — an assumption of "unchanged, therefore fine" is exactly what it exists to distrust. A full pass by hand is still owed before tagging.

### Run 1 — 2026-08-03, commit `936281f`

**Steps 1–3, 5–11: passed by hand.**

**Step 4: passed** — initially reported as a failure, reclassified on evidence. The log shows `start Anchor 5 @16:02:44` then `recover-gap inferred_end=16:02:44` written at 16:02:57. The inferred end is the **last durable write, not the relaunch** — which is exactly what the step checks. "No active task after relaunching" is the specified behaviour, not a fault: `RecoverGap` deliberately does not auto-resume.

Two earlier kills in the same log confirm the mechanism with heartbeats landing: `A` 52s, `Weiterentwicklung - Anchor` 102s.

*Observation, not a defect:* a kill inside the first heartbeat interval makes the start itself the last durable write, so the recovered block has a **zero-second duration** (`Anchor 5`). Within R4's ~60s bound and therefore correct, but a 0-minute block on the timeline is an artifact worth a deliberate decision rather than a discovery later.

**Steps 12–13: passed, verified directly.** The first attempt left no snapshot because the app was stopped by killing npm — `RunEvent::Exit` never fires that way, so `compact_on_shutdown` never runs. Not a defect, but note that **step 4 and step 12 need different shutdown methods**, and using the ungraceful one for both makes each look ambiguous.

Re-run with a graceful close (`WM_CLOSE`): `transitions.jsonl` 55 records → **0**, and `snapshot.json` appeared (v2, watermark 54, 28 closed blocks, 10 issued auto-names). Relaunching from that snapshot and closing again produced a second snapshot whose fingerprint over every block's id, name, span, capture origin and end determination is **identical** — replay across the compaction boundary is lossless.

**Step 14: passed.** `edit-identity` at seq 55 targeting `c55f846f` — the block `Anchor 7`, one of the 28 held **only** in the snapshot. The log was empty before that edit, so the target could not have come from any surviving log line. **A14 confirmed on the running application**, not merely in a test: a block below the watermark is still a valid reconstruction target.

**Outcome: 🟢 14/14, tagged `validated-baseline-1`.** Anchor now has something it did not have before this run — not just components asserted correct in isolation, but the whole system observed behaving correctly end to end.
