# Switch between mini widget and full window UI

A way to toggle between the two existing windows (the always-on-top mini widget and the full dashboard) — e.g. a button or hotkey that hides one and shows the other, rather than having both open simultaneously as they currently can be.

Raw idea, not yet scoped. Open questions for whenever this goes through Discovery/Design:
- Is the goal to ever have only one window visible at a time (a true mode switch), or just a fast way to bring the other one forward while both still technically exist?
- Should this get its own hotkey (alongside the five already registered in `app/src-tauri/src/lib.rs`), a UI control on each window, or both?
- Interacts with the mini widget's `alwaysOnTop`/`skipTaskbar` window config (`app/src-tauri/tauri.conf.json`) — switching "into" widget mode presumably means hiding the dashboard, not closing it (state must persist).
