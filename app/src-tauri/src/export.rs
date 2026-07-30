//! Pure, Tauri-free computation for turning stored Time Blocks into
//! billing-usable XLSX/JSON output (see `docs/product/features/export.md`).
//! Deliberately takes only shared references to the source data — nothing here
//! can mutate the durable timeline, by construction, not by convention.

use crate::model::TimeBlock;
use chrono::{DateTime, Utc};
use rust_xlsxwriter::Workbook;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportRow {
    pub name: String,
    pub project: Option<String>,
    pub client: Option<String>,
    pub duration_seconds: i64,
}

/// Time Blocks whose `start` falls in `[range_start, range_end)` — inclusion is
/// always by start time, never end time (see Technical Constraints). If
/// `active` is Some and its start is in range, a cloned synthetic block with
/// `end = Some(now)` stands in for its elapsed-so-far duration; the real
/// active entry (borrowed, not owned) is never touched.
pub fn blocks_in_range(
    closed: &[TimeBlock],
    active: Option<&TimeBlock>,
    range_start: DateTime<Utc>,
    range_end: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Vec<TimeBlock> {
    let in_range = |b: &TimeBlock| b.start >= range_start && b.start < range_end;

    let mut result: Vec<TimeBlock> = closed.iter().filter(|b| in_range(b)).cloned().collect();

    if let Some(active) = active {
        if in_range(active) {
            let mut synthetic = active.clone();
            synthetic.end = Some(now);
            result.push(synthetic);
        }
    }

    result
}

/// Groups by (name, project, client), summing durations. First-seen order.
pub fn group(blocks: &[TimeBlock]) -> Vec<ExportRow> {
    let mut rows: Vec<ExportRow> = Vec::new();

    for block in blocks {
        let Some(duration) = block.duration() else {
            continue;
        };
        let seconds = duration.num_seconds();

        match rows
            .iter_mut()
            .find(|r| r.name == block.name && r.project == block.project && r.client == block.client)
        {
            Some(row) => row.duration_seconds += seconds,
            None => rows.push(ExportRow {
                name: block.name.clone(),
                project: block.project.clone(),
                client: block.client.clone(),
                duration_seconds: seconds,
            }),
        }
    }

    rows
}

/// Ceiling rounding to the next multiple of `interval_minutes`. A zero duration
/// stays zero — there's nothing to round up from "no work done".
pub fn round_up_seconds(total_seconds: i64, interval_minutes: u32) -> i64 {
    if total_seconds == 0 {
        return 0;
    }
    let interval_seconds = i64::from(interval_minutes) * 60;
    let intervals = (total_seconds + interval_seconds - 1) / interval_seconds;
    intervals * interval_seconds
}

/// Always grouped; each row's total is rounded if `rounding_interval` is Some.
pub fn xlsx_rows(blocks: &[TimeBlock], rounding_interval: Option<u32>) -> Vec<ExportRow> {
    let mut rows = group(blocks);
    if let Some(interval) = rounding_interval {
        for row in &mut rows {
            row.duration_seconds = round_up_seconds(row.duration_seconds, interval);
        }
    }
    rows
}

/// JSON's shape depends on whether rounding is enabled for this export — see
/// `docs/product/features/export.md`'s "JSON rounding-on vs. rounding-off
/// shape". Raw when rounding is off (nothing to sum yet); grouped+rounded
/// (identical to `xlsx_rows`) when it's on, so the two formats numerically
/// agree by construction.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum JsonExport {
    Raw(Vec<TimeBlock>),
    Grouped(Vec<ExportRow>),
}

pub fn json_export(blocks: &[TimeBlock], rounding_interval: Option<u32>) -> JsonExport {
    match rounding_interval {
        None => JsonExport::Raw(blocks.to_vec()),
        Some(interval) => JsonExport::Grouped(xlsx_rows(blocks, Some(interval))),
    }
}

/// Writes a single flat worksheet: Name / Project / Client / Duration (minutes,
/// decimal — precise, never rounded for display beyond what `rows` already
/// carries).
pub fn write_xlsx(rows: &[ExportRow], path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.write_string(0, 0, "Name").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 1, "Project").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 2, "Client").map_err(|e| e.to_string())?;
    worksheet.write_string(0, 3, "Duration (minutes)").map_err(|e| e.to_string())?;

    for (i, row) in rows.iter().enumerate() {
        let excel_row = (i + 1) as u32;
        worksheet.write_string(excel_row, 0, &row.name).map_err(|e| e.to_string())?;
        worksheet
            .write_string(excel_row, 1, row.project.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
        worksheet
            .write_string(excel_row, 2, row.client.as_deref().unwrap_or(""))
            .map_err(|e| e.to_string())?;
        worksheet
            .write_number(excel_row, 3, row.duration_seconds as f64 / 60.0)
            .map_err(|e| e.to_string())?;
    }

    workbook.save(path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EndDetermination;
    use std::sync::LazyLock;

    static BASE: LazyLock<DateTime<Utc>> = LazyLock::new(Utc::now);

    fn t(offset_secs: i64) -> DateTime<Utc> {
        *BASE + chrono::Duration::seconds(offset_secs)
    }

    fn closed_block(name: &str, project: Option<&str>, start_offset: i64, duration_secs: i64) -> TimeBlock {
        let mut b = TimeBlock::new(name.into(), project.map(String::from), None, t(start_offset));
        b.end = Some(t(start_offset + duration_secs));
        b.end_determination = Some(EndDetermination::UserDetermined);
        b
    }

    #[test]
    fn group_combines_same_task_separated_by_an_interrupting_task() {
        let blocks = vec![
            closed_block("A", Some("Acme"), 0, 300),
            closed_block("B", None, 300, 60), // the interrupting task in between
            closed_block("A", Some("Acme"), 360, 300),
        ];

        let rows = group(&blocks);
        assert_eq!(rows.len(), 2, "must combine into one row for A and one for B, not three rows");
        let a = rows.iter().find(|r| r.name == "A").unwrap();
        assert_eq!(a.duration_seconds, 600);
    }

    #[test]
    fn rounding_ceils_1_minute_to_15_and_16_minutes_to_30_at_15_minute_interval() {
        assert_eq!(round_up_seconds(60, 15), 15 * 60);
        assert_eq!(round_up_seconds(16 * 60, 15), 30 * 60);
    }

    #[test]
    fn rounding_disabled_preserves_exact_unrounded_sum() {
        let blocks = vec![closed_block("A", None, 0, 90)];
        let rows = xlsx_rows(&blocks, None);
        assert_eq!(rows[0].duration_seconds, 90);
    }

    #[test]
    fn blocks_in_range_filters_by_start_time_not_end_time() {
        // Starts just before the range boundary, ends well after it — must be
        // excluded, because inclusion is decided by start, never end.
        let blocks = vec![closed_block("A", None, -10, 1000)];
        let range_start = t(0);
        let range_end = t(3600);
        let result = blocks_in_range(&blocks, None, range_start, range_end, t(5000));
        assert!(result.is_empty(), "a block starting before the range must be excluded even if it overlaps the range");

        let blocks = vec![closed_block("B", None, 100, 50)];
        let result = blocks_in_range(&blocks, None, range_start, range_end, t(5000));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn active_entry_in_range_is_included_with_elapsed_so_far_duration_and_is_not_mutated() {
        let active = TimeBlock::new("C".into(), None, None, t(0));
        let now = t(120);
        let result = blocks_in_range(&[], Some(&active), t(-10), t(3600), now);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].duration(), Some(chrono::Duration::seconds(120)));
        assert!(active.end.is_none(), "the real active entry must never be mutated by computing an export");
    }

    #[test]
    fn json_raw_mode_is_one_entry_per_time_block_with_exact_duration() {
        let blocks = vec![closed_block("A", None, 0, 90), closed_block("A", None, 200, 30)];
        match json_export(&blocks, None) {
            JsonExport::Raw(entries) => {
                assert_eq!(entries.len(), 2, "raw mode must not group same-name entries");
                assert_eq!(entries[0].duration(), Some(chrono::Duration::seconds(90)));
            }
            JsonExport::Grouped(_) => panic!("rounding disabled must produce Raw, not Grouped"),
        }
    }

    #[test]
    fn json_grouped_mode_numerically_matches_xlsx_rows_for_same_input() {
        let blocks = vec![
            closed_block("A", Some("Acme"), 0, 300),
            closed_block("A", Some("Acme"), 360, 300),
        ];
        let xlsx = xlsx_rows(&blocks, Some(15));
        match json_export(&blocks, Some(15)) {
            JsonExport::Grouped(rows) => assert_eq!(rows, xlsx, "JSON and XLSX must numerically agree when rounding is enabled"),
            JsonExport::Raw(_) => panic!("rounding enabled must produce Grouped, not Raw"),
        }
    }

    #[test]
    fn write_xlsx_produces_a_non_empty_file_starting_with_the_zip_magic_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.xlsx");
        let rows = vec![ExportRow { name: "A".into(), project: None, client: None, duration_seconds: 900 }];

        write_xlsx(&rows, &path).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK", "an .xlsx file is a zip archive and must start with the zip magic bytes");
    }
}
