//! Replays the transition log into an `InterruptionStack`, per ADR 0004: the
//! first line that fails checksum or JSON validation is the torn tail — it and
//! everything after it is discarded, every prior line is untouched. Replay is
//! watermark-based (filters by `record.seq > watermark`), not
//! truncation-order-dependent, so correctness never assumes compaction actually
//! ran (see ADR 0004 Consequences).

use crate::log::checksum::decode_line;
use crate::stack::InterruptionStack;
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("I/O error reading transition log: {0}")]
    Io(#[from] std::io::Error),
    #[error("log line {seq} passed checksum/JSON validation but is logically inconsistent: {source}")]
    Inconsistent { seq: u64, #[source] source: crate::stack::StackError },
}

pub struct ReplayResult {
    pub stack: InterruptionStack,
    /// The sequence number the writer should assign to the next appended line.
    pub next_seq: u64,
    /// True if a torn/corrupt trailing line was found and discarded.
    pub torn_line_discarded: bool,
    /// The timestamp of the last successfully-parsed (non-torn) line, whether or
    /// not it was actually applied (a pre-watermark line still counts — it was
    /// still a real, durable write). Used at startup as the inferred end time
    /// for a `recovered-gap` resolution when `stack.active` survives replay.
    pub last_timestamp: Option<DateTime<Utc>>,
}

/// Replay the log at `path`. `watermark`: lines with `seq <= watermark` are
/// skipped (already incorporated into a prior snapshot) but still count toward
/// determining `next_seq`. `starting_stack`: the state a snapshot at `watermark`
/// already reconstructed — required whenever `watermark` is `Some`, since
/// skipping pre-watermark lines means the state they'd otherwise produce has to
/// come from somewhere. Compaction/snapshots aren't implemented this slice, so
/// callers here should pass `(path, None, None)` for a full replay from empty;
/// the parameters exist so Slice 2's compaction work can reuse this function
/// rather than duplicating replay logic.
pub fn replay(
    path: impl AsRef<Path>,
    watermark: Option<u64>,
    starting_stack: Option<InterruptionStack>,
) -> Result<ReplayResult, ReplayError> {
    let path = path.as_ref();
    let mut stack = starting_stack.unwrap_or_default();
    let mut last_seq_seen: Option<u64> = None;
    let mut last_timestamp: Option<DateTime<Utc>> = None;
    let mut torn_line_discarded = false;

    if !path.exists() {
        let next_seq = watermark.map(|w| w + 1).unwrap_or(0);
        return Ok(ReplayResult { stack, next_seq, torn_line_discarded: false, last_timestamp: None });
    }

    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let record = match decode_line(&line) {
            Ok(record) => record,
            Err(_) => {
                // Torn/corrupt tail: stop here, discard this and everything after.
                torn_line_discarded = true;
                break;
            }
        };

        last_seq_seen = Some(record.seq);
        last_timestamp = Some(record.timestamp);

        let already_in_snapshot = watermark.is_some_and(|w| record.seq <= w);
        if !already_in_snapshot {
            stack
                .apply(&record.payload, record.timestamp)
                .map_err(|source| ReplayError::Inconsistent { seq: record.seq, source })?;
        }
    }

    let next_seq_from_lines = last_seq_seen.map(|s| s + 1).unwrap_or(0);
    let next_seq_from_watermark = watermark.map(|w| w + 1).unwrap_or(0);
    let next_seq = next_seq_from_lines.max(next_seq_from_watermark);
    Ok(ReplayResult { stack, next_seq, torn_line_discarded, last_timestamp })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::writer::LogWriter;
    use crate::model::TransitionPayload;
    use std::fs;
    use std::io::Write;

    fn payload(n: &str) -> TransitionPayload {
        TransitionPayload::Start { name: n.to_string(), project: None, client: None }
    }

    #[test]
    fn replay_of_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.jsonl");
        let result = replay(&path, None, None).unwrap();
        assert_eq!(result.next_seq, 0);
        assert!(!result.torn_line_discarded);
        assert!(result.stack.active.is_none());
        assert!(result.last_timestamp.is_none());
    }

    #[test]
    fn last_timestamp_reflects_the_last_good_line_leftover_active_included() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let last_record = {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer.append(payload("A")).unwrap();
            writer
                .append(TransitionPayload::Interrupt { name: "B".into(), project: None, client: None })
                .unwrap()
        };

        let result = replay(&path, None, None).unwrap();
        assert_eq!(result.last_timestamp, Some(last_record.timestamp));
    }

    #[test]
    fn replay_reconstructs_stack_from_a_real_sequence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer.append(payload("A")).unwrap();
            writer
                .append(TransitionPayload::Interrupt { name: "B".into(), project: None, client: None })
                .unwrap();
        }

        let result = replay(&path, None, None).unwrap();
        assert!(!result.torn_line_discarded);
        assert_eq!(result.next_seq, 2);
        assert_eq!(result.stack.stack_depth(), 1);
        assert_eq!(result.stack.active.as_ref().unwrap().name, "B");
        assert_eq!(result.stack.closed[0].name, "A");
        assert_eq!(result.stack.closed[0].completion_reason, None);
    }

    /// The deliberately-corrupted-line test required by the implementation plan:
    /// N good lines + one hand-truncated garbage line -> replay yields exactly N
    /// records, torn_line_discarded == true, and next_seq resumes correctly.
    #[test]
    fn deliberately_corrupted_trailing_line_is_discarded_prior_lines_survive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer.append(payload("A")).unwrap(); // seq 0: Start
            writer
                .append(TransitionPayload::Switch { name: "B".into(), project: None, client: None })
                .unwrap(); // seq 1
            writer
                .append(TransitionPayload::Switch { name: "C".into(), project: None, client: None })
                .unwrap(); // seq 2
        }
        // Simulate a torn write: append a hand-truncated garbage line (as if the
        // process died mid-write of the 4th record) with no valid checksum.
        {
            let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
            file.write_all(b"{\"seq\":3,\"timestamp\":\"2026-01-01T00:00:00Z\",\"type\":\"star").unwrap();
            // Deliberately no tab, no checksum, no newline: a genuine torn write.
        }

        let result = replay(&path, None, None).unwrap();
        assert!(result.torn_line_discarded);
        assert_eq!(result.stack.active.as_ref().unwrap().name, "C", "all 3 good lines replayed correctly before the torn tail stopped things");
        assert_eq!(result.next_seq, 3, "next_seq must resume from the last good record (seq 2), ignoring the torn line entirely");
    }

    /// Proves replay correctness doesn't depend on truncation having physically
    /// happened: replaying the FULL log with a watermark + seeded starting state
    /// must match replaying a PHYSICALLY TRUNCATED copy of the same log seeded
    /// with that same starting state.
    #[test]
    fn watermark_based_replay_matches_replay_of_a_physically_truncated_log() {
        let dir = tempfile::tempdir().unwrap();
        let full_path = dir.path().join("full.jsonl");
        {
            let mut writer = LogWriter::open(&full_path, 0).unwrap();
            writer.append(payload("A")).unwrap(); // seq 0
            writer
                .append(TransitionPayload::Interrupt { name: "B".into(), project: None, client: None })
                .unwrap(); // seq 1
            writer
                .append(TransitionPayload::Interrupt { name: "C".into(), project: None, client: None })
                .unwrap(); // seq 2
        }

        // The state a snapshot at watermark=1 would have reconstructed: Start A,
        // then Interrupt B (i.e. A paused/pending, B active).
        let seed = || {
            let mut s = InterruptionStack::new();
            s.apply(&payload("A"), chrono::Utc::now()).unwrap();
            s.apply(
                &TransitionPayload::Interrupt { name: "B".into(), project: None, client: None },
                chrono::Utc::now(),
            )
            .unwrap();
            s
        };

        // Full replay with a watermark of 1, seeded with that snapshot state —
        // only seq 2 (Interrupt C) actually gets applied on top.
        let watermarked = replay(&full_path, Some(1), Some(seed())).unwrap();

        // Physically-truncated comparison: a log file containing ONLY line 3
        // (seq 2), replayed with no watermark but the same seeded starting state.
        let truncated_path = dir.path().join("truncated.jsonl");
        let full_contents = fs::read_to_string(&full_path).unwrap();
        let third_line = full_contents.lines().nth(2).unwrap();
        fs::write(&truncated_path, format!("{third_line}\n")).unwrap();
        let truncated = replay(&truncated_path, None, Some(seed())).unwrap();

        assert_eq!(watermarked.stack.stack_depth(), truncated.stack.stack_depth());
        assert_eq!(
            watermarked.stack.active.as_ref().map(|b| &b.name),
            truncated.stack.active.as_ref().map(|b| &b.name),
            "replay via watermark over the full log must match replay of a physically truncated log seeded with the same prior state"
        );
        assert_eq!(watermarked.next_seq, 3);
        assert_eq!(truncated.next_seq, 3, "next_seq determination must not depend on whether truncation physically happened");
    }
}
