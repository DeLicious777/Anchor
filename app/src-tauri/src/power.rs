//! Live sleep/hibernate detection via a hidden message-only window handling
//! `WM_POWERBROADCAST`. Runs on its own dedicated OS thread with its own
//! message loop — Tauri's own event loop (on the main thread) never sees this.
//!
//! Distinct from startup-based recovered-gap (`state::AppState::init`): this
//! handles the SAME running process waking from sleep, so per the confirmed
//! design (see `model::TransitionPayload::RecoverGap` docs), it auto-resumes
//! the same task identity — a restart does not.

use crate::commands::{apply_transition, emit_state_changed};
use crate::model::{TimeBlock, TransitionPayload};
use crate::state::AppState;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

/// Safely past the 60s heartbeat interval, to avoid false positives from
/// ordinary scheduling jitter.
pub const RESUME_GAP_THRESHOLD_SECS: i64 = 90;

/// Pure decision logic, independent of the actual `WM_POWERBROADCAST` wiring so
/// it's unit-testable without a real sleep/wake cycle. Given the currently
/// active entry (if any) and how long ago the last durable write was, decide
/// whether to resolve a gap — and if so, the two transitions to apply in
/// order: close as `recovered-gap`, then auto-resume the same identity. The
/// threshold is inclusive (>=): a gap of exactly the threshold counts.
pub fn resolve_resume_gap(
    active: Option<&TimeBlock>,
    last_activity_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<(TransitionPayload, TransitionPayload)> {
    let active = active?;
    if now - last_activity_at < ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS) {
        return None;
    }
    let recover = TransitionPayload::RecoverGap { inferred_end: last_activity_at };
    let resume = TransitionPayload::Start {
        name: active.name.clone(),
        project: active.project.clone(),
        client: active.client.clone(),
    };
    Some((recover, resume))
}

/// The AppHandle reachable from the raw WndProc — there is only ever one such
/// window for the app's lifetime, so a global slot is the simplest correct
/// option (the alternative, stashing a pointer via `GWLP_USERDATA`, adds real
/// complexity for the same single-instance guarantee).
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

fn handle_resume(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let now = Utc::now();
    let decision = {
        let inner = state.inner.lock().unwrap();
        resolve_resume_gap(inner.stack.active.as_ref(), inner.last_activity_at, now)
    };
    let Some((recover_payload, resume_payload)) = decision else {
        return;
    };

    match apply_transition(&state, |_| recover_payload.clone()) {
        Ok(view) => emit_state_changed(app, &view),
        Err(e) => {
            eprintln!("resume-gap recovery (close) failed: {e}");
            return;
        }
    }
    match apply_transition(&state, |_| resume_payload.clone()) {
        Ok(view) => emit_state_changed(app, &view),
        Err(e) => eprintln!("resume-gap recovery (auto-resume) failed: {e}"),
    }
}

#[cfg(target_os = "windows")]
mod win {
    use super::{handle_resume, APP_HANDLE};
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW, TranslateMessage,
        CW_USEDEFAULT, HWND_MESSAGE, MSG, WINDOW_EX_STYLE, WM_POWERBROADCAST, WNDCLASSEXW, WS_OVERLAPPED,
    };

    // Documented, stable Win32 constants (winuser.h) — referenced by literal
    // value since exposing them varies by `windows` crate feature selection.
    const PBT_APMRESUMESUSPEND: u32 = 0x0007;
    const PBT_APMRESUMEAUTOMATIC: u32 = 0x0012;

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if msg == WM_POWERBROADCAST {
            let event = wparam.0 as u32;
            if event == PBT_APMRESUMEAUTOMATIC || event == PBT_APMRESUMESUSPEND {
                if let Some(app) = APP_HANDLE.get() {
                    handle_resume(app);
                }
            }
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// Runs forever on its own thread: creates a hidden message-only window and
    /// pumps its message loop. `app` is stashed once globally (see
    /// `APP_HANDLE`) since the raw `WndProc` has no way to capture a closure.
    pub fn run(app: tauri::AppHandle) {
        let _ = APP_HANDLE.set(app);

        unsafe {
            let class_name: PCWSTR = w!("AnchorPowerBroadcastWindow");
            let instance = match GetModuleHandleW(None) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("warning: GetModuleHandleW failed ({e}) — live sleep/hibernate detection is disabled this run");
                    return;
                }
            };

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(wndproc),
                hInstance: instance.into(),
                lpszClassName: class_name,
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                PCWSTR::null(),
                WS_OVERLAPPED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                Some(HWND_MESSAGE),
                None,
                Some(instance.into()),
                None,
            );

            let Ok(hwnd) = hwnd else {
                eprintln!("warning: could not create power-broadcast message window — live sleep/hibernate detection is disabled this run");
                return;
            };
            let _ = hwnd; // kept alive implicitly by the OS for the process lifetime

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub use win::run;

#[cfg(not(target_os = "windows"))]
pub fn run(_app: tauri::AppHandle) {
    eprintln!("live sleep/hibernate detection is only implemented for Windows");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_block(name: &str, start: DateTime<Utc>) -> TimeBlock {
        TimeBlock::new(name.to_string(), None, None, start)
    }

    #[test]
    fn no_gap_when_nothing_active() {
        let now = Utc::now();
        assert!(resolve_resume_gap(None, now, now).is_none());
    }

    #[test]
    fn no_gap_when_last_activity_is_recent() {
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(30); // well under threshold
        let block = active_block("A", now - ChronoDuration::seconds(60));
        assert!(resolve_resume_gap(Some(&block), last_activity, now).is_none());
    }

    #[test]
    fn resolves_recover_gap_then_start_when_gap_exceeds_threshold() {
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS + 60);
        let block = active_block("A", now - ChronoDuration::seconds(600));

        let (recover, resume) = resolve_resume_gap(Some(&block), last_activity, now).unwrap();

        match recover {
            TransitionPayload::RecoverGap { inferred_end } => assert_eq!(inferred_end, last_activity),
            other => panic!("expected RecoverGap, got {other:?}"),
        }
        match resume {
            TransitionPayload::Start { name, .. } => assert_eq!(name, "A", "auto-resume must reuse the same identity"),
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn boundary_at_exactly_the_threshold_counts_as_a_gap() {
        // Inclusive boundary: >= threshold, not strictly >. A gap of exactly
        // RESUME_GAP_THRESHOLD_SECS is just as much "at least that stale" as
        // one a second longer — there's no reason to special-case equality.
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS);
        let block = active_block("A", now - ChronoDuration::seconds(600));
        assert!(resolve_resume_gap(Some(&block), last_activity, now).is_some());
    }

    #[test]
    fn boundary_one_second_under_threshold_is_not_a_gap() {
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS - 1);
        let block = active_block("A", now - ChronoDuration::seconds(600));
        assert!(resolve_resume_gap(Some(&block), last_activity, now).is_none());
    }
}
