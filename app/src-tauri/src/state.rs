//! Shared app state: the in-memory stack and the log writer, behind one Mutex so
//! they can never drift apart (a mutation to one always happens alongside the
//! other, under the same lock).

use crate::log::reader::replay;
use crate::log::writer::LogWriter;
use crate::stack::InterruptionStack;
use std::path::Path;
use std::sync::Mutex;

pub struct AppState {
    pub inner: Mutex<Inner>,
}

pub struct Inner {
    pub stack: InterruptionStack,
    pub writer: LogWriter,
}

impl AppState {
    /// Replays the log at `path` to reconstruct state, then opens the writer at
    /// the resulting `next_seq`. Returns whether a torn tail was found and
    /// discarded, so the caller can surface it (e.g. log a warning) — this
    /// slice doesn't implement UI for it, but it must not be silently dropped.
    pub fn init(path: impl AsRef<Path>) -> Result<(Self, bool), Box<dyn std::error::Error>> {
        let result = replay(&path, None, None)?;
        let writer = LogWriter::open(&path, result.next_seq)?;
        let state = AppState {
            inner: Mutex::new(Inner { stack: result.stack, writer }),
        };
        Ok((state, result.torn_line_discarded))
    }
}
