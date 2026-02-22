/**
 * API 层统一导出
 * 所有 API 模块统一从此处导入
 */

export { tauriInvoke, tauriInvokeAll, createCachedInvoke } from "@api/tauri";
export type { InvokeOptions } from "@api/tauri";

export { serverApi } from "@api/server";
export type { ServerStatusInfo } from "@api/server";

export { javaApi } from "@api/java";
export type { JavaInfo } from "@api/java";

export { configApi } from "@api/config";
export type { ConfigEntry, ServerProperties } from "@api/config";

export { playerApi } from "@api/player";
export type { PlayerEntry, BanEntry, OpEntry } from "@api/player";

export { settingsApi, checkAcrylicSupport, applyAcrylic, getSystemFonts } from "@api/settings";
export type { AppSettings } from "@api/settings";

export { systemApi } from "@api/system";
export type {
  CpuInfo,
  MemoryInfo,
  SwapInfo,
  DiskDetail,
  DiskInfo,
  NetworkInterface,
  NetworkInfo,
  SystemInfo,
} from "@api/system";

export * from "@api/update";
export * from "@api/plugin";
export * from "@api/remoteLocales";
