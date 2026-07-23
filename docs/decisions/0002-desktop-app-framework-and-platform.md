---
status: accepted
date: 2026-07-23
owner: erich
related: [docs/product/features/interruption-stack.md, docs/architecture/constraints.md, docs/decisions/0001-manual-assisted-tracking-for-mvp.md, docs/risks.md, docs/assumptions.md]
---

# 0002: Desktop App Framework and Target Platform

## Context

`docs/product/features/interruption-stack.md` requires: user-remappable global hotkeys, an always-on-top mini widget, a full dashboard window, and durably-persisted (append-only, crash-atomic) local storage — and explicitly deferred "exact hotkey API and target OS(es)" to this workflow. Constraints from the author: Windows is the MVP/target platform, with cross-platform (macOS/Linux) likely useful later without a full rewrite; frontend stack preference is React or Svelte with TypeScript and Ramda; low memory footprint is the top priority (this is an always-running background app), with startup speed second.

## Options Considered

1. **Electron** (bundled Chromium + Node.js runtime) + React/Svelte + TypeScript.
2. **Tauri** (Rust core process + the OS's native webview — WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux — instead of a bundled browser engine) + React/Svelte + TypeScript.
3. **Native Windows-only** (WinUI3 + C#/.NET).
4. **JVM-based** (JavaFX, or a Kotlin/Compose Multiplatform desktop target).

## Trade-offs

Memory/startup figures below are widely-reported, order-of-magnitude characterizations (Electron's overhead comes from bundling a full Chromium instance per app; Tauri's own project documentation claims smaller binaries/lower memory specifically *because* it reuses the OS's resident webview instead of bundling one) — not independently benchmarked for this specific app. Treat the relative ordering as reliable and the absolute numbers as illustrative only.

| | Electron (1) | Tauri (2) | Native Windows (3) | JVM-based (4) |
|---|---|---|---|---|
| Memory footprint | High — bundles a full Chromium instance (commonly cited as 100+ MB baseline, multiple processes) | Low — reuses the OS's already-resident native webview, no bundled browser engine | Lowest of the four — no webview/browser engine at all | Medium-high — JVM baseline memory is non-trivial even before app code runs |
| Startup speed | Slower — Chromium engine cold start | Fast — native webview is already part of the OS | Fastest — no runtime/engine to initialize beyond the OS's own UI toolkit | Slower — JVM cold start is a known weak point for desktop tools |
| Frontend stack fit | Full fit — Svelte/TS/Ramda run as-is | Full fit — same frontend stack, Rust only for OS-level bridges | No fit — requires XAML + C#, abandoning the Svelte/TS/Ramda stack entirely | No fit — requires JavaFX/Swing or Compose (Kotlin), not Svelte/TS |
| Cross-platform story (later) | Strong — same Chromium bundle everywhere | Workable — same core architecture on macOS/Linux, but each OS's native webview has its own quirks (e.g. WebKitGTK on Linux can lag behind modern web APIs) | Very weak — WinUI3 is Windows-only; a macOS/Linux target would mean rebuilding the entire UI layer | Workable in principle (JVM is cross-platform) but the frontend stack is already given up in (4) |
| New learning surface | None beyond the desired frontend stack | A Rust core process — see Consequences for the actual scope, which is wider than just "hotkeys" | C#/.NET and XAML, a full stack switch away from stated preference | Java/Kotlin, a full stack switch away from stated preference |
| Ecosystem maturity for this app's needs | Mature — hotkeys, tray, file I/O all well-trodden | Maturing, not mature — `tauri-plugin-global-shortcut` and multi-window support exist and are actively maintained, but the ecosystem is meaningfully younger than Electron's; see Consequences for what this means concretely (sleep/hibernate detection isn't provided out of the box) | Mature within Windows, irrelevant elsewhere | Mature for enterprise apps, less common for lightweight always-on background tools |

Native Windows (3) and JVM-based (4) are not quantified more precisely on memory/startup than "low"/"medium-high" above, because the frontend-stack fit already rules them out on a constraint (see Decision) — further precision wouldn't change the outcome. If the frontend-stack constraint in `docs/architecture/constraints.md` is ever revisited, this comparison should be redone with real numbers before re-deciding.

## Decision

**Tauri, with a Svelte + TypeScript frontend (using Ramda for functional composition), targeting Windows for MVP.**

This is the only option that satisfies the frontend stack constraint (`docs/architecture/constraints.md`) *and* wins decisively on both stated priorities (memory footprint, then startup speed), while keeping a realistic cross-platform path for later. Electron shares the frontend fit but loses on both priorities by a wide margin for an always-running background app. Native Windows and JVM-based options were rejected because they require abandoning that stack constraint entirely — this ADR is what promoted the stack from a stated preference to a documented constraint (see `docs/architecture/constraints.md`), precisely because it was doing the work of eliminating two of the four options outright rather than being weighed as one factor among several.

Svelte over React specifically because it compiles away most of its runtime, which is a marginally better fit for the stated top priority (low memory footprint) than React's runtime — a low-stakes, reversible choice, since it only affects the frontend layer, not the data model or Rust core.

**This decision should be revisited (new ADR, not an edit) if:** `tauri-plugin-global-shortcut` turns out not to support hotkey-conflict detection at all (an unconfirmed Acceptance Criterion dependency — see Consequences), or if the sleep/hibernate detection work (direct Windows API calls via `WM_POWERBROADCAST`, not a Tauri-provided feature) takes more than roughly one week of dedicated effort to reach a working prototype — a concrete, observable trigger rather than a subjective "harder than expected."

## Consequences

- **Resolves the hotkey-API and target-OS deferrals from `docs/product/features/interruption-stack.md`**: global hotkeys are implemented via Tauri's `global-shortcut` plugin (OS-registered, user-remappable). Whether this plugin actually surfaces a distinguishable "already registered by another process" error (required by that feature doc's Acceptance Criteria) is **not yet confirmed** — flagged as an open implementation question, not assumed solved.
- **Does NOT fully resolve gap detection on its own — sleep/hibernate specifically requires new, unscoped work.** `docs/product/features/interruption-stack.md` requires detecting an OS-level sleep/hibernate signal while an entry is active (Technical Constraints), but Tauri does not expose this natively. It requires direct Windows API calls from the Rust core (e.g. handling `WM_POWERBROADCAST` / `PBT_APMRESUMESUSPEND` via the `windows` crate or equivalent) — genuinely new, platform-specific, low-level Rust work, not covered by the general "OS bridges" framing below. This is a real risk for a solo, likely-Rust-inexperienced maintainer; see `docs/risks.md` R6 and `docs/assumptions.md`.
- **Torn-write detection is not resolved by this ADR.** Durable fsync/flush per record (chosen in `docs/product/features/interruption-stack.md`) guarantees a *completed* write survives a crash, but says nothing about *detecting* a partial/torn record — that requires an explicit framing scheme (e.g. a length prefix or checksum per record), which is not decided here. Deferred to implementation, same as the exact on-disk format below — but named explicitly so it isn't mistaken for already handled.
- **The actual Rust surface area is wider than "hotkeys"**: OS bridges (hotkeys, tray, window management), cross-window IPC/state sync between the mini widget and dashboard, the append-only log's read/write/compaction, the heartbeat timer, and (per the point above) sleep/hibernate detection. That's several distinct subsystems, not one narrow bridge — a real new-language cost, accepted because nothing else satisfies the memory/startup priorities as well as the frontend-stack constraint together.
- **Two-window architecture**: the mini widget and dashboard are two separate Tauri windows sharing one backend process and one source of truth for state — window-management (creation, focus behavior, always-on-top for the widget) becomes a concrete implementation concern for whoever builds this, not yet spec'd further here.
- **WebView2 dependency on Windows**: Tauri requires the WebView2 runtime. It ships preinstalled on current Windows 10/11, but the build/release process needs to handle (or explicitly document) the case of an older or stripped-down Windows install missing it.
- **Cross-platform quirks are deferred, not solved**: when macOS/Linux support is actually built, WKWebView and WebKitGTK behavioral differences (especially WebKitGTK's sometimes-lagging web API support) will need their own evaluation — not a blocker for the Windows-only MVP, but should be revisited as its own design pass before cross-platform work starts, not assumed to be free.
- **Exact on-disk log format** (e.g. newline-delimited JSON vs. a compact binary encoding, plus the torn-write framing scheme above) is not decided here — that's a smaller, more reversible implementation choice, not an architecture-level fork, and can be made when the persistence layer is actually built.
