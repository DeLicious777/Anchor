//! Persisted selection intent for the Timeline and History View's shared range.
//!
//! Presets are stored as presets rather than resolved instants, so `Today` and
//! `ThisWeek` continue to mean the current day/week after a restart. Custom
//! ranges store inclusive calendar dates. Resolving either form to instants is
//! surface work for M3c; Export deliberately owns a separate range.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum ViewRange {
    Today,
    ThisWeek,
    Custom { start: NaiveDate, end: NaiveDate },
}

impl Default for ViewRange {
    fn default() -> Self {
        Self::Today
    }
}

impl ViewRange {
    fn validate(&self) -> Result<(), String> {
        match self {
            Self::Custom { start, end } if start > end => Err(format!(
                "custom view range start {start} must not be after end {end}"
            )),
            _ => Ok(()),
        }
    }

    /// Missing, unreadable, corrupt, or invalid file -> Today. A preference
    /// file must never prevent the app from starting.
    pub fn load(path: impl AsRef<Path>) -> Self {
        std::fs::read_to_string(path.as_ref())
            .ok()
            .and_then(|contents| serde_json::from_str::<Self>(&contents).ok())
            .filter(|range| range.validate().is_ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        self.validate()
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
}

pub struct ViewRangeState {
    pub inner: Mutex<ViewRange>,
    path: PathBuf,
}

impl ViewRangeState {
    pub fn init(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let range = ViewRange::load(&path);
        Self {
            inner: Mutex::new(range),
            path,
        }
    }
}

/// Mutate the shared preference as one durable operation. Validation and save
/// failure both restore the previous in-memory value so both consumers can
/// never observe state that a restart would discard.
pub fn mutate_view_range(
    state: &ViewRangeState,
    f: impl FnOnce(&mut ViewRange),
) -> Result<ViewRange, String> {
    let mut range = state
        .inner
        .lock()
        .map_err(|_| "view range lock poisoned".to_string())?;
    let backup = range.clone();

    f(&mut range);

    if let Err(message) = range.validate() {
        *range = backup;
        return Err(message);
    }

    if let Err(error) = range.save(&state.path) {
        *range = backup;
        return Err(format!("failed to persist view range: {error}"));
    }

    Ok(range.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_settings::{mutate_export_settings, ExportSettingsState};

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn default_is_today_and_missing_unreadable_or_corrupt_files_fall_back_to_it() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.json");
        assert_eq!(ViewRange::default(), ViewRange::Today);
        assert_eq!(ViewRange::load(&missing), ViewRange::Today);

        let unreadable = dir.path().join("directory-not-file");
        std::fs::create_dir(&unreadable).unwrap();
        assert_eq!(ViewRange::load(&unreadable), ViewRange::Today);

        let corrupt = dir.path().join("corrupt.json");
        std::fs::write(&corrupt, "not json").unwrap();
        assert_eq!(ViewRange::load(&corrupt), ViewRange::Today);

        let incomplete = dir.path().join("incomplete.json");
        std::fs::write(&incomplete, r#"{"mode":"custom","start":"2026-08-03"}"#).unwrap();
        assert_eq!(ViewRange::load(&incomplete), ViewRange::Today);

        let reversed = dir.path().join("reversed.json");
        std::fs::write(
            &reversed,
            r#"{"mode":"custom","start":"2026-08-07","end":"2026-08-03"}"#,
        )
        .unwrap();
        assert_eq!(ViewRange::load(&reversed), ViewRange::Today);
    }

    #[test]
    fn every_intent_has_a_stable_tagged_json_shape_and_round_trips() {
        let cases = [
            (ViewRange::Today, serde_json::json!({ "mode": "today" })),
            (
                ViewRange::ThisWeek,
                serde_json::json!({ "mode": "this-week" }),
            ),
            (
                ViewRange::Custom {
                    start: date(2026, 8, 3),
                    end: date(2026, 8, 7),
                },
                serde_json::json!({
                    "mode": "custom",
                    "start": "2026-08-03",
                    "end": "2026-08-07"
                }),
            ),
        ];

        for (range, expected_json) in cases {
            let encoded = serde_json::to_value(&range).unwrap();
            assert_eq!(encoded, expected_json);
            assert_eq!(serde_json::from_value::<ViewRange>(encoded).unwrap(), range);
        }
    }

    #[test]
    fn each_intent_saves_and_loads_without_resolving_presets_to_dates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("view_range.json");
        for range in [
            ViewRange::Today,
            ViewRange::ThisWeek,
            ViewRange::Custom {
                start: date(2026, 7, 1),
                end: date(2026, 7, 31),
            },
        ] {
            range.save(&path).unwrap();
            assert_eq!(ViewRange::load(&path), range);
        }
    }

    #[test]
    fn reversed_custom_range_is_rejected_without_changing_memory_or_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("view_range.json");
        let state = ViewRangeState::init(&path);
        mutate_view_range(&state, |range| *range = ViewRange::ThisWeek).unwrap();
        let disk_before = std::fs::read_to_string(&path).unwrap();

        let error = mutate_view_range(&state, |range| {
            *range = ViewRange::Custom {
                start: date(2026, 8, 7),
                end: date(2026, 8, 3),
            };
        })
        .unwrap_err();

        assert!(error.contains("must not be after"));
        assert_eq!(*state.inner.lock().unwrap(), ViewRange::ThisWeek);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), disk_before);
    }

    #[test]
    fn mutation_persists_and_reloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("view_range.json");
        let state = ViewRangeState::init(&path);
        let custom = ViewRange::Custom {
            start: date(2026, 8, 1),
            end: date(2026, 8, 7),
        };

        assert_eq!(
            mutate_view_range(&state, |range| *range = custom.clone()).unwrap(),
            custom
        );
        assert_eq!(*ViewRangeState::init(&path).inner.lock().unwrap(), custom);
    }

    #[test]
    fn save_failure_rolls_back_memory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing-parent").join("view_range.json");
        let state = ViewRangeState::init(&path);

        let error = mutate_view_range(&state, |range| *range = ViewRange::ThisWeek).unwrap_err();
        assert!(error.contains("failed to persist view range"));
        assert_eq!(*state.inner.lock().unwrap(), ViewRange::Today);
    }

    #[test]
    fn changing_view_range_does_not_touch_export_settings() {
        let dir = tempfile::tempdir().unwrap();
        let export_path = dir.path().join("export_settings.json");
        let view_path = dir.path().join("view_range.json");
        let export_state = ExportSettingsState::init(&export_path);
        mutate_export_settings(&export_state, |settings| {
            settings.rounding_enabled = false;
            settings.rounding_interval_minutes = 5;
        })
        .unwrap();
        let export_before = std::fs::read_to_string(&export_path).unwrap();

        let view_state = ViewRangeState::init(&view_path);
        mutate_view_range(&view_state, |range| *range = ViewRange::ThisWeek).unwrap();

        assert_eq!(
            std::fs::read_to_string(&export_path).unwrap(),
            export_before
        );
        assert_eq!(
            export_state.inner.lock().unwrap().rounding_interval_minutes,
            5
        );
    }
}
