import { tauriInvoke, isBrowserEnv, HTTP_API_BASE } from "@api/tauri";
import type { ServerInstance } from "@type/server";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ServerStatusInfo {
  id: string;
  status: "Stopped" | "Starting" | "Running" | "Stopping" | "Error";
  pid: number | null;
  uptime: number | null;
}

export interface ParsedServerCoreInfo {
  coreType: string;
  mainClass: string | null;
  jarPath: string | null;
}

export interface ServerLogLineEvent {
  server_id: string;
  line: string;
}

export interface StartupCandidateItem {
  id: string;
  mode: "starter" | "jar" | "bat" | "sh" | "ps1";
  label: string;
  detail: string;
  path: string;
  recommended: number;
}

export interface StartupScanResult {
  parsedCore: ParsedServerCoreInfo;
  candidates: StartupCandidateItem[];
  detectedCoreTypeKey: string | null;
  coreTypeOptions: string[];
  mcVersionOptions: string[];
  detectedMcVersion: string | null;
  mcVersionDetectionFailed: boolean;
}

interface ParsedServerCoreInfoRaw {
  core_type: string;
  main_class: string | null;
  jar_path: string | null;
}

interface StartupCandidateItemRaw {
  id: string;
  mode: string;
  label: string;
  detail: string;
  path: string;
  recommended: number;
}

interface StartupScanResultRaw {
  parsed_core: ParsedServerCoreInfoRaw;
  candidates: StartupCandidateItemRaw[];
  detected_core_type_key: string | null;
  core_type_options: string[];
  mc_version_options: string[];
  detected_mc_version: string | null;
  mc_version_detection_failed: boolean;
}

export const serverApi = {
  async create(params: {
    name: string;
    coreType: string;
    mcVersion: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    javaPath: string;
    jarPath: string;
    startupMode?: "jar" | "bat" | "sh" | "ps1";
  }): Promise<ServerInstance> {
    return tauriInvoke("create_server", {
      name: params.name,
      coreType: params.coreType,
      mcVersion: params.mcVersion,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      javaPath: params.javaPath,
      jarPath: params.jarPath,
      startupMode: params.startupMode ?? "jar",
    });
  },

  async importServer(params: {
    name: string;
    jarPath: string;
    startupMode: "jar" | "bat" | "sh" | "ps1";
    javaPath: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    onlineMode: boolean;
  }): Promise<ServerInstance> {
    return tauriInvoke("import_server", {
      name: params.name,
      jarPath: params.jarPath,
      startupMode: params.startupMode,
      javaPath: params.javaPath,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      onlineMode: params.onlineMode,
    });
  },

  async importModpack(params: {
    name: string;
    modpackPath: string;
    javaPath: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    startupMode: "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";
    onlineMode: boolean;
    customCommand?: string;
    runPath: string;
    startupFilePath?: string;
    coreType?: string;
    mcVersion?: string;
  }): Promise<ServerInstance> {
    return tauriInvoke("import_modpack", {
      name: params.name,
      modpackPath: params.modpackPath,
      javaPath: params.javaPath,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      startupMode: params.startupMode,
      onlineMode: params.onlineMode,
      customCommand: params.customCommand,
      runPath: params.runPath,
      startupFilePath: params.startupFilePath,
      coreType: params.coreType,
      mcVersion: params.mcVersion,
    });
  },

  async parseServerCoreType(sourcePath: string): Promise<ParsedServerCoreInfo> {
    const result = await tauriInvoke<ParsedServerCoreInfoRaw>("parse_server_core_type", {
      sourcePath,
    });
    return {
      coreType: result.core_type,
      mainClass: result.main_class,
      jarPath: result.jar_path,
    };
  },

  async scanStartupCandidates(
    sourcePath: string,
    sourceType: "archive" | "folder",
  ): Promise<StartupScanResult> {
    const result = await tauriInvoke<StartupScanResultRaw>("scan_startup_candidates", {
      sourcePath,
      sourceType,
    });

    return {
      parsedCore: {
        coreType: result.parsed_core.core_type,
        mainClass: result.parsed_core.main_class,
        jarPath: result.parsed_core.jar_path,
      },
      candidates: result.candidates.map((item) => ({
        id: item.id,
        mode: (item.mode as StartupCandidateItem["mode"]) ?? "jar",
        label: item.label,
        detail: item.detail,
        path: item.path,
        recommended: item.recommended,
      })),
      detectedCoreTypeKey: result.detected_core_type_key,
      coreTypeOptions: result.core_type_options,
      mcVersionOptions: result.mc_version_options,
      detectedMcVersion: result.detected_mc_version,
      mcVersionDetectionFailed: result.mc_version_detection_failed,
    };
  },

  async collectCopyConflicts(sourceDir: string, targetDir: string): Promise<string[]> {
    return tauriInvoke("collect_copy_conflicts", { sourceDir, targetDir });
  },

  async copyDirectoryContents(sourceDir: string, targetDir: string): Promise<void> {
    return tauriInvoke("copy_directory_contents", { sourceDir, targetDir });
  },

  async addExistingServer(params: {
    name: string;
    serverPath: string;
    javaPath: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    startupMode: "jar" | "bat" | "sh" | "ps1";
    executablePath?: string;
  }): Promise<ServerInstance> {
    return tauriInvoke("add_existing_server", {
      name: params.name,
      serverPath: params.serverPath,
      javaPath: params.javaPath,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      startupMode: params.startupMode,
      executablePath: params.executablePath,
    });
  },

  async start(id: string): Promise<void> {
    return tauriInvoke("start_server", { id });
  },

  async stop(id: string): Promise<void> {
    return tauriInvoke("stop_server", { id });
  },

  async sendCommand(id: string, command: string): Promise<void> {
    return tauriInvoke("send_command", { id, command });
  },

  async getList(): Promise<ServerInstance[]> {
    return tauriInvoke("get_server_list");
  },

  async getStatus(id: string): Promise<ServerStatusInfo> {
    return tauriInvoke("get_server_status", { id });
  },

  async deleteServer(id: string): Promise<void> {
    return tauriInvoke("delete_server", { id });
  },

  async getLogs(id: string, since: number, maxLines?: number): Promise<string[]> {
    return tauriInvoke("get_server_logs", { id, since, maxLines });
  },

  onLogLine(callback: (payload: ServerLogLineEvent) => void): Promise<UnlistenFn> {
    // 浏览器环境使用 SSE
    if (isBrowserEnv()) {
      return this.subscribeLogStream(callback);
    }
    // Tauri 环境使用事件监听
    return listen<ServerLogLineEvent>("server-log-line", (event) => {
      callback(event.payload);
    });
  },

  /**
   * SSE 日志流订阅（浏览器/Docker 模式）
   * 返回取消订阅函数
   */
  subscribeLogStream(callback: (payload: ServerLogLineEvent) => void): Promise<UnlistenFn> {
    return new Promise((resolve) => {
      const url = `${HTTP_API_BASE}/api/logs/stream`;
      const eventSource = new EventSource(url);

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as ServerLogLineEvent;
          callback(data);
        } catch (e) {
          console.warn("[SSE] Failed to parse log event:", e);
        }
      };

      eventSource.onerror = (e) => {
        console.warn("[SSE] Connection error, reconnecting...", e);
        // 自动重连：关闭旧连接，延迟后创建新连接
        eventSource.close();
        setTimeout(() => {
          this.subscribeLogStream(callback);
        }, 3000);
      };

      // 返回取消订阅函数
      resolve(() => {
        eventSource.close();
      });
    });
  },

  async updateServerName(id: string, name: string): Promise<void> {
    return tauriInvoke("update_server_name", { id, name });
  },

  async validateServerPath(newPath: string): Promise<{
    valid: boolean;
    message: string;
    jarPath: string | null;
    startupMode: string | null;
  }> {
    const result = await tauriInvoke<{
      valid: boolean;
      message: string;
      jar_path: string | null;
      startup_mode: string | null;
    }>("validate_server_path", { newPath });
    return {
      valid: result.valid,
      message: result.message,
      jarPath: result.jar_path,
      startupMode: result.startup_mode,
    };
  },

  async updateServerPath(
    id: string,
    newPath: string,
    newJarPath?: string,
    newStartupMode?: string,
  ): Promise<ServerInstance> {
    return tauriInvoke("update_server_path", {
      id,
      newPath,
      newJarPath,
      newStartupMode,
    });
  },
};
