//! Resolves where the transition log lives, using Tauri's per-user app-data dir.

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn log_file_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("transitions.jsonl"))
}
