//! What to do when Anchor loses continuity — the single shared rule behind both
//! recovery paths, per [ADR 0007](../../../docs/decisions/0007-auto-resume-after-a-short-gap.md).
//!
//! A gap is any stretch where Anchor was not running or not awake while a Time
//! Block was active: a crash, a kill, a sleep, a hibernate. Two callers detect
//! it — `state::AppState::init` at startup and `power.rs` on resume — and
//! before ADR 0007 they disagreed about what to do, which is why this module
//! exists.
//!
//! **The disagreement was real and shipped.** `power.rs` had a 90-second grace
//! window below which a wake produced no transition at all; `AppState::init`
//! had no equivalent and closed any leftover block however brief the outage. So
//! a 13-second sleep-wake was a non-event while a 13-second crash-relaunch
//! closed the block — and because no heartbeat lands in 13 seconds, the last
//! durable write *was* the start, producing a zero-duration block. Verification
//! run 1 hit exactly that. ADR 0005 had claimed to remove precisely this
//! wake-versus-crash anomaly; it removed one half of it.
//!
//! Both callers now ask this module, so they cannot drift apart again.

use crate::model::{TimeBlock, TransitionPayload};
use chrono::{DateTime, Duration, Utc};

/// Below this, assume nothing happened and leave the block alone.
///
/// Keeps `power.rs`'s original value and its original justification: safely
/// past the 60-second heartbeat interval, so ordinary scheduling jitter cannot
/// look like a gap. ADR 0007 extends it to the startup path, where its absence
/// was what produced zero-duration blocks.
pub const CONTINUITY_THRESHOLD_SECS: i64 = 90;

/// Above this, close the block and let the user resume deliberately.
///
/// **Unvalidated — a plausible default, not a measured one** ([`principles.md`](../../../docs/principles.md) #7),
/// the same framing ADR 0004's `N = 500` and the 60-second heartbeat use.
/// **Revisit if** phantom resumed blocks turn out to be common in practice
/// (lower it), or if manually restarting after a lunch-length outage stays
/// annoying (raise it).
pub const RESUME_LIMIT_SECS: i64 = 60 * 60;

/// What the caller should do about a detected gap.
///
/// Returning a decision rather than applying it keeps this pure and testable
/// without a real crash or a real sleep cycle — the same reason `power.rs`'s
/// predecessor was written this way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapResolution {
    /// Nothing to do. Either nothing was active, or the gap is short enough to
    /// treat as continuous.
    Continue,
    /// Close the block at its last known-alive point, and start a fresh block
    /// for the same work at the moment of recovery.
    ///
    /// **The gap itself is never counted as work.** The new block starts at
    /// recovery time, not at `inferred_end`, so the timeline keeps an honest
    /// hole where Anchor was not running. Resuming *from* the inferred end was
    /// considered and rejected in ADR 0007 as the one option that can silently
    /// inflate a billed total.
    RecoverAndResume {
        inferred_end: DateTime<Utc>,
        name: String,
        project: Option<String>,
        client: Option<String>,
    },
    /// Close the block and stop. The outage was long enough that Anchor should
    /// not guess the user came back to the same task.
    RecoverOnly { inferred_end: DateTime<Utc> },
}

/// Decide what a gap means, given the currently active block (if any), when the
/// last durable write happened, and the present moment.
///
/// `now` is supplied by the caller rather than read here so the rule stays pure
/// and independently testable. Both callers pass a live clock; neither replays
/// through this function — the *transitions* it produces are what get logged
/// and replayed, so replay never re-decides.
pub fn resolve(
    active: Option<&TimeBlock>,
    last_activity_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> GapResolution {
    let Some(block) = active else {
        return GapResolution::Continue;
    };

    let gap = now - last_activity_at;
    if gap < Duration::seconds(CONTINUITY_THRESHOLD_SECS) {
        return GapResolution::Continue;
    }
    if gap < Duration::seconds(RESUME_LIMIT_SECS) {
        return GapResolution::RecoverAndResume {
            inferred_end: last_activity_at,
            name: block.name.clone(),
            project: block.project.clone(),
            client: block.client.clone(),
        };
    }
    GapResolution::RecoverOnly { inferred_end: last_activity_at }
}

impl GapResolution {
    /// The transitions to append, in order. Empty for `Continue`.
    ///
    /// The resumed `Start` carries no explicit time: like every live capture it
    /// takes the moment its own transition is logged, which is what keeps the
    /// gap out of the record.
    pub fn transitions(&self) -> Vec<TransitionPayload> {
        match self {
            Self::Continue => Vec::new(),
            Self::RecoverOnly { inferred_end } => {
                vec![TransitionPayload::RecoverGap { inferred_end: *inferred_end }]
            }
            Self::RecoverAndResume { inferred_end, name, project, client } => vec![
                TransitionPayload::RecoverGap { inferred_end: *inferred_end },
                TransitionPayload::Start {
                    name: name.clone(),
                    project: project.clone(),
                    client: client.clone(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(offset_secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + offset_secs, 0).unwrap()
    }

    fn active(name: &str) -> TimeBlock {
        TimeBlock::new(name.into(), Some("Proj".into()), Some("Client".into()), t(0), 7)
    }

    #[test]
    fn nothing_active_means_nothing_to_do() {
        assert_eq!(resolve(None, t(0), t(100_000)), GapResolution::Continue);
    }

    /// The zone that used to exist on the wake path only. Its absence at
    /// startup is what produced verification run 1's zero-duration block.
    #[test]
    fn a_gap_under_the_continuity_threshold_is_not_a_gap_at_all() {
        let b = active("A");
        for elapsed in [0, 1, 13, CONTINUITY_THRESHOLD_SECS - 1] {
            assert_eq!(
                resolve(Some(&b), t(0), t(elapsed)),
                GapResolution::Continue,
                "a {elapsed}s outage must leave the block untouched"
            );
        }
    }

    #[test]
    fn the_continuity_threshold_is_inclusive_so_exactly_90s_is_a_gap() {
        let b = active("A");
        assert!(matches!(
            resolve(Some(&b), t(0), t(CONTINUITY_THRESHOLD_SECS)),
            GapResolution::RecoverAndResume { .. }
        ));
    }

    #[test]
    fn a_short_gap_closes_the_block_and_resumes_the_same_work() {
        let b = active("Weiterentwicklung");
        let r = resolve(Some(&b), t(0), t(300));
        assert_eq!(
            r,
            GapResolution::RecoverAndResume {
                inferred_end: t(0),
                name: "Weiterentwicklung".into(),
                project: Some("Proj".into()),
                client: Some("Client".into()),
            },
            "identity is carried over verbatim"
        );

        let tx = r.transitions();
        assert_eq!(tx.len(), 2, "close, then start — in that order");
        assert!(matches!(tx[0], TransitionPayload::RecoverGap { .. }));
        assert!(matches!(tx[1], TransitionPayload::Start { .. }));
    }

    /// **The gap is never counted as work.** The resumed `Start` carries no
    /// explicit time, so it takes the moment it is logged rather than the
    /// inferred end — leaving an honest hole where Anchor was not running.
    /// Resuming from the inferred end was rejected in ADR 0007 as the only
    /// option that can silently inflate a billed total.
    #[test]
    fn the_resumed_start_carries_no_time_so_the_gap_is_never_billed() {
        let b = active("A");
        let tx = resolve(Some(&b), t(0), t(300)).transitions();
        match &tx[1] {
            TransitionPayload::Start { name, .. } => assert_eq!(name, "A"),
            other => panic!("expected Start, got {other:?}"),
        }
        // The payload has no start field at all — the gap cannot be billed
        // because there is nowhere to express it.
    }

    #[test]
    fn a_long_gap_closes_without_resuming() {
        let b = active("A");
        for elapsed in [RESUME_LIMIT_SECS, RESUME_LIMIT_SECS + 1, 86_400] {
            assert_eq!(
                resolve(Some(&b), t(0), t(elapsed)),
                GapResolution::RecoverOnly { inferred_end: t(0) },
                "a {elapsed}s outage must not guess the user came back"
            );
        }
        let tx = resolve(Some(&b), t(0), t(86_400)).transitions();
        assert_eq!(tx.len(), 1, "close only");
    }

    #[test]
    fn the_resume_limit_is_exclusive_so_one_second_under_still_resumes() {
        let b = active("A");
        assert!(matches!(
            resolve(Some(&b), t(0), t(RESUME_LIMIT_SECS - 1)),
            GapResolution::RecoverAndResume { .. }
        ));
    }

    /// The whole point of this module: one rule, so startup and wake cannot
    /// disagree again. Asserted directly rather than left to the two callers.
    #[test]
    fn the_same_gap_resolves_identically_regardless_of_which_path_asks() {
        let b = active("A");
        for elapsed in [0, 45, 90, 300, 3_599, 3_600, 100_000] {
            let from_startup = resolve(Some(&b), t(0), t(elapsed));
            let from_wake = resolve(Some(&b), t(0), t(elapsed));
            assert_eq!(from_startup, from_wake, "the rule cannot depend on the caller");
        }
    }
}
