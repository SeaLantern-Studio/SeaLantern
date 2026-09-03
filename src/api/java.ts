import { tauriInvoke } from "@api/tauri";
import { invoke } from "@api/invoke";

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
  /**
   * 检测系统中的 Java 环境
   */
  async detect(): Promise<JavaInfo[]> {
    // 后端返回 JavaDetectionReport，含 installations 和 errors
    const report = await invoke<{ installations: JavaInfo[]; errors: unknown[] }>("java_detect");
    return report.installations;
  },

  /**
   * 验证指定路径的 Java 是否可用
   */
  async validate(path: string): Promise<JavaInfo> {
    return invoke<JavaInfo>("java_validate", { path });
  },

  /**
   * 安装 Java
   */
  async installJava(url: string, versionName: string): Promise<string> {
    return tauriInvoke("install_java", { url, versionName });
  },

  /**
   * 取消 Java 安装
   */
  async cancelInstall(): Promise<void> {
    return tauriInvoke("cancel_java_install");
  },
};
