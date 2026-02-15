import { getVersion } from "@tauri-apps/api/app";

/**
 * 应用版本号管理
 *
 * 版本号从 Tauri 后端读取，后端版本号来自：
 * - src-tauri/Cargo.toml - version
 * - src-tauri/tauri.conf.json - version
 *
 * 修改版本时只需要更新这两个文件，前端会自动同步
 */

let cachedVersion: string | null = null;

/**
 * 获取应用版本号（从 Tauri 后端读取）
 */
export async function getAppVersion(): Promise<string> {
  if (cachedVersion) {
    return cachedVersion;
  }

  try {
    cachedVersion = await getVersion();
    return cachedVersion;
  } catch (error) {
    console.error("获取版本号失败:", error);
    return "0.0.0";
  }
}

/**
 * 同步获取版本号（用于模板中）
 * 注意：首次调用时可能返回 '加载中...'，需要配合 onMounted 使用
 */
export function getAppVersionSync(): string {
  return cachedVersion || "加载中...";
}

export const BUILD_YEAR = "2026";
