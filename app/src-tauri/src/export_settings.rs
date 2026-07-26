//! Persisted rounding preference for Export (see
//! `docs/product/features/export.md`). Load/save mirrors `settings.rs`'s
//! `HotkeyBindings` shape exactly; the managed, mutable `State` wrapper mirrors
//! `templates.rs`'s `TemplateState`/`mutate_templates` (including the
//! snapshot-and-rollback-on-save-failure guarantee), since the dashboard reads
//! and updates this live while the app runs, unlike hotkeys which are only
//! read once at startup.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportSettings {
    pub rounding_enabled: bool,
    pub rounding_interval_minutes: u32,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self { rounding_enabled: true, rounding_interval_minutes: 15 }
    }
}

impl ExportSettings {
    /// Missing or corrupt file → defaults, never blocks startup (same
    /// reasoning as `HotkeyBindings::load`: a hand-edited typo shouldn't
    /// prevent the app from launching).
    pub fn load(path: impl AsRef<Path>) -> Self {
        match std::fs::read_to_string(path.as_ref()) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

pub struct ExportSettingsState {
    pub inner: Mutex<ExportSettings>,
    path: PathBuf,
}

impl ExportSettingsState {
    pub fn init(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let settings = ExportSettings::load(&path);
        Self { inner: Mutex::new(settings), path }
    }
}

/// Same snapshot-then-save-then-rollback-on-failure shape as
/// `templates::mutate_templates` — a rare disk write failure must never leave
/// memory and disk silently diverged within a session.
pub fn mutate_export_settings(
    state: &ExportSettingsState,
    f: impl FnOnce(&mut ExportSettings),
) -> Result<ExportSettings, String> {
    let mut settings = state.inner.lock().map_err(|_| "export settings lock poisoned".to_string())?;
    let backup = settings.clone();

    f(&mut settings);

    if let Err(e) = settings.save(&state.path) {
        *settings = backup;
        return Err(format!("failed to persist export settings: {e}"));
    }

    Ok(settings.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_of_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        assert_eq!(ExportSettings::load(&path), ExportSettings::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export_settings.json");
        let settings = ExportSettings { rounding_enabled: false, rounding_interval_minutes: 5 };
        settings.save(&path).unwrap();

        assert_eq!(ExportSettings::load(&path), settings);
    }

    #[test]
    fn load_of_corrupt_file_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export_settings.json");
        std::fs::write(&path, "not valid json").unwrap();
        assert_eq!(ExportSettings::load(&path), ExportSettings::default());
    }

    #[test]
    fn mutate_export_settings_round_trips_via_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export_settings.json");
        let state = ExportSettingsState::init(&path);

        let updated = mutate_export_settings(&state, |s| {
            s.rounding_enabled = false;
            s.rounding_interval_minutes = 10;
        })
        .unwrap();
        assert_eq!(updated.rounding_interval_minutes, 10);

        let reloaded = ExportSettingsState::init(&path);
        assert_eq!(*reloaded.inner.lock().unwrap(), updated);
    }

    #[test]
    fn mutate_export_settings_rolls_back_in_memory_if_save_fails() {
        let dir = tempfile::tempdir().unwrap();
        // A path whose parent directory doesn't exist forces the save to fail.
        let path = dir.path().join("missing-subdir").join("export_settings.json");
        let state = ExportSettingsState::init(&path);
        let original = state.inner.lock().unwrap().clone();

        let err = mutate_export_settings(&state, |s| {
            s.rounding_enabled = !s.rounding_enabled;
        })
        .unwrap_err();
        assert!(err.contains("failed to persist"));

        assert_eq!(*state.inner.lock().unwrap(), original, "in-memory state must roll back when the save fails");
    }
}
