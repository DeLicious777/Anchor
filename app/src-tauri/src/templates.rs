//! Task Templates: reusable name/project/client presets, stored entirely
//! separately from the transition log — a template is never a tracked event
//! and never replayed (see `docs/product/features/task-templates.md`). This
//! module deliberately has its own independent lock (`TemplateState`), not
//! folded into `AppState.inner` — templates and the interruption stack are
//! genuinely independent concerns that never need atomic joint mutation.

use crate::model::TaskTemplate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TemplateStore {
    templates: Vec<TaskTemplate>,
}

impl TemplateStore {
    /// Missing file or corrupt JSON both fall back to an empty store — a
    /// hand-corrupted templates file must never block the app from starting,
    /// same principle as `settings::HotkeyBindings::load`.
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

    pub fn list(&self) -> &[TaskTemplate] {
        &self.templates
    }

    /// Duplicates — including an exact (name, project, client) match — are
    /// allowed unconditionally. The feature doc explicitly allows two templates
    /// sharing a name with different project/client and never asks for a
    /// uniqueness constraint on the full triple; inventing one adds an
    /// undocumented failure mode for a preset with no referential-integrity
    /// requirement (unlike Time Blocks).
    pub fn create(&mut self, name: String, project: Option<String>, client: Option<String>) -> TaskTemplate {
        let template = TaskTemplate::new(name, project, client);
        self.templates.push(template.clone());
        template
    }

    pub fn update(
        &mut self,
        id: Uuid,
        name: String,
        project: Option<String>,
        client: Option<String>,
    ) -> Result<TaskTemplate, String> {
        let template = self
            .templates
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("template {id} not found"))?;
        template.name = name;
        template.project = project;
        template.client = client;
        Ok(template.clone())
    }

    pub fn delete(&mut self, id: Uuid) -> Result<(), String> {
        let before = self.templates.len();
        self.templates.retain(|t| t.id != id);
        if self.templates.len() == before {
            return Err(format!("template {id} not found"));
        }
        Ok(())
    }
}

pub struct TemplateState {
    pub inner: Mutex<TemplateStore>,
    path: PathBuf,
}

impl TemplateState {
    pub fn init(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let store = TemplateStore::load(&path);
        Self { inner: Mutex::new(store), path }
    }
}

/// The single entry point every template mutation goes through — mirrors
/// `commands::apply_transition`'s "durable write happens under the same lock
/// as the in-memory mutation" discipline, but simpler: no dry-run is needed
/// since CRUD here is unconditional (no precondition to violate). Snapshots
/// the store before mutating and rolls back in memory if the save fails, so a
/// rare disk-write failure can never leave memory and disk silently diverged
/// within a session.
pub fn mutate_templates<T>(
    state: &TemplateState,
    f: impl FnOnce(&mut TemplateStore) -> Result<T, String>,
) -> Result<(T, Vec<TaskTemplate>), String> {
    let mut store = state.inner.lock().map_err(|_| "template store lock poisoned".to_string())?;
    let backup = store.clone();
    let result = f(&mut store)?;
    if let Err(e) = store.save(&state.path) {
        *store = backup;
        return Err(e.to_string());
    }
    Ok((result, store.list().to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_of_missing_file_returns_default_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        assert_eq!(TemplateStore::load(&path), TemplateStore::default());
        assert!(TemplateStore::load(&path).list().is_empty());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("templates.json");
        let mut store = TemplateStore::default();
        store.create("Standup".into(), Some("Acme".into()), None);

        store.save(&path).unwrap();
        let loaded = TemplateStore::load(&path);
        assert_eq!(loaded, store);
    }

    #[test]
    fn load_of_corrupt_file_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("templates.json");
        std::fs::write(&path, "not valid json").unwrap();
        assert_eq!(TemplateStore::load(&path), TemplateStore::default());
    }

    #[test]
    fn create_appends_and_assigns_fresh_uuid() {
        let mut store = TemplateStore::default();
        let a = store.create("A".into(), None, None);
        let b = store.create("B".into(), None, None);
        assert_ne!(a.id, b.id);
        assert_eq!(store.list().len(), 2);
    }

    #[test]
    fn update_by_id_changes_fields_and_leaves_others_untouched() {
        let mut store = TemplateStore::default();
        let a = store.create("A".into(), None, None);
        let b = store.create("B".into(), Some("Acme".into()), None);

        let updated = store.update(a.id, "A2".into(), Some("Globex".into()), Some("client1".into())).unwrap();
        assert_eq!(updated.name, "A2");
        assert_eq!(updated.project, Some("Globex".to_string()));

        let b_untouched = store.list().iter().find(|t| t.id == b.id).unwrap();
        assert_eq!(b_untouched.name, "B");
        assert_eq!(b_untouched.project, Some("Acme".to_string()));
    }

    #[test]
    fn update_of_unknown_id_returns_err() {
        let mut store = TemplateStore::default();
        let err = store.update(Uuid::new_v4(), "X".into(), None, None).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn delete_by_id_removes_only_that_template() {
        let mut store = TemplateStore::default();
        let a = store.create("A".into(), None, None);
        let b = store.create("B".into(), None, None);

        store.delete(a.id).unwrap();
        assert_eq!(store.list().len(), 1);
        assert_eq!(store.list()[0].id, b.id);
    }

    #[test]
    fn delete_of_unknown_id_returns_err() {
        let mut store = TemplateStore::default();
        let err = store.delete(Uuid::new_v4()).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn creating_two_templates_with_identical_name_project_client_is_allowed() {
        let mut store = TemplateStore::default();
        let a = store.create("Standup".into(), None, None);
        let b = store.create("Standup".into(), None, None);
        assert_ne!(a.id, b.id, "distinct instances even with identical content");
        assert_eq!(store.list().len(), 2, "no dedup — duplicates are allowed unconditionally");
    }

    #[test]
    fn mutate_templates_round_trips_via_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("templates.json");
        let state = TemplateState::init(&path);

        let (created, list) = mutate_templates(&state, |store| Ok(store.create("A".into(), None, None))).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);

        // Persisted durably — a fresh TemplateState from the same path sees it.
        let reloaded = TemplateState::init(&path);
        assert_eq!(reloaded.inner.lock().unwrap().list().len(), 1);
    }

    #[test]
    fn mutate_templates_rolls_back_in_memory_if_save_fails() {
        let dir = tempfile::tempdir().unwrap();
        // A path whose parent directory doesn't exist — save() will fail.
        let unwritable_path = dir.path().join("no-such-dir").join("templates.json");
        let state = TemplateState::init(&unwritable_path);

        let result = mutate_templates(&state, |store| Ok(store.create("A".into(), None, None)));
        assert!(result.is_err());

        // The in-memory store must NOT reflect the failed mutation — otherwise
        // this session's view would diverge from what's actually on disk.
        assert!(state.inner.lock().unwrap().list().is_empty());
    }
}
