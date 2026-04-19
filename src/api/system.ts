import { tauriInvoke } from "@api/tauri";
import { isUploadSupported, pickFileFromBrowser, uploadFile } from "@api/upload";

export interface CpuInfo {
  name: string;
  count: number;
  usage: number;
}

export interface MemoryInfo {
  total: number;
  used: number;
  available: number;
  usage: number;
}

export interface SwapInfo {
  total: number;
  used: number;
  usage: number;
}

export interface DiskDetail {
  name: string;
  mount_point: string;
  file_system: string;
  total: number;
  used: number;
  available: number;
  usage: number;
  is_removable: boolean;
}

export interface DiskInfo {
  total: number;
  used: number;
  available: number;
  usage: number;
  disks?: DiskDetail[];
  path?: string;
}

export interface NetworkInterface {
  name: string;
  received: number;
  transmitted: number;
}

export interface NetworkInfo {
  total_received: number;
  total_transmitted: number;
  interfaces: NetworkInterface[];
}

export interface SystemInfo {
  os: string;
  arch: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  host_name: string;
  cpu: CpuInfo;
  memory: MemoryInfo;
  swap: SwapInfo;
  disk: DiskInfo;
  network: NetworkInfo;
  uptime: number;
  process_count: number;
}

export interface ServerResourceUsage {
  server_id: string;
  server_name: string;
  status: string;
  pid: number | null;
  cpu: CpuInfo;
  memory: MemoryInfo;
  disk: DiskInfo;
}

export const systemApi = {
  async pickAndUploadBrowserFile(accept?: string): Promise<string | null> {
    if (!isUploadSupported()) {
      throw new Error("仅在Docker/浏览器环境中支持该方法");
    }

    const input = document.createElement("input");
    input.type = "file";
    if (accept) {
      input.accept = accept;
    }

    const selectedFile = await new Promise<File | null>((resolve) => {
      input.addEventListener(
        "change",
        () => {
          resolve(input.files?.[0] ?? null);
        },
        { once: true },
      );
      input.click();
    });

    if (!selectedFile) {
      return null;
    }

    const uploaded = await uploadFile(selectedFile);
    return uploaded.saved_path;
  },

  async getSystemInfo(): Promise<SystemInfo> {
    return tauriInvoke("get_system_info");
  },

  async getServerResourceUsage(serverId: string): Promise<ServerResourceUsage> {
    return tauriInvoke("get_server_resource_usage", { serverId });
  },

  async pickJarFile(): Promise<string | null> {
    if (isUploadSupported()) {
      return this.pickAndUploadBrowserFile(".jar");
    }
    return tauriInvoke("pick_jar_file");
  },

  async pickArchiveFile(): Promise<string | null> {
    if (isUploadSupported()) {
      return this.pickAndUploadBrowserFile(".zip,.tar,.tar.gz,.tgz,.jar");
    }
    return tauriInvoke("pick_archive_file");
  },

  async pickStartupFile(mode: "jar" | "bat" | "sh"): Promise<string | null> {
    if (isUploadSupported()) {
      const acceptMap: Record<string, string> = {
        jar: ".jar",
        bat: ".bat",
        sh: ".sh",
      };
      const file = await pickFileFromBrowser({ accept: acceptMap[mode] || ".jar" });
      if (file && file instanceof File) {
        const result = await uploadFile(file);
        return result.saved_path;
      }
      return null;
    }
    return tauriInvoke("pick_startup_file", { mode });
  },

  async pickServerExecutable(): Promise<{ path: string; mode: "jar" | "bat" | "sh" } | null> {
    if (isUploadSupported()) {
      const file = await pickFileFromBrowser({ accept: ".jar,.bat,.sh" });
      if (file && file instanceof File) {
        const result = await uploadFile(file);
        const ext = file.name.split(".").pop()?.toLowerCase() || "jar";
        const mode = ext === "bat" ? "bat" : ext === "sh" ? "sh" : "jar";
        return { path: result.saved_path, mode };
      }
      return null;
    }
    const result = await tauriInvoke<[string, string] | null>("pick_server_executable");
    if (result) {
      return { path: result[0], mode: result[1] as "jar" | "bat" | "sh" };
    }
    return null;
  },

  async pickJavaFile(): Promise<string | null> {
    if (isUploadSupported()) {
      const file = await pickFileFromBrowser({ accept: ".exe" });
      if (file && file instanceof File) {
        const result = await uploadFile(file);
        return result.saved_path;
      }
      return null;
    }
    return tauriInvoke("pick_java_file");
  },

  async pickSaveFile(): Promise<string | null> {
    if (isUploadSupported()) {
      throw new Error("Docker环境不支持原生文件选择器，请使用文件上传功能");
    }
    return tauriInvoke("pick_save_file");
  },

  async pickFolder(): Promise<string | null> {
    if (isUploadSupported()) {
      throw new Error("Docker环境不支持原生文件选择器，请使用文件上传功能");
    }
    return tauriInvoke("pick_folder");
  },

  async pickImageFile(): Promise<string | null> {
    if (isUploadSupported()) {
      const file = await pickFileFromBrowser({ accept: ".png,.jpg,.jpeg,.webp,.gif,.bmp" });
      if (file && file instanceof File) {
        const result = await uploadFile(file);
        return result.saved_path;
      }
      return null;
    }
    return tauriInvoke("pick_image_file");
  },

  async openFile(path: string): Promise<void> {
    return tauriInvoke("open_file", { path });
  },

  async openFolder(path: string): Promise<void> {
    return tauriInvoke("open_folder", { path });
  },

  async getDefaultRunPath(): Promise<string> {
    return tauriInvoke("get_default_run_path");
  },

  async getSafeModeStatus(): Promise<boolean> {
    return tauriInvoke("get_safe_mode_status");
  },
};
