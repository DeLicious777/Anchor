export type CompletionReason = "explicit" | "auto-completed-on-skip" | "recovered-gap";

export interface TimeBlock {
  id: string;
  name: string;
  project: string | null;
  client: string | null;
  start: string;
  end: string | null;
  completion_reason: CompletionReason | null;
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
  closed: TimeBlock[];
}
