import { systemApi } from "@api/system";
import { i18n } from "@language";

/**
 * 应用版本号管理
 *
 * 版本号从后端命令读取。
 * 正式版默认展示底层 semver；nightly 构建可由后端注入自定义显示版本。
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
    cachedVersion = await systemApi.getAppVersion();
    return cachedVersion;
  } catch (error) {
    console.error(i18n.t("about.update_check_failed"), error);
    return "0.0.0";
  }
}

/**
 * 同步获取版本号（用于模板中）
 * 注意：首次调用时可能返回加载中状态，需要配合 onMounted 使用
 */
export function getAppVersionSync(): string {
  return cachedVersion || i18n.t("common.loading");
}

export const BUILD_YEAR = "2026";
