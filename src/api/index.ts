/**
 * API 层统一导出
 * 所有 API 模块统一从此处导入
 */

export { tauriInvoke, tauriInvokeAll, createCachedInvoke } from "./tauri";
export type { InvokeOptions } from "./tauri";

export { serverApi } from "./server";
export type { ServerStatusInfo } from "./server";

export { javaApi } from "./java";
export type { JavaInfo } from "./java";

export { configApi } from "./config";
export type { ConfigEntry, ServerProperties } from "./config";

export { playerApi } from "./player";
export type { PlayerEntry, BanEntry, OpEntry } from "./player";

export { settingsApi, checkAcrylicSupport, applyAcrylic, getSystemFonts } from "./settings";
export type { AppSettings } from "./settings";

export { systemApi } from "./system";
export type {
  CpuInfo,
  MemoryInfo,
  SwapInfo,
  DiskDetail,
  DiskInfo,
  NetworkInterface,
  NetworkInfo,
  SystemInfo,
} from "./system";

export * from "./update";
export * from "./plugin";
export * from "./remoteLocales";
