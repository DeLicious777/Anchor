/** How this block's end time was established. `null` while still active. */
export type EndDetermination = "user-determined" | "system-inferred";

/**
 * How the block entered the system, and whether it has since been adjusted.
 * Origin survives adjustment deliberately — a manually entered block nudged by
 * a second must stay distinguishable from a live capture that needed fixing.
 */
export type CaptureOrigin =
  | "live-capture"
  | "live-capture-adjusted"
  | "manual-entry"
  | "manual-entry-adjusted";

/**
 * What ultimately happened to interrupted work. **Do not read this directly** —
 * `null` means *never interrupted* OR *interrupted and unresolved*, and only
 * the live stack tells them apart. Use `ClosedBlock.derived_interruption_status`,
 * which the backend computes as the single canonical answer (ADR 0005).
 */
export type InterruptionOutcome = "resumed" | "skipped";

export type DerivedInterruptionStatus = "never-interrupted" | "pending" | "resumed" | "skipped";

/**
 * Replaced the single `completion_reason` field (ADR 0005), which conflated
 * three independent questions: how the end was established, how the block
 * entered the system, and what became of interrupted work.
 */
export interface TimeBlock {
  id: string;
  name: string;
  project: string | null;
  client: string | null;
  start: string;
  end: string | null;
  end_determination: EndDetermination | null;
  capture_origin: CaptureOrigin;
  interruption_outcome: InterruptionOutcome | null;
}

/** A closed block as the backend serves it: the block plus its canonical projection. */
export interface ClosedBlock extends TimeBlock {
  derived_interruption_status: DerivedInterruptionStatus;
}

export interface StackFrame {
  paused_time_block_id: string;
  name: string;
  project: string | null;
  client: string | null;
}

export interface StackView {
  active: TimeBlock | null;
  stack: StackFrame[];
  closed: ClosedBlock[];
}

export interface TaskTemplate {
  id: string;
  name: string;
  project: string | null;
  client: string | null;
}

export interface ExportSettings {
  rounding_enabled: boolean;
  rounding_interval_minutes: number;
}

export interface HotkeyBindings {
  switch: string;
  interrupt: string;
  return_previous: string;
  return_original: string;
  complete: string;
}
