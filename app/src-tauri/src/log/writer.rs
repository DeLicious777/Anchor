//! Appends transitions to the on-disk log, per ADR 0004: durable-before-committed.
//!
//! `append()` assigns the sequence number, encodes the line, writes it, and
//! fsyncs — all before returning `Ok`. The caller (see `commands.rs`) must only
//! mutate the in-memory `InterruptionStack` after `append()` succeeds; that
//! ordering is the concrete mechanism behind "a transition is durably committed
//! before it's considered done."

use crate::log::checksum::encode_line;
use crate::model::{TransitionPayload, TransitionRecord};
use chrono::Utc;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct LogWriter {
    path: PathBuf,
    file: File,
    next_seq: u64,
}

impl LogWriter {
    /// `next_seq` is the sequence number the next appended line should use —
    /// callers determine this from `log::reader::replay()` at startup (one past
    /// whatever the last valid line's `seq` was, or 0 for a fresh log), so the
    /// writer never needs to re-parse the file itself.
    pub fn open(path: impl AsRef<Path>, next_seq: u64) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { path, file, next_seq })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Append one transition. Returns the fully-assigned record on success —
    /// the caller applies it to the in-memory stack only after this returns Ok.
    pub fn append(&mut self, payload: TransitionPayload) -> std::io::Result<TransitionRecord> {
        let record = TransitionRecord {
            seq: self.next_seq,
            timestamp: Utc::now(),
            payload,
        };
        let line = encode_line(&record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.file.write_all(line.as_bytes())?;
        self.file.sync_all()?;
        self.next_seq += 1;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::checksum::decode_line;
    use std::io::{BufRead, BufReader};

    fn sample_payload(n: &str) -> TransitionPayload {
        TransitionPayload::Start { name: n.to_string(), project: None, client: None }
    }

    #[test]
    fn append_assigns_increasing_sequence_numbers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let mut writer = LogWriter::open(&path, 0).unwrap();

        let r0 = writer.append(sample_payload("a")).unwrap();
        let r1 = writer.append(sample_payload("b")).unwrap();
        let r2 = writer.append(sample_payload("c")).unwrap();

        assert_eq!(r0.seq, 0);
        assert_eq!(r1.seq, 1);
        assert_eq!(r2.seq, 2);
        assert_eq!(writer.next_seq(), 3);
    }

    #[test]
    fn append_writes_lines_decodable_by_checksum_module() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(sample_payload("a")).unwrap();
        writer.append(sample_payload("b")).unwrap();

        let file = File::open(&path).unwrap();
        let lines: Vec<String> = BufReader::new(file).lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines.len(), 2);
        let decoded: Vec<_> = lines.iter().map(|l| decode_line(l).unwrap()).collect();
        assert_eq!(decoded[0].seq, 0);
        assert_eq!(decoded[1].seq, 1);
    }

    #[test]
    fn reopening_with_a_later_next_seq_continues_from_there() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer.append(sample_payload("a")).unwrap();
        }
        // Simulate a restart: caller replayed the file, determined next_seq = 1.
        let mut writer2 = LogWriter::open(&path, 1).unwrap();
        let r = writer2.append(sample_payload("b")).unwrap();
        assert_eq!(r.seq, 1);

        let file = File::open(&path).unwrap();
        let lines: Vec<String> = BufReader::new(file).lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines.len(), 2, "append must not truncate/overwrite the existing log");
    }
}
