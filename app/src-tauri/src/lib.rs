pub mod commands;
pub mod log;
pub mod model;
pub mod paths;
pub mod stack;
pub mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            let log_path = paths::log_file_path(handle)?;
            let (state, torn_line_discarded) = AppState::init(&log_path)?;
            if torn_line_discarded {
                // Slice 1 has no UI surface for this yet — at minimum, don't lose
                // the signal silently.
                eprintln!(
                    "warning: a torn/corrupt trailing line was found in {} and discarded on startup",
                    log_path.display()
                );
            }
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::switch,
            commands::interrupt,
            commands::return_previous,
            commands::return_original,
            commands::complete,
            commands::get_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
