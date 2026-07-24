import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { StackView } from "./types";

const STATE_CHANGED_EVENT = "state-changed";

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
