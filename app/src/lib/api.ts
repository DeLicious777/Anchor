import { invoke } from "@tauri-apps/api/core";
import type { StackView } from "./types";

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
