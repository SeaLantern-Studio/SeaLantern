/**
 * 平台检测工具函数
 */

import { platform } from "@tauri-apps/plugin-os";

let cachedPlatform: string | null = null;

/**
 * 获取当前平台
 * @returns 平台名称: "macos" | "windows" | "linux" | "ios" | "android"
 */
export async function getPlatform(): Promise<string> {
  if (cachedPlatform === null) {
    cachedPlatform = await platform();
  }
  return cachedPlatform;
}

/**
 * 检查是否为 macOS
 */
export async function isMacOS(): Promise<boolean> {
  const p = await getPlatform();
  return p === "macos";
}

/**
 * 检查是否为 Windows
 */
export async function isWindows(): Promise<boolean> {
  const p = await getPlatform();
  return p === "windows";
}

/**
 * 检查是否为 Linux
 */
export async function isLinux(): Promise<boolean> {
  const p = await getPlatform();
  return p === "linux";
}
