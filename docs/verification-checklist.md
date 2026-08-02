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
Watch: nothing auto-resumes. Recovery deliberately does not restart the task.

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
Watch: `snapshot.json` appears, and `transitions.jsonl` is now **empty**. If the snapshot is missing while the log is empty, stop: that is total data loss and the ordering guarantee has been violated.

**13. Relaunch.**
Seam: snapshot → replay → identical projection.
Watch: the History View is **exactly** as you left it — same blocks, same names, same times, same capture origins, same interruption statuses. Everything below the watermark now comes from the snapshot rather than the log.

**14. Edit Identity on a block that existed before the compaction, then restart once more.**
Seam: **A14** — identity surviving a truncated log.
Watch: the edit resolves against the right block and survives. This is the one step that proves a block below the watermark is still addressable; if identity came only from the log, there would be nothing left to name.

---

## Recording the result

Add a dated line below for each full pass, including clean ones.

| Date | Build | Result | Notes |
|---|---|---|---|
| _(not yet run)_ | | | |

**If a step fails**, treat it as implementation evidence, not a UI annoyance: check whether an accepted document claims the behaviour that just failed, and if so amend the document from the evidence rather than working around it in code (`.claude/docs-standards.md`, "Treating accepted decisions during implementation").
