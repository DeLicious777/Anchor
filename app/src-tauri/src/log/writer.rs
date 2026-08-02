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
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Drop any bytes after the final record boundary, so the next append starts on
/// a fresh line. Returns whether anything was removed.
///
/// **This is what keeps ADR 0004's "a torn write can only affect the last
/// record" true.** Without it, a torn fragment stays on disk with no trailing
/// newline, the next append is concatenated onto it, and the combined line
/// fails its checksum forever — so every transition committed after a torn
/// write is silently lost on the next restart, permanently, and `next_seq`
/// stalls. That was a live data-loss bug (found 2026-07-29); see
/// `log::reader::tests::work_committed_after_a_torn_write_survives_the_next_restart`.
///
/// **Why discarding these bytes is correct, not lossy.** `encode_line` ends
/// every complete record with `\n`, so trailing bytes without one are a write
/// that did not finish. `append` returns `Ok` only after `sync_all`, so such a
/// write was never acknowledged to its caller — `commands::apply_transition`
/// never applied it to the in-memory stack, and the user was never told it
/// succeeded. Discarding it is exactly "not durably committed."
///
/// The one indistinguishable edge — every byte landed except the final newline
/// — resolves the same way for the same reason: the write was still never
/// acknowledged. Erring toward discard can only drop a transition nobody was
/// told about; erring the other way would replay one the user never saw commit.
///
/// **Why not truncate at the first undecodable line instead** (the obvious
/// alternative): in a log already damaged by the bug above, that line sits
/// mid-file with valid, still-recoverable records after it. Truncating there
/// would destroy them. This only ever removes bytes past the last newline, so
/// it cannot reach committed history.
fn truncate_incomplete_tail(path: &Path) -> std::io::Result<bool> {
    let mut file = match OpenOptions::new().read(true).write(true).open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };

    let len = file.metadata()?.len();
    if len == 0 {
        return Ok(false);
    }

    // Fast path: a well-formed log ends with the newline `encode_line` wrote.
    // One seek and one byte, which is the case on every normal startup.
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    if last[0] == b'\n' {
        return Ok(false);
    }

    // Rare path only: locate the last record boundary. Reading the whole file
    // here is acceptable because replay is about to do the same, and because
    // this branch is reached only after an ungraceful shutdown.
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::with_capacity(len as usize);
    file.read_to_end(&mut bytes)?;
    let keep = bytes.iter().rposition(|&b| b == b'\n').map(|i| i as u64 + 1).unwrap_or(0);
    file.set_len(keep)?;
    file.sync_all()?;
    Ok(true)
}

/// Restore the file to exactly `len` bytes and make that durable.
///
/// The rollback half of `append`'s all-or-nothing guarantee. `truncate_incomplete_tail`
/// handles the *startup* case by cutting back to the last newline; this handles the
/// *in-session* case, where cutting to the last newline would be wrong — a failed
/// `sync_all` can leave a line that is complete and newline-terminated, yet was never
/// acknowledged to its caller. Only the caller's recorded pre-append length identifies
/// what to remove.
fn rollback_to(file: &mut File, len: u64) -> std::io::Result<()> {
    file.set_len(len)?;
    file.sync_all()
}

pub struct LogWriter {
    path: PathBuf,
    file: File,
    next_seq: u64,
    /// Set when a rollback itself failed, leaving the file at an unknown length.
    /// Further appends are refused rather than written onto an unknown tail.
    poisoned: bool,
}

impl LogWriter {
    /// `next_seq` is the sequence number the next appended line should use —
    /// callers determine this from `log::reader::replay()` at startup (one past
    /// whatever the last valid line's `seq` was, or 0 for a fresh log), so the
    /// writer never needs to re-parse the file itself.
    pub fn open(path: impl AsRef<Path>, next_seq: u64) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        // Restore the append-only invariant before opening for append: the file
        // must end at a record boundary, or the first write lands on the tail of
        // an unfinished one. See `truncate_incomplete_tail`.
        truncate_incomplete_tail(&path)?;
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { path, file, next_seq, poisoned: false })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Append one transition. Returns the fully-assigned record on success —
    /// the caller applies it to the in-memory stack only after this returns Ok.
    ///
    /// **All-or-nothing: a failed append consumes no sequence number and leaves no
    /// bytes behind.** Previously it could do both. `next_seq` advanced only after
    /// `write_all` *and* `sync_all` succeeded, while bytes could already have reached
    /// disk either way — so a failure left the file dirty and the number reusable, in
    /// two distinct ways (`docs/risks.md` **R14**):
    ///
    /// - a failed `sync_all` could leave a complete, checksum-valid line for that
    ///   `seq` on disk, and the next append would reuse the number — two readable
    ///   lines with the same `seq`, which makes ADR 0004's watermark filter undefined
    ///   and, under [ADR 0006], gives two Time Blocks the same identity;
    /// - a failed or partial `write_all` could leave an unterminated fragment.
    ///   `truncate_incomplete_tail` only runs at `open`, so nothing repaired it before
    ///   the next append concatenated onto it — reproducing the permanent data loss of
    ///   the torn-tail bug *within a single session* rather than across a restart.
    ///
    /// Both close the same way: record the length before writing, and on any failure
    /// restore it. The caller was never told the append succeeded, so removing it is
    /// exactly "not durably committed" — the same reasoning `truncate_incomplete_tail`
    /// documents, applied to the case where the tail is still newline-terminated and
    /// therefore invisible to it.
    pub fn append(&mut self, payload: TransitionPayload) -> std::io::Result<TransitionRecord> {
        if self.poisoned {
            return Err(std::io::Error::other(
                "log writer is poisoned: a previous append failed and could not be \
                 rolled back, so the file's length is unknown. Restart to recover — \
                 `open` repairs the tail.",
            ));
        }

        let record = TransitionRecord {
            seq: self.next_seq,
            timestamp: Utc::now(),
            payload,
        };
        let line = encode_line(&record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Captured before the write so a rollback restores the exact prior state.
        // Cutting to the last newline instead would be wrong here: a failed
        // `sync_all` leaves a line that *is* newline-terminated.
        let len_before = self.file.metadata()?.len();

        if let Err(e) = self
            .file
            .write_all(line.as_bytes())
            .and_then(|()| self.file.sync_all())
        {
            // Rollback failure is unrecoverable in-session: the length is now unknown,
            // and appending onto an unknown tail is precisely the bug above. Refuse
            // instead, and surface the original cause rather than the rollback's.
            if rollback_to(&mut self.file, len_before).is_err() {
                self.poisoned = true;
            }
            return Err(e);
        }

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

    /// The rollback mechanism behind `append`'s all-or-nothing guarantee, on the
    /// case `truncate_incomplete_tail` cannot handle: the bytes to remove form a
    /// **complete, newline-terminated line**, which is what a failed `sync_all`
    /// leaves behind. Cutting to the last newline would keep it; only the recorded
    /// pre-append length identifies it.
    #[test]
    fn rollback_removes_a_complete_line_that_truncating_to_a_newline_would_keep() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(sample_payload("committed")).unwrap();

        let len_before = std::fs::metadata(&path).unwrap().len();
        // A second complete record, exactly as a successful `write_all` leaves it.
        writer.append(sample_payload("never-acknowledged")).unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > len_before);

        // `truncate_incomplete_tail` is a no-op here — the file ends in a newline,
        // so it cannot see that the last line was never acknowledged.
        assert!(!truncate_incomplete_tail(&path).unwrap());

        let mut file = OpenOptions::new().read(true).write(true).open(&path).unwrap();
        rollback_to(&mut file, len_before).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 1);
        assert!(contents.contains("committed"));
        assert!(!contents.contains("never-acknowledged"));
        assert!(contents.ends_with('\n'), "must still end at a record boundary");
        let record = decode_line(contents.lines().next().unwrap()).unwrap();
        assert_eq!(record.seq, 0, "committed history is untouched");
    }

    /// A rollback that itself fails leaves the file at an unknown length. Appending
    /// onto an unknown tail is exactly the torn-tail data-loss bug, so the writer
    /// refuses instead. Restarting recovers, because `open` repairs the tail.
    #[test]
    fn a_poisoned_writer_refuses_to_append_rather_than_writing_onto_an_unknown_tail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(sample_payload("committed")).unwrap();
        let len_after_commit = std::fs::metadata(&path).unwrap().len();

        writer.poisoned = true;

        let err = writer.append(sample_payload("refused")).unwrap_err();
        assert!(err.to_string().contains("poisoned"));
        assert_eq!(
            writer.next_seq(),
            1,
            "a refused append must not consume a sequence number"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            len_after_commit,
            "a refused append must write nothing"
        );

        // A restart clears it: `open` re-establishes the record-boundary invariant.
        let reopened = LogWriter::open(&path, 1).unwrap();
        assert!(!reopened.poisoned);
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

    /// The dangerous regression. `truncate_incomplete_tail` runs on **every**
    /// `open`, so if it ever mistook a healthy log for a damaged one it would
    /// silently delete a committed record on an ordinary startup. Locks the fast
    /// path: a log ending at a record boundary is byte-for-byte untouched.
    #[test]
    fn opening_a_healthy_log_never_truncates_anything() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer.append(sample_payload("a")).unwrap();
            writer.append(sample_payload("b")).unwrap();
        }
        let before = std::fs::read(&path).unwrap();

        // Reopen several times, as a normal restart would.
        for _ in 0..3 {
            let _ = LogWriter::open(&path, 2).unwrap();
        }

        assert_eq!(before, std::fs::read(&path).unwrap(), "a healthy log must survive open untouched");
    }

    #[test]
    fn opening_a_missing_or_empty_log_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();

        let missing = dir.path().join("missing.jsonl");
        let mut writer = LogWriter::open(&missing, 0).unwrap();
        assert_eq!(writer.append(sample_payload("a")).unwrap().seq, 0);

        let empty = dir.path().join("empty.jsonl");
        std::fs::write(&empty, b"").unwrap();
        let mut writer = LogWriter::open(&empty, 0).unwrap();
        assert_eq!(writer.append(sample_payload("a")).unwrap().seq, 0);
    }

    /// A fragment with no record boundary anywhere before it — the whole file is
    /// one unfinished write. Truncating to zero is correct: nothing in it was
    /// ever acknowledged.
    #[test]
    fn a_log_containing_only_an_incomplete_write_truncates_to_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        std::fs::write(&path, b"{\"seq\":0,\"timestamp\":\"2026-01-01T00:00:00Z\",\"ty").unwrap();

        let mut writer = LogWriter::open(&path, 0).unwrap();
        writer.append(sample_payload("a")).unwrap();

        let file = File::open(&path).unwrap();
        let lines: Vec<String> = BufReader::new(file).lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines.len(), 1, "only the newly appended record remains");
        assert_eq!(decode_line(&lines[0]).unwrap().seq, 0);
    }

    /// Documents the one deliberately-accepted edge: a record whose bytes all
    /// landed except the trailing newline is discarded rather than repaired.
    /// It decodes, but it was never acknowledged — `append` returns only after
    /// `sync_all` — so the caller never applied it and the user never saw it
    /// commit. Discard is the safe direction; this test pins that choice so a
    /// future change to "repair it instead" is a deliberate decision.
    #[test]
    fn a_complete_record_missing_only_its_newline_is_discarded_not_repaired() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.jsonl");
        {
            let mut writer = LogWriter::open(&path, 0).unwrap();
            writer.append(sample_payload("committed")).unwrap();
        }
        // Append a fully-formed second record, then strip its trailing newline —
        // the byte whose absence marks the write as unfinished.
        {
            let mut w = LogWriter::open(&path, 1).unwrap();
            assert_eq!(w.append(sample_payload("unacknowledged")).unwrap().seq, 1);
        }
        let all = std::fs::read(&path).unwrap();
        std::fs::write(&path, &all[..all.len() - 1]).unwrap();

        let _ = LogWriter::open(&path, 2).unwrap();

        let file = File::open(&path).unwrap();
        let lines: Vec<String> = BufReader::new(file).lines().map(|l| l.unwrap()).collect();
        assert_eq!(lines.len(), 1, "the unterminated record is dropped");
        assert!(lines[0].contains("committed"));
        assert!(!lines[0].contains("unacknowledged"));
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
