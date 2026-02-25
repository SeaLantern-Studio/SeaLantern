import { tauriInvoke } from "@api/tauri";

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
  disks: DiskDetail[];
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

export const systemApi = {
  async getSystemInfo(): Promise<SystemInfo> {
    return tauriInvoke("get_system_info");
  },

  async pickJarFile(): Promise<string | null> {
    return tauriInvoke("pick_jar_file");
  },

  async pickArchiveFile(): Promise<string | null> {
    return tauriInvoke("pick_archive_file");
  },

  async pickStartupFile(mode: "jar" | "bat" | "sh"): Promise<string | null> {
    return tauriInvoke("pick_startup_file", { mode });
  },

  async pickServerExecutable(): Promise<{ path: string; mode: "jar" | "bat" | "sh" } | null> {
    const result = await tauriInvoke<[string, string] | null>("pick_server_executable");
    if (result) {
      return { path: result[0], mode: result[1] as "jar" | "bat" | "sh" };
    }
    return null;
  },

  async pickJavaFile(): Promise<string | null> {
    return tauriInvoke("pick_java_file");
  },

  async pickSaveFile(): Promise<string | null> {
    return tauriInvoke("pick_save_file");
  },

  async pickFolder(): Promise<string | null> {
    return tauriInvoke("pick_folder");
  },

  async pickImageFile(): Promise<string | null> {
    return tauriInvoke("pick_image_file");
  },

  async openFile(path: string): Promise<void> {
    return tauriInvoke("open_file", { path });
  },

  async openFolder(path: string): Promise<void> {
    return tauriInvoke("open_folder", { path });
  },
};
