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
/// whether to resolve a gap — and if so, the single transition to apply: close
/// the active entry as `recovered-gap`. The threshold is inclusive (>=): a gap
/// of exactly the threshold counts.
///
/// Returns ONE payload, not a pair. It used to return `(RecoverGap, Start)`;
/// the return type was collapsed deliberately when ADR 0005 open item 9 removed
/// the auto-resume, so the second transition cannot be reintroduced by accident
/// — there is no longer anywhere to put it.
pub fn resolve_resume_gap(
    active: Option<&TimeBlock>,
    last_activity_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<TransitionPayload> {
    active?;
    if now - last_activity_at < ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS) {
        return None;
    }
    Some(TransitionPayload::RecoverGap { inferred_end: last_activity_at })
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
    let Some(recover_payload) = decision else {
        return;
    };

    // One transition, then done. Nothing is started: the user resumes
    // deliberately (ADR 0005 open item 9). Per
    // `docs/product/features/interruption-stack.md`, no prompt interrupts them
    // at wake either — the closed entry is surfaced in the dashboard whenever
    // they next open it.
    match apply_transition(&state, |_| recover_payload.clone()) {
        Ok(view) => emit_state_changed(app, &view),
        Err(e) => eprintln!("resume-gap recovery failed: {e}"),
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
        TimeBlock::new(name.to_string(), None, None, start, 0)
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
    fn resolves_to_recover_gap_alone_when_gap_exceeds_threshold() {
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS + 60);
        let block = active_block("A", now - ChronoDuration::seconds(600));

        let recover = resolve_resume_gap(Some(&block), last_activity, now).unwrap();

        match recover {
            TransitionPayload::RecoverGap { inferred_end } => assert_eq!(inferred_end, last_activity),
            other => panic!("expected RecoverGap, got {other:?}"),
        }
    }

    /// Regression for ADR 0005 open item 9. Wake used to emit `(RecoverGap,
    /// Start)`, asserting both that the user resumed and when — inventing a
    /// start time Anchor cannot know (`docs/principles.md` #3). The return type
    /// is now a single payload precisely so the second transition has nowhere
    /// to live, but assert the *behaviour* too: a wake must never produce a
    /// Start, whatever the shape of the signature later becomes.
    #[test]
    fn wake_never_auto_starts_a_task() {
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS + 3600);
        let block = active_block("A", now - ChronoDuration::seconds(7200));

        let decision = resolve_resume_gap(Some(&block), last_activity, now).unwrap();

        assert!(
            !matches!(decision, TransitionPayload::Start { .. }),
            "wake must close the gap and start nothing — the user resumes deliberately"
        );
    }

    /// Wake and crash are the same class of event, so they must resolve the
    /// same way: `state::AppState::init` closes the active entry as a recovered
    /// gap and starts nothing. Pins that the two paths agree.
    #[test]
    fn wake_resolves_the_same_way_startup_recovery_does() {
        let now = Utc::now();
        let last_activity = now - ChronoDuration::seconds(RESUME_GAP_THRESHOLD_SECS + 10);
        let block = active_block("A", now - ChronoDuration::seconds(600));

        let decision = resolve_resume_gap(Some(&block), last_activity, now).unwrap();

        assert!(
            matches!(decision, TransitionPayload::RecoverGap { .. }),
            "exactly the transition AppState::init applies on a crashed restart"
        );
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
