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

export async function checkDeveloperMode(): Promise<boolean> {
  return tauriInvoke("check_developer_mode");
}

/**
 * 将日志文本上传到 mclo.gs 并返回可分享的链接。
 * 自动适配原生（invoke）与 Docker（HTTP API）两种运行模式。
 */
export async function shareLogs(content: string): Promise<string> {
  return tauriInvoke<string>("share_logs", { content });
}
