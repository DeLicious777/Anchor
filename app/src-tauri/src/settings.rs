//! Persisted, user-remappable hotkey bindings. Remapping itself is a later
//! polish item (this slice has no settings UI) — but the underlying mechanism
//! (load a binding, register it, have it survive a restart) is what the
//! Acceptance Criteria actually test, so that part is built now.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyBindings {
    pub switch: String,
    pub interrupt: String,
    pub return_previous: String,
    pub return_original: String,
    pub complete: String,
}

impl Default for HotkeyBindings {
    fn default() -> Self {
        Self {
            switch: "Ctrl+Alt+S".to_string(),
            interrupt: "Ctrl+Alt+I".to_string(),
            return_previous: "Ctrl+Alt+P".to_string(),
            return_original: "Ctrl+Alt+O".to_string(),
            complete: "Ctrl+Alt+C".to_string(),
        }
    }
}

impl HotkeyBindings {
    /// Loads bindings from `path` if it exists and parses; falls back to
    /// defaults for a missing file OR a corrupt one (a hand-edited typo in the
    /// settings file shouldn't prevent the app from starting at all — sane
    /// defaults are always better than refusing to launch).
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match std::fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_of_missing_file_returns_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        assert_eq!(HotkeyBindings::load(&path), HotkeyBindings::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let mut bindings = HotkeyBindings::default();
        bindings.switch = "Ctrl+Shift+X".to_string();
        bindings.save(&path).unwrap();

        let loaded = HotkeyBindings::load(&path);
        assert_eq!(loaded, bindings);
        assert_eq!(loaded.switch, "Ctrl+Shift+X", "a remapped binding must persist across a reload (simulates surviving a restart)");
    }

    #[test]
    fn load_of_corrupt_file_falls_back_to_defaults_rather_than_failing_to_start() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "not valid json at all").unwrap();
        assert_eq!(HotkeyBindings::load(&path), HotkeyBindings::default());
    }
}
