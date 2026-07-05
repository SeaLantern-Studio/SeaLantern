import { isBrowserEnv, tauriInvoke } from "@api/tauri";
import { i18n } from "@language";

const JAVA_INSTALL_DESKTOP_ONLY_ERROR_KEY = "settings.java_download_desktop_only";

function assertJavaInstallSupported(): void {
  if (isBrowserEnv()) {
    throw new Error(i18n.t(JAVA_INSTALL_DESKTOP_ONLY_ERROR_KEY));
  }
}

/**
 * Java 环境信息
 */
export interface JavaInfo {
  path: string;
  version: string;
  vendor: string;
  is_64bit: boolean;
  major_version: number;
}

/**
 * Java 环境管理 API
 */
export const javaApi = {
  canInstallLocally(): boolean {
    return !isBrowserEnv();
  },

  /**
   * 检测系统中的 Java 环境
   */
  async detect(): Promise<JavaInfo[]> {
    return tauriInvoke("detect_java");
  },

  /**
   * 验证指定路径的 Java 是否可用
   */
  async validate(path: string): Promise<JavaInfo> {
    return tauriInvoke("validate_java_path", { path });
  },

  /**
   * 安装 Java
   */
  async installJava(url: string, versionName: string): Promise<string> {
    assertJavaInstallSupported();
    return tauriInvoke("install_java", { url, versionName });
  },

  /**
   * 取消 Java 安装
   */
  async cancelInstall(): Promise<void> {
    return tauriInvoke("cancel_java_install");
  },
};
