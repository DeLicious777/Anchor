//! Live sleep/hibernate detection via a hidden message-only window handling
//! `WM_POWERBROADCAST`. Runs on its own dedicated OS thread with its own
//! message loop — Tauri's own event loop (on the main thread) never sees this.
//!
//! Handled IDENTICALLY to startup-based recovered-gap (`state::AppState::init`):
//! close the active entry with an inferred end, and start nothing. Wake and
//! crash are the same class of event — Anchor lost continuity and cannot know
//! what happened in the gap — so they get the same handling.
//!
//! This changed on 2026-07-29 (ADR 0005 open item 9). Wake used to also emit a
//! `Start` with the same identity, on the reasoning that this is "the SAME
//! running process" so the task identity is still known. That is true and
//! beside the point: knowing WHICH task was active is not knowing THAT the user
//! resumed it, nor WHEN. Someone who wakes a laptop to check the time has not
//! gone back to work. Anchor was inventing a start time it could not know —
//! `docs/principles.md` #3.
//!
//! The user now resumes deliberately, via the capture action. Note the gap this
//! does NOT close: `RecoverGap` closes the active block without pushing a stack
//! frame, so no return path survives a wake or a crash. Pre-existing, symmetric
//! across both paths, and owned by Pause's design work (issue #16).

use crate::commands::{apply_transition, emit_state_changed};
use crate::state::AppState;
use chrono::Utc;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

/// Re-exported for the tests and callers that still name it. The rule itself
/// now lives in `crate::gap`, shared with `state::AppState::init` — see
/// [ADR 0007](../../../docs/decisions/0007-auto-resume-after-a-short-gap.md).
/// Before that, this path and the startup path disagreed about short gaps.
pub use crate::gap::CONTINUITY_THRESHOLD_SECS as RESUME_GAP_THRESHOLD_SECS;

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
    let resolution = {
        let inner = state.inner.lock().unwrap();
        crate::gap::resolve(inner.stack.active.as_ref(), inner.last_activity_at, now)
    };

    // Whatever the shared rule says, applied in order. A short gap yields
    // RecoverGap + Start (ADR 0007); a long one yields RecoverGap alone; a very
    // short one yields nothing at all. No prompt interrupts the user at wake
    // either way — `docs/product/features/interruption-stack.md`.
    for payload in resolution.transitions() {
        match apply_transition(&state, |_| payload.clone()) {
            Ok(view) => emit_state_changed(app, &view),
            // Stop on the first failure rather than pressing on: a Start whose
            // preceding RecoverGap did not commit would be applied against a
            // block that is still active, and rejected anyway.
            Err(e) => {
                eprintln!("resume-gap recovery failed: {e}");
                return;
            }
        }
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
    //! **These tests moved to `crate::gap`, they were not dropped.**
    //!
    //! Every assertion that used to live here was about the *decision* — is
    //! this a gap, does the boundary count, what transition results — and that
    //! decision now lives in one shared place so this path and startup cannot
    //! disagree (ADR 0007). `gap.rs` covers all of it, plus the zone neither
    //! path tested before: a gap short enough to ignore entirely.
    //!
    //! One test genuinely changed rather than moved.
    //! `wake_never_auto_starts_a_task` asserted ADR 0005 open item 9, which
    //! **ADR 0007 supersedes**: a wake inside the resume limit now does emit a
    //! `Start`. It is not weakened, it is inverted, and its replacement is
    //! `gap::tests::a_short_gap_closes_the_block_and_resumes_the_same_work`.
    //! The old test was right for the old decision.
    //!
    //! What is left in this module is the wiring — `handle_resume` needs a live
    //! `AppHandle` and a real `WM_POWERBROADCAST`, so it is exercised by the
    //! manual pass (`docs/verification-checklist.md` step 4), not from here.

    use super::*;

    /// The one thing still worth pinning locally: this module must keep
    /// deferring to the shared rule rather than reintroducing a threshold of
    /// its own. That divergence is exactly what ADR 0007 was written to fix.
    #[test]
    fn the_wake_threshold_is_the_shared_one() {
        assert_eq!(RESUME_GAP_THRESHOLD_SECS, crate::gap::CONTINUITY_THRESHOLD_SECS);
    }
}
