import { tauriInvoke, isBrowserEnv, HTTP_API_BASE } from "@api/tauri";
import { invoke } from "@api/invoke";
import type { ServerInstance } from "@type/server";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ServerStatusInfo {
  id: string;
  // Unknown 用于后端返回了未识别的状态，避免误判为已停止
  status: "Stopped" | "Starting" | "Running" | "Stopping" | "Error" | "Unknown";
  pid: number | null;
  uptime: number | null;
}

export interface ParsedServerCoreInfo {
  coreType: string;
  mainClass: string | null;
  jarPath: string | null;
}

export interface ServerLogLineEvent {
  instance_id: string;
  line: ConsoleLogLine;
}

export interface ConsoleLogLine {
  // Rust i64 经 Tauri/JSON 序列化后到达前端是 number，不能用 bigint
  sequence: number;
  timestamp: number;
  source: string;
  line: string;
}

export interface ForceStopPreparation {
  token: string;
  expiresAt: number;
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

/** 后端 Instance 原始结构，字段和前端 ServerInstance 差异较大需转换 */
interface InstanceRaw {
  id: string;
  name: string;
  aliases: string[];
  core_type: string;
  core_version: string;
  game_version: string;
  directory: string;
  port: number;
  max_memory_mib: number;
  min_memory_mib: number;
  created_at_unix_secs: number;
  last_started_at_unix_secs: number | null;
  server_metadata: unknown;
  launch: LocalLaunchRaw;
}

/** 后端启动配置，前端把部分字段平铺到 ServerInstance */
interface LocalLaunchRaw {
  startup_mode: string;
  startup_target: string | null;
  custom_command: string | null;
  custom_executable: string | null;
  custom_arguments: string[];
  java_executable: string | null;
  jvm_arguments: string[];
}

/** 后端 ServerSnapshot 原始结构 */
interface ServerSnapshotRaw {
  instance_id: string;
  state: string;
  pid: number | null;
  uptime_secs: number | null;
  error_message: string | null;
}

/** Instance 原始结构转前端 ServerInstance */
function toServerInstance(i: InstanceRaw): ServerInstance {
  return {
    id: i.id,
    name: i.name,
    core_type: i.core_type,
    core_version: i.core_version,
    mc_version: i.game_version,
    path: i.directory,
    jar_path: i.launch.startup_target ?? "",
    startup_mode: i.launch.startup_mode as ServerInstance["startup_mode"],
    custom_command: i.launch.custom_command,
    java_path: i.launch.java_executable ?? "",
    max_memory: i.max_memory_mib,
    min_memory: i.min_memory_mib,
    jvm_args: i.launch.jvm_arguments,
    port: i.port,
    created_at: i.created_at_unix_secs,
    last_started_at: i.last_started_at_unix_secs,
  };
}

/** 后端 state 小写枚举转前端 PascalCase 状态 */
function toServerStatusInfo(s: ServerSnapshotRaw): ServerStatusInfo {
  const stateMap: Record<string, ServerStatusInfo["status"]> = {
    starting: "Starting",
    running: "Running",
    stopping: "Stopping",
    stopped: "Stopped",
    error: "Error",
    crashed: "Error",
  };

  const mappedStatus = stateMap[s.state];

  if (!mappedStatus) {
    // 未知的后端状态，避免默认为 Stopped 造成误导
    console.warn("Unknown server state from backend:", {
      state: s.state,
      snapshot: s,
    });
  }

  return {
    id: s.instance_id,
    status: mappedStatus ?? "Unknown",
    pid: s.pid,
    uptime: s.uptime_secs,
  };
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
    await invoke("start_server", { id });
  },

  async stop(id: string): Promise<void> {
    await invoke("stop_server", { id });
  },

  async prepareForceStop(id: string): Promise<ForceStopPreparation> {
    return tauriInvoke("prepare_force_stop_server", { id });
  },

  async forceStop(id: string, confirmationToken: string): Promise<void> {
    // Tauri 模式透传 confirmationToken，后端当前未校验，但保留以备未来启用
    // Axum 模式后端 force_stop 只收 id，不发送 token
    await invoke("force_stop_server", { id, confirmationToken });
  },

  async sendCommand(id: string, command: string): Promise<void> {
    await invoke("send_server_command", { id, command });
  },

  async getList(): Promise<ServerInstance[]> {
    const raw = await invoke<InstanceRaw[]>("list_instances");
    return raw.map(toServerInstance);
  },

  async getStatus(id: string): Promise<ServerStatusInfo> {
    const raw = await invoke<ServerSnapshotRaw>("server_status", { id });
    return toServerStatusInfo(raw);
  },

  async deleteServer(id: string): Promise<void> {
    await invoke("delete_instance", { id });
  },

  async getLogs(id: string, since: number, maxLines?: number): Promise<ConsoleLogLine[]> {
    // 后端命令参数为 recent_limit，不传则可能一次性返回全部日志
    return tauriInvoke("get_server_logs", { id, since, recent_limit: maxLines });
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
      // 取消后不再重连;重连定时器与当前连接统一由 unlisten 管理
      let cancelled = false;
      let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
      let currentEventSource: EventSource | null = null;

      const connect = (): void => {
        if (cancelled) return;
        const eventSource = new EventSource(url);
        currentEventSource = eventSource;

        eventSource.addEventListener("message", (event) => {
          try {
            const data = JSON.parse(event.data) as ServerLogLineEvent;
            callback(data);
          } catch (e) {
            console.warn("[SSE] Failed to parse log event:", e);
          }
        });

        eventSource.addEventListener("error", (e) => {
          console.warn("[SSE] Connection error, reconnecting...", e);
          eventSource.close();
          if (currentEventSource === eventSource) currentEventSource = null;
          // 自动重连:延迟后创建新连接,取消后不再续连
          if (!cancelled) {
            reconnectTimer = setTimeout(connect, 3000);
          }
        });
      };

      connect();

      // 返回取消订阅函数:关闭当前连接并阻止重连链
      resolve(() => {
        cancelled = true;
        if (reconnectTimer) {
          clearTimeout(reconnectTimer);
          reconnectTimer = null;
        }
        if (currentEventSource) {
          currentEventSource.close();
          currentEventSource = null;
        }
      });
    });
  },

  async updateServerName(id: string, name: string): Promise<void> {
    await invoke("rename_instance", { id, name });
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
