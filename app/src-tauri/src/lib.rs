pub mod commands;
pub mod export;
pub mod export_settings;
pub mod heartbeat;
pub mod hotkeys;
pub mod log;
pub mod model;
pub mod paths;
pub mod power;
pub mod settings;
pub mod stack;
pub mod state;
pub mod templates;

use export_settings::ExportSettingsState;
use hotkeys::{HotkeyAction, HotkeyState};
use settings::HotkeyBindings;
use state::AppState;
use tauri::Manager;
use templates::TemplateState;
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let Some(hotkey_state) = app.try_state::<HotkeyState>() else {
                        return;
                    };
                    let Ok(registered) = hotkey_state.registered.lock() else {
                        return;
                    };
                    let Some(action) = registered.iter().find(|(s, _)| s == shortcut).map(|(_, action)| *action) else {
                        return;
                    };
                    drop(registered);

                    // Delegate straight to the same commands the dashboard
                    // buttons call — Switch/Interrupt with a blank name auto-
                    // assigns "Anchor N" (see `commands::name_or_default`), so
                    // the hotkey now actually starts tracking immediately
                    // instead of just focusing the dashboard's name field.
                    // Each command emits `state-changed` itself on success.
                    let app_handle = app.clone();
                    let state = app.state::<AppState>();
                    let result = match action {
                        HotkeyAction::Switch => commands::switch(app_handle, state, String::new(), None, None),
                        HotkeyAction::Interrupt => commands::interrupt(app_handle, state, String::new(), None, None),
                        HotkeyAction::ReturnPrevious => commands::return_previous(app_handle, state),
                        HotkeyAction::ReturnOriginal => commands::return_original(app_handle, state),
                        HotkeyAction::Complete => commands::complete(app_handle, state),
                    };
                    if let Err(e) = result {
                        eprintln!("hotkey action failed: {e}");
                    }
                })
                .build(),
        )
        .setup(|app| {
            let handle = app.handle();
            let log_path = paths::log_file_path(handle)?;
            let (state, report) = AppState::init(&log_path)?;
            // No dedicated UI surface for either signal yet — at minimum, don't
            // lose them silently.
            if report.torn_line_discarded {
                eprintln!(
                    "warning: a torn/corrupt trailing line was found in {} and discarded on startup",
                    log_path.display()
                );
            }
            if report.startup_gap_recovered {
                eprintln!("info: an active task left over from the last run was closed as recovered-gap on startup");
            }
            app.manage(state);

            let settings_path = paths::settings_file_path(handle)?;
            let bindings = HotkeyBindings::load(&settings_path);
            // Persist on first run (or after falling back from a corrupt file)
            // so the settings file always reflects what's actually active.
            if let Err(e) = bindings.save(&settings_path) {
                eprintln!("warning: could not persist hotkey settings to {}: {e}", settings_path.display());
            }

            let (registered, failures) = hotkeys::register_bindings(handle, &bindings);
            for (action, message) in &failures {
                eprintln!("warning: failed to register hotkey for {}: {message}", action.label());
            }
            let hotkey_state = HotkeyState::new(bindings, settings_path);
            *hotkey_state.registered.lock().unwrap() = registered;
            app.manage(hotkey_state);

            let templates_path = paths::templates_file_path(handle)?;
            app.manage(TemplateState::init(templates_path));

            let export_settings_path = paths::export_settings_file_path(handle)?;
            app.manage(ExportSettingsState::init(export_settings_path));

            let heartbeat_handle = handle.clone();
            std::thread::spawn(move || heartbeat::run(heartbeat_handle));

            let power_handle = handle.clone();
            std::thread::spawn(move || power::run(power_handle));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::switch,
            commands::interrupt,
            commands::rename_active,
            commands::return_previous,
            commands::return_original,
            commands::complete,
            commands::get_state,
            commands::create_template,
            commands::update_template,
            commands::delete_template,
            commands::list_templates,
            commands::get_export_settings,
            commands::update_export_settings,
            commands::export_xlsx,
            commands::export_json,
            commands::get_hotkey_bindings,
            commands::update_hotkey_bindings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
