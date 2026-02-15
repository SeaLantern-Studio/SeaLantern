import { invoke } from "@tauri-apps/api/core";

export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return await invoke<T>(command, args);
}
