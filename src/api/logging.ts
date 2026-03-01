import { invoke } from "@tauri-apps/api/core";

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export async function getLogs(limit?: number): Promise<LogEntry[]> {
  return invoke("get_logs", { limit });
}

export async function clearLogs(): Promise<void> {
  return invoke("clear_logs");
}

export async function checkDeveloperMode(): Promise<boolean> {
  return invoke("check_developer_mode");
}
