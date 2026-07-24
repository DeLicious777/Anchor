//! Line framing for the transition log, per ADR 0004: `<json>\t<crc32-hex>\n`.
//!
//! The checksum is computed over the JSON substring's raw bytes and lives OUTSIDE
//! the JSON object — never as a field inside it. That's the whole point of this
//! module: putting the checksum inside the object it protects is a self-reference
//! bug (you'd need to serialize twice, or hand-splice a field into someone else's
//! serialization). Framing it outside means encode/decode never touch the JSON
//! structure to compute or verify the checksum — just raw bytes and a tab split.

use crate::model::TransitionRecord;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FramingError {
    #[error("line has no tab separator between JSON and checksum")]
    NoTab,
    #[error("checksum mismatch: expected {expected:08x}, found {found:08x}")]
    ChecksumMismatch { expected: u32, found: u32 },
    #[error("checksum field is not valid hex: {0}")]
    InvalidChecksumHex(String),
    #[error("JSON payload is invalid: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

/// Encode a record into a full log line, including the trailing newline.
pub fn encode_line(record: &TransitionRecord) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(record)?;
    let checksum = crc32fast::hash(json.as_bytes());
    Ok(format!("{json}\t{checksum:08x}\n"))
}

/// Decode a single line (WITHOUT its trailing newline — strip that before calling,
/// e.g. via `BufRead::lines()` which already does this). Verifies the checksum
/// over the JSON substring's exact bytes before attempting to parse it, so a
/// torn/corrupt line is never even handed to serde.
pub fn decode_line(line: &str) -> Result<TransitionRecord, FramingError> {
    let (json_part, checksum_part) = line.rsplit_once('\t').ok_or(FramingError::NoTab)?;

    let expected = u32::from_str_radix(checksum_part, 16)
        .map_err(|_| FramingError::InvalidChecksumHex(checksum_part.to_string()))?;
    let found = crc32fast::hash(json_part.as_bytes());
    if found != expected {
        return Err(FramingError::ChecksumMismatch { expected, found });
    }

    let record = serde_json::from_str(json_part)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TransitionPayload;

    fn sample_record(seq: u64) -> TransitionRecord {
        TransitionRecord {
            seq,
            timestamp: chrono::Utc::now(),
            payload: TransitionPayload::Start {
                name: "Client work".to_string(),
                project: Some("Acme".to_string()),
                client: None,
            },
        }
    }

    #[test]
    fn round_trip() {
        let record = sample_record(1);
        let line = encode_line(&record).unwrap();
        assert!(line.ends_with('\n'));
        let decoded = decode_line(line.trim_end_matches('\n')).unwrap();
        assert_eq!(decoded.seq, record.seq);
    }

    #[test]
    fn detects_single_byte_corruption() {
        let record = sample_record(2);
        let line = encode_line(&record).unwrap();
        let trimmed = line.trim_end_matches('\n');
        // Flip one character inside the JSON portion, leave the checksum as-is.
        let (json_part, checksum_part) = trimmed.rsplit_once('\t').unwrap();
        let mut corrupted_json: Vec<char> = json_part.chars().collect();
        let mid = corrupted_json.len() / 2;
        corrupted_json[mid] = if corrupted_json[mid] == 'a' { 'b' } else { 'a' };
        let corrupted_line: String = corrupted_json.into_iter().collect::<String>() + "\t" + checksum_part;

        let result = decode_line(&corrupted_line);
        assert!(matches!(result, Err(FramingError::ChecksumMismatch { .. })));
    }

    #[test]
    fn missing_tab_is_no_tab_error() {
        let result = decode_line("not a valid line at all");
        assert!(matches!(result, Err(FramingError::NoTab)));
    }

    #[test]
    fn checksum_never_appears_inside_json_object() {
        // Guards the actual design invariant: the checksum field must not be
        // parseable as part of the JSON object itself.
        let record = sample_record(3);
        let line = encode_line(&record).unwrap();
        let (json_part, _) = line.trim_end_matches('\n').rsplit_once('\t').unwrap();
        let parsed: serde_json::Value = serde_json::from_str(json_part).unwrap();
        assert!(parsed.get("checksum").is_none());
    }
}
