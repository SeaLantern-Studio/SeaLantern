import { tauriInvoke } from "@api/tauri";

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export async function getLogs(limit?: number): Promise<LogEntry[]> {
  return tauriInvoke("get_logs", { limit });
}

export async function clearLogs(): Promise<void> {
  return tauriInvoke("clear_logs");
}

export async function exportAppLogs(savePath: string): Promise<void> {
  return tauriInvoke("export_app_logs", { savePath });
}

export async function checkDeveloperMode(): Promise<boolean> {
  return tauriInvoke("check_developer_mode");
}
