import { platform } from "@tauri-apps/plugin-os";

let cachedPlatform: string | null = null;

/**
 * 获取当前操作系统平台
 * @returns 'macos' | 'windows' | 'linux' | 'ios' | 'android'
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
