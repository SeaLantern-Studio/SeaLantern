import { tauriInvoke } from "@api/tauri";
import { invoke } from "@api/invoke";
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

export interface IPv6TestTarget {
  target: string;
  address: string;
  error: string;
  kind: string;
}

export interface IPv6TestResult {
  supported: boolean;
  message: string;
  detail?: string;
  error_kind?: string;
  targets?: IPv6TestTarget[];
}

/** 后端 SystemSnapshot 原始结构，和前端 SystemInfo 字段有差异需转换 */
interface SystemSnapshotRaw {
  os: string;
  arch: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  host_name: string;
  cpu: CpuInfo;
  memory: MemoryInfo;
  swap: SwapInfo;
  disk: {
    total: number;
    used: number;
    available: number;
    usage: number;
    disks: DiskDetailRaw[];
  };
  networks: { interface: string; received: number; transmitted: number }[];
  uptime: number;
  process_count: number;
}

/** 后端单个磁盘信息，比前端 DiskDetail 少 usage 字段需计算补齐 */
interface DiskDetailRaw {
  name: string;
  mount_point: string;
  file_system: string;
  total: number;
  used: number;
  available: number;
  is_removable: boolean;
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
    const raw = await invoke<SystemSnapshotRaw>("get_system_snapshot");
    // 后端 networks 是数组，前端 NetworkInfo 要汇总总量并映射字段名
    return {
      os: raw.os,
      arch: raw.arch,
      os_name: raw.os_name,
      os_version: raw.os_version,
      kernel_version: raw.kernel_version,
      host_name: raw.host_name,
      cpu: raw.cpu,
      memory: raw.memory,
      swap: raw.swap,
      disk: {
        total: raw.disk.total,
        used: raw.disk.used,
        available: raw.disk.available,
        usage: raw.disk.usage,
        disks: raw.disk.disks.map((d) => ({
          ...d,
          // 后端 DiskInfo 没有 usage，按容量占比计算
          usage: d.total > 0 ? (d.used / d.total) * 100 : 0,
        })),
      },
      network: {
        total_received: raw.networks.reduce((s, n) => s + n.received, 0),
        total_transmitted: raw.networks.reduce((s, n) => s + n.transmitted, 0),
        interfaces: raw.networks.map((n) => ({
          name: n.interface,
          received: n.received,
          transmitted: n.transmitted,
        })),
      },
      uptime: raw.uptime,
      process_count: raw.process_count,
    };
  },

  async getServerResourceUsage(serverId: string): Promise<ServerResourceUsage> {
    // 后端命令参数为 instance_id，走统一 invoke 入口，Tauri 与 Axum 模式契约一致
    return invoke<ServerResourceUsage>("get_server_resource_usage", { instance_id: serverId });
  },

  async pickJarFile(): Promise<string | null> {
    if (isUploadSupported()) {
      return this.pickAndUploadBrowserFile(".jar");
    }
    return tauriInvoke("desktop_pick_jar_file");
  },

  async pickArchiveFile(): Promise<string | null> {
    if (isUploadSupported()) {
      return this.pickAndUploadBrowserFile(".zip,.tar,.tar.gz,.tgz,.jar");
    }
    return tauriInvoke("desktop_pick_archive_file");
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
    return tauriInvoke("desktop_pick_startup_file", { mode });
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
    const result = await tauriInvoke<[string, string] | null>("desktop_pick_server_executable");
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
    return tauriInvoke("desktop_pick_java_file");
  },

  async pickSaveFile(): Promise<string | null> {
    if (isUploadSupported()) {
      throw new Error("Docker环境不支持原生文件选择器，请使用文件上传功能");
    }
    return tauriInvoke("desktop_pick_save_file");
  },

  async pickFolder(): Promise<string | null> {
    if (isUploadSupported()) {
      throw new Error("Docker环境不支持原生文件选择器，请使用文件上传功能");
    }
    return tauriInvoke("desktop_pick_folder");
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
    return tauriInvoke("desktop_pick_image_file");
  },

  async openFile(path: string): Promise<void> {
    if (isUploadSupported()) {
      throw new Error("Docker环境不支持从浏览器直接打开本地文件");
    }
    return tauriInvoke("open_file", { path });
  },

  async openFolder(path: string): Promise<void> {
    if (isUploadSupported()) {
      throw new Error("Docker环境不支持从浏览器直接打开本地文件夹");
    }
    return tauriInvoke("open_folder", { path });
  },

  async getDefaultRunPath(): Promise<string> {
    return tauriInvoke("get_default_run_path");
  },

  async getSafeModeStatus(): Promise<boolean> {
    return tauriInvoke("get_safe_mode_status");
  },

  async testIPv6Connectivity(): Promise<IPv6TestResult> {
    return tauriInvoke("test_ipv6_connectivity");
  },

  /** 删除指定文件 */
  async removeFile(path: string): Promise<void> {
    return tauriInvoke("remove_file", { path });
  },
};
