import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import type { ExportSettings, HotkeyBindings, StackView, TaskTemplate } from "./types";

const STATE_CHANGED_EVENT = "state-changed";
const TEMPLATES_CHANGED_EVENT = "templates-changed";

/**
 * Every window (dashboard, mini widget) calls this on mount instead of polling
 * `getState()` repeatedly. The backend emits `state-changed` after every
 * successful mutation (see `commands::emit_state_changed`), so this is what
 * makes both windows agree within milliseconds, by construction.
 */
export function onStateChanged(callback: (view: StackView) => void): Promise<UnlistenFn> {
  return listen<StackView>(STATE_CHANGED_EVENT, (event) => callback(event.payload));
}

export function switchTask(name: string, project: string | null, client: string | null): Promise<StackView> {
  return invoke("switch", { name, project, client });
}

export function interruptTask(name: string, project: string | null, client: string | null): Promise<StackView> {
  return invoke("interrupt", { name, project, client });
}

export function returnPrevious(): Promise<StackView> {
  return invoke("return_previous");
}

export function returnOriginal(): Promise<StackView> {
  return invoke("return_original");
}

export function completeTask(): Promise<StackView> {
  return invoke("complete");
}

/**
 * Renames the currently active task in place — no new Time Block, no stack
 * effect, start time untouched. Used both to give an auto-named ("Anchor N")
 * task a real name and to retarget it to an existing template/past task
 * while it's still running.
 */
export function renameActive(name: string, project: string | null, client: string | null): Promise<StackView> {
  return invoke("rename_active", { name, project, client });
}

export function getState(): Promise<StackView> {
  return invoke("get_state");
}

/**
 * Templates are a separate slice from the interruption stack — their own
 * event, so listeners never have to guess which part of the app changed.
 */
export function onTemplatesChanged(callback: (templates: TaskTemplate[]) => void): Promise<UnlistenFn> {
  return listen<TaskTemplate[]>(TEMPLATES_CHANGED_EVENT, (event) => callback(event.payload));
}

export function createTemplate(name: string, project: string | null, client: string | null): Promise<TaskTemplate> {
  return invoke("create_template", { name, project, client });
}

export function updateTemplate(
  id: string,
  name: string,
  project: string | null,
  client: string | null,
): Promise<TaskTemplate> {
  return invoke("update_template", { id, name, project, client });
}

export function deleteTemplate(id: string): Promise<void> {
  return invoke("delete_template", { id });
}

export function listTemplates(): Promise<TaskTemplate[]> {
  return invoke("list_templates");
}

export function getExportSettings(): Promise<ExportSettings> {
  return invoke("get_export_settings");
}

export function updateExportSettings(roundingEnabled: boolean, roundingIntervalMinutes: number): Promise<ExportSettings> {
  return invoke("update_export_settings", { roundingEnabled, roundingIntervalMinutes });
}

export function exportXlsx(
  path: string,
  rangeStart: string,
  rangeEnd: string,
  roundingEnabled: boolean,
  roundingIntervalMinutes: number,
): Promise<void> {
  return invoke("export_xlsx", { path, rangeStart, rangeEnd, roundingEnabled, roundingIntervalMinutes });
}

export function exportJson(
  path: string,
  rangeStart: string,
  rangeEnd: string,
  roundingEnabled: boolean,
  roundingIntervalMinutes: number,
): Promise<void> {
  return invoke("export_json", { path, rangeStart, rangeEnd, roundingEnabled, roundingIntervalMinutes });
}

export function getHotkeyBindings(): Promise<HotkeyBindings> {
  return invoke("get_hotkey_bindings");
}

/**
 * All five bindings are sent together and applied atomically on the backend
 * (`hotkeys::apply_remap`) — either every accelerator registers and persists,
 * or none of them do and the previous bindings stay live. A rejected remap
 * throws, with a message identifying which action/accelerator failed.
 */
export function updateHotkeyBindings(bindings: HotkeyBindings): Promise<HotkeyBindings> {
  return invoke("update_hotkey_bindings", {
    switch: bindings.switch,
    interrupt: bindings.interrupt,
    returnPrevious: bindings.return_previous,
    returnOriginal: bindings.return_original,
    complete: bindings.complete,
  });
}

/**
 * Native "Save As" dialog (tauri-plugin-dialog) — the user picks the
 * destination, we never invent our own file-picker UI. Returns null if the
 * user cancelled.
 */
export function chooseSaveLocation(suggestedName: string, extension: "xlsx" | "json"): Promise<string | null> {
  return save({
    defaultPath: suggestedName,
    filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
  });
}
