//! Global hotkey registration, shared between startup (`lib.rs`) and the live
//! remap command (`commands::update_hotkey_bindings`).
//!
//! `register_bindings` is best-effort per binding — an invalid accelerator or
//! an OS-level conflict on one action must never prevent the other four from
//! registering, matching the original startup behavior. The remap path
//! (`apply_remap`) builds atomicity on top of that: if any of the five new
//! bindings fails, everything attempted in that call is unregistered and the
//! previous, already-working bindings are restored — a failed remap must
//! never leave the user with fewer working hotkeys than before.

use crate::settings::HotkeyBindings;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Switch,
    Interrupt,
    ReturnPrevious,
    ReturnOriginal,
    Complete,
}

impl HotkeyAction {
    pub const ALL: [HotkeyAction; 5] = [
        HotkeyAction::Switch,
        HotkeyAction::Interrupt,
        HotkeyAction::ReturnPrevious,
        HotkeyAction::ReturnOriginal,
        HotkeyAction::Complete,
    ];

    pub fn accelerator(self, bindings: &HotkeyBindings) -> String {
        match self {
            HotkeyAction::Switch => bindings.switch.clone(),
            HotkeyAction::Interrupt => bindings.interrupt.clone(),
            HotkeyAction::ReturnPrevious => bindings.return_previous.clone(),
            HotkeyAction::ReturnOriginal => bindings.return_original.clone(),
            HotkeyAction::Complete => bindings.complete.clone(),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            HotkeyAction::Switch => "Switch",
            HotkeyAction::Interrupt => "Interrupt",
            HotkeyAction::ReturnPrevious => "Return to Previous",
            HotkeyAction::ReturnOriginal => "Return to Original",
            HotkeyAction::Complete => "Complete",
        }
    }
}

/// `registered` is the live, actually-registered set (read by the shortcut
/// event handler); `bindings` is the last known-good configuration (used to
/// populate the settings UI and to roll back to on a failed remap). The two
/// are kept in lockstep by `apply_remap` — they only ever diverge mid-call,
/// never at rest.
pub struct HotkeyState {
    pub registered: Mutex<Vec<(Shortcut, HotkeyAction)>>,
    pub bindings: Mutex<HotkeyBindings>,
    path: PathBuf,
}

impl HotkeyState {
    pub fn new(bindings: HotkeyBindings, path: PathBuf) -> Self {
        Self { registered: Mutex::new(Vec::new()), bindings: Mutex::new(bindings), path }
    }
}

/// Registers every binding independently. Returns whatever actually
/// succeeded, plus every failure with the action it belongs to — the caller
/// decides whether a failure is fatal (startup just logs it; remap treats any
/// failure as reason to roll back the whole attempt).
pub fn register_bindings(
    app: &AppHandle,
    bindings: &HotkeyBindings,
) -> (Vec<(Shortcut, HotkeyAction)>, Vec<(HotkeyAction, String)>) {
    let mut registered = Vec::new();
    let mut failures = Vec::new();
    for action in HotkeyAction::ALL {
        let accelerator = action.accelerator(bindings);
        match accelerator.parse::<Shortcut>() {
            Ok(shortcut) => match app.global_shortcut().register(shortcut) {
                Ok(()) => registered.push((shortcut, action)),
                Err(e) => failures.push((
                    action,
                    format!("{accelerator:?} could not be registered — likely already bound to another app or Anchor action ({e})"),
                )),
            },
            Err(e) => failures.push((action, format!("{accelerator:?} is not a valid hotkey ({e})"))),
        }
    }
    (registered, failures)
}

pub fn unregister_all(app: &AppHandle, registered: &[(Shortcut, HotkeyAction)]) {
    for (shortcut, _) in registered {
        let _ = app.global_shortcut().unregister(*shortcut);
    }
}

/// Two actions can never share one accelerator — the runtime handler
/// dispatches on the first exact shortcut match, so a shared binding would
/// silently shadow one of the two actions rather than failing loudly.
fn find_duplicate(bindings: &HotkeyBindings) -> Option<(HotkeyAction, HotkeyAction, String)> {
    let pairs: Vec<(HotkeyAction, String)> =
        HotkeyAction::ALL.iter().map(|a| (*a, a.accelerator(bindings))).collect();
    for i in 0..pairs.len() {
        for j in (i + 1)..pairs.len() {
            if pairs[i].1.eq_ignore_ascii_case(&pairs[j].1) {
                return Some((pairs[i].0, pairs[j].0, pairs[i].1.clone()));
            }
        }
    }
    None
}

/// The live-remap entry point: atomic, unlike startup registration. Either
/// all five new bindings take effect and are persisted, or none of them do
/// and the previous bindings are restored.
pub fn apply_remap(app: &AppHandle, state: &HotkeyState, new_bindings: HotkeyBindings) -> Result<HotkeyBindings, String> {
    if let Some((a, b, accelerator)) = find_duplicate(&new_bindings) {
        return Err(format!("{} and {} can't both be bound to {accelerator}", a.label(), b.label()));
    }

    let mut registered = state.registered.lock().map_err(|_| "hotkey state lock poisoned".to_string())?;
    let mut current_bindings = state.bindings.lock().map_err(|_| "hotkey state lock poisoned".to_string())?;

    unregister_all(app, &registered);

    let (new_registered, mut failures) = register_bindings(app, &new_bindings);
    if let Some((action, message)) = failures.drain(..).next() {
        unregister_all(app, &new_registered);
        let (restored, _) = register_bindings(app, &current_bindings);
        *registered = restored;
        return Err(format!("{}: {message}", action.label()));
    }

    if let Err(e) = new_bindings.save(&state.path) {
        unregister_all(app, &new_registered);
        let (restored, _) = register_bindings(app, &current_bindings);
        *registered = restored;
        return Err(format!("could not save hotkey settings: {e}"));
    }

    *registered = new_registered;
    *current_bindings = new_bindings.clone();
    Ok(new_bindings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(switch: &str, interrupt: &str, return_previous: &str, return_original: &str, complete: &str) -> HotkeyBindings {
        HotkeyBindings {
            switch: switch.to_string(),
            interrupt: interrupt.to_string(),
            return_previous: return_previous.to_string(),
            return_original: return_original.to_string(),
            complete: complete.to_string(),
        }
    }

    #[test]
    fn no_duplicate_found_among_distinct_defaults() {
        assert!(find_duplicate(&HotkeyBindings::default()).is_none());
    }

    #[test]
    fn duplicate_accelerator_across_two_actions_is_detected() {
        let b = bindings("Ctrl+Alt+S", "Ctrl+Alt+S", "Ctrl+Alt+P", "Ctrl+Alt+O", "Ctrl+Alt+C");
        let (a, c, accelerator) = find_duplicate(&b).expect("duplicate must be detected");
        assert_eq!(a, HotkeyAction::Switch);
        assert_eq!(c, HotkeyAction::Interrupt);
        assert_eq!(accelerator, "Ctrl+Alt+S");
    }

    #[test]
    fn duplicate_detection_is_case_insensitive() {
        let b = bindings("Ctrl+Alt+S", "ctrl+alt+s", "Ctrl+Alt+P", "Ctrl+Alt+O", "Ctrl+Alt+C");
        assert!(find_duplicate(&b).is_some());
    }
}
