import { tauriInvoke } from "./tauri";

export interface JavaInfo {
  path: string;
  version: string;
  vendor: string;
  is_64bit: boolean;
  major_version: number;
}

export const javaApi = {
  async detect(): Promise<JavaInfo[]> {
    return tauriInvoke("detect_java");
  },

  async validate(path: string): Promise<JavaInfo> {
    return tauriInvoke("validate_java_path", { path });
  },

  async installJava(url: string, versionName: string): Promise<string> {
    return tauriInvoke("install_java", { url, versionName });
  },

  async cancelInstall(): Promise<void> {
    return tauriInvoke("cancel_java_install");
  },
};

