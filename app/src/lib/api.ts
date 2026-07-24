import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { StackView, TaskTemplate } from "./types";

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
