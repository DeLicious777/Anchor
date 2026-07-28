# Switch between mini widget and full window UI

A way to toggle between the two existing windows (the always-on-top mini widget and the full dashboard) — a control or hotkey that shows one and hides the other, rather than having both open simultaneously as they currently can be.

## Decided (2026-07-28)

- **A true mode switch**: exactly one window visible at a time. Not a bring-to-front convenience, and not configurable to keep both — one window, one mode.
- **Hide, never close.** Both windows are declared statically in `app/src-tauri/tauri.conf.json`; hiding preserves window state and avoids a re-create path. Nothing about the switch touches tracking state, which lives in the Rust `AppState`/transition log, not in either window.

## Still open

- **What triggers it** — a sixth global hotkey alongside the five already registered (`HotkeyAction::{Switch, Interrupt, ReturnPrevious, ReturnOriginal, Complete}` in `app/src-tauri/src/hotkeys.rs`), a UI control on each window, or both? A hotkey fits the project's fast-interaction thesis; a UI control is discoverable. These aren't exclusive.
- **Naming collision worth avoiding**: the existing `HotkeyAction::Switch` means "switch *task*". A window-switch action needs a name that can't be confused with it, in both the settings UI and the docs — "Switch" appearing twice with different meanings in a remappable-bindings list is a real usability trap.
- **What is the startup mode?** If only one window shows at a time, which one opens on launch, and is that remembered across sessions?
- **Does the mode survive a crash/restart** alongside the gap-recovery path, or always reset to a default?
- **What happens to the mode when the user needs the dashboard implicitly** — e.g. correcting a `recovered-gap` entry, which `docs/product/features/interruption-stack.md` says happens "whenever the user next opens the dashboard"? Does anything auto-switch, or is it always user-initiated?

## Interacts with

- [`visual-redesign.md`](visual-redesign.md) — a true mode switch makes each window a self-contained mode rather than a pair of panes. That's an information-architecture premise for the redesign, so it should be settled *before* the IA pass, even though the redesign is built first.
- Widget config (`alwaysOnTop`, `skipTaskbar: true`, `resizable: false`) is unchanged by this; only visibility toggles. Note `skipTaskbar: true` means that while in widget mode there is no taskbar entry — if the hotkey fails to register (the `register_bindings` failure path already logs a warning), the user could be left with no obvious way back to the dashboard. That argues for a UI control on the widget as a floor, not just a hotkey.
