pub mod commands;
pub mod heartbeat;
pub mod log;
pub mod model;
pub mod paths;
pub mod power;
pub mod settings;
pub mod stack;
pub mod state;

use commands::{apply_transition, emit_state_changed};
use model::TransitionPayload;
use settings::HotkeyBindings;
use state::AppState;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Clone, Copy, Debug)]
enum HotkeyAction {
    Switch,
    Interrupt,
    ReturnPrevious,
    ReturnOriginal,
    Complete,
}

/// Which action each successfully-registered shortcut maps to. Populated once
/// in `.setup()`, read-only afterward — the handler below only reads it.
struct RegisteredHotkeys(Vec<(Shortcut, HotkeyAction)>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let Some(registered) = app.try_state::<RegisteredHotkeys>() else {
                        return;
                    };
                    let Some((_, action)) = registered.0.iter().find(|(s, _)| s == shortcut) else {
                        return;
                    };

                    match action {
                        HotkeyAction::Switch | HotkeyAction::Interrupt => {
                            // Deliberately not a dedicated quick-input popup this
                            // slice — bring the dashboard forward with its name
                            // field focused, reusing the existing form.
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("focus-name-input", ());
                            }
                        }
                        HotkeyAction::ReturnPrevious | HotkeyAction::ReturnOriginal | HotkeyAction::Complete => {
                            let Some(state) = app.try_state::<AppState>() else {
                                return;
                            };
                            let payload = match action {
                                HotkeyAction::ReturnPrevious => TransitionPayload::ReturnPrevious,
                                HotkeyAction::ReturnOriginal => TransitionPayload::ReturnOriginal,
                                HotkeyAction::Complete => TransitionPayload::Complete,
                                HotkeyAction::Switch | HotkeyAction::Interrupt => unreachable!(),
                            };
                            match apply_transition(&state, |_| payload.clone()) {
                                Ok(view) => emit_state_changed(app, &view),
                                Err(e) => eprintln!("hotkey action failed: {e}"),
                            }
                        }
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

            let mut registered = Vec::new();
            for (accelerator, action) in [
                (&bindings.switch, HotkeyAction::Switch),
                (&bindings.interrupt, HotkeyAction::Interrupt),
                (&bindings.return_previous, HotkeyAction::ReturnPrevious),
                (&bindings.return_original, HotkeyAction::ReturnOriginal),
                (&bindings.complete, HotkeyAction::Complete),
            ] {
                match accelerator.parse::<Shortcut>() {
                    Ok(shortcut) => match app.global_shortcut().register(shortcut) {
                        Ok(()) => registered.push((shortcut, action)),
                        Err(e) => eprintln!("warning: failed to register hotkey {accelerator:?} ({action:?}): {e}"),
                    },
                    Err(e) => eprintln!("warning: invalid hotkey accelerator {accelerator:?}: {e}"),
                }
            }
            app.manage(RegisteredHotkeys(registered));

            let heartbeat_handle = handle.clone();
            std::thread::spawn(move || heartbeat::run(heartbeat_handle));

            let power_handle = handle.clone();
            std::thread::spawn(move || power::run(power_handle));

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
