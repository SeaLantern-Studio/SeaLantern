import {
  tauriInvoke,
  isBrowserEnv,
  HTTP_API_BASE,
  ensureBrowserSession,
  notifyBrowserUnauthorized,
  readBrowserAuthToken,
} from "@api/tauri";
import type { ServerStatus } from "@type/common";
import type {
  CpuPolicyConfig,
  DockerLaunchDetail,
  JvmPresetConfig,
  LocalLaunchDetail,
  ServerInstance,
} from "@type/server";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isActiveServerStatus } from "@utils/serverStatus";

export interface ServerStatusInfo {
  id: string;
  status: ServerStatus;
  pid: number | null;
  uptime: number | null;
  detail_message?: string | null;
  error_message?: string | null;
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

export interface ServerLogStructuredEvent {
  server_id: string;
  line: string;
  stream: "stdout" | "stderr" | "unknown";
  event_kind: "server_ready" | "player_join" | "player_leave" | "chat" | "error" | null;
  player: string | null;
  message: string | null;
}

export interface ServerRuntimeEventPayloadRawLine {
  type: "raw_line";
  line: string;
  stream: string;
}

export interface ServerRuntimeEventPayloadStructuredLog {
  type: "structured_log";
  line: string;
  stream: string;
  event_kind: string | null;
  player: string | null;
  message: string | null;
}

export interface ServerRuntimeEventPayloadCommand {
  type: "command";
  command: string;
  success: boolean | null;
  error: string | null;
  actor: string;
}

export interface ServerRuntimeEventPayloadLifecycle {
  type: "lifecycle";
  detail: string | null;
  error: string | null;
  from_mode: string | null;
  to_mode: string | null;
}

export interface ServerRuntimeEvent {
  event_id: string;
  occurred_at: number;
  scope: "server";
  server_id: string;
  source: string;
  kind: string;
  payload:
    | ServerRuntimeEventPayloadRawLine
    | ServerRuntimeEventPayloadStructuredLog
    | ServerRuntimeEventPayloadCommand
    | ServerRuntimeEventPayloadLifecycle;
}

export interface AppOperationEvent {
  event_id: string;
  occurred_at: number;
  scope: "app";
  action: string;
  source: string;
  kind: "operation_requested" | "operation_succeeded" | "operation_failed";
  payload: {
    type: "operation";
    action: string;
    detail: string | null;
    error: string | null;
  };
}

export interface ForceStopPreparation {
  token: string;
  expiresAt: number;
}

export interface ForceStopAllFailure {
  serverId: string;
  error: string;
}

export interface ForceStopAllResult {
  attemptedServerIds: string[];
  failed: ForceStopAllFailure[];
}

interface ForceStopPreparationRaw {
  token: string;
  expires_at: number;
}

function formatForceStopError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

async function readSseStream(
  reader: ReadableStreamDefaultReader<string>,
  callback: (payload: ServerLogLineEvent) => void,
  isDisposed: () => boolean,
  buffer: string = "",
): Promise<void> {
  // Keep the trailing partial frame between reads; SSE payload boundaries do not align
  // with chunk boundaries, so parsing line-by-line would randomly truncate log events.
  const { value, done } = await reader.read();
  if (done || isDisposed()) {
    return;
  }

  let nextBuffer = `${buffer}${value}`;
  const frames = nextBuffer.split("\n\n");
  nextBuffer = frames.pop() || "";

  for (const frame of frames) {
    const dataLines = frame
      .split("\n")
      .filter((line) => line.startsWith("data:"))
      .map((line) => line.slice(5).trim());

    if (dataLines.length === 0) {
      continue;
    }

    const raw = dataLines.join("\n");
    if (raw === "ping") {
      continue;
    }

    try {
      const data = JSON.parse(raw) as ServerLogLineEvent;
      callback(data);
    } catch (error) {
      console.warn("[SSE] Failed to parse log event:", error);
    }
  }

  await readSseStream(reader, callback, isDisposed, nextBuffer);
}

export interface StartupCandidateItem {
  id: string;
  mode: "starter" | "jar" | "bat" | "sh" | "ps1" | "custom";
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

interface LocalLaunchDetailRaw {
  startupMode: LocalLaunchDetail["startup_mode"];
  javaPath: string;
  launchTarget: string;
  effectiveJvmArgs: string[];
  commandPreview: string;
}

interface DockerLaunchDetailRaw {
  runtimeKind: DockerLaunchDetail["runtime_kind"];
  image: string;
  imageTag: string;
  containerName: string;
  cpusetApplied: string | null;
  jvmPreset: DockerLaunchDetail["jvm_preset"];
  jvmOptsPreview: string | null;
  jvmXxOptsPreview: string | null;
  jvmOptsArgsCount: number;
  jvmXxOptsArgsCount: number;
  jvmOptsOverriddenByRuntimeEnv: boolean;
  jvmXxOptsOverriddenByRuntimeEnv: boolean;
  activeProcessorCountStatus: DockerLaunchDetail["active_processor_count_status"];
  activeProcessorCountValue: number | null;
  dockerArgsPreview: string[];
  commandPreview: string;
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
    jvmArgs?: string[];
    cpuPolicy?: CpuPolicyConfig;
    jvmPreset?: JvmPresetConfig;
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
      jvmArgs: params.jvmArgs ?? [],
      cpuPolicy: params.cpuPolicy,
      jvmPreset: params.jvmPreset,
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
    jvmArgs?: string[];
    cpuPolicy?: CpuPolicyConfig;
    jvmPreset?: JvmPresetConfig;
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
      jvmArgs: params.jvmArgs ?? [],
      cpuPolicy: params.cpuPolicy,
      jvmPreset: params.jvmPreset,
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
    jvmArgs?: string[];
    cpuPolicy?: CpuPolicyConfig;
    jvmPreset?: JvmPresetConfig;
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
      jvmArgs: params.jvmArgs ?? [],
      cpuPolicy: params.cpuPolicy,
      jvmPreset: params.jvmPreset,
    });
  },

  async parseServerCoreKey(sourcePath: string): Promise<ParsedServerCoreInfo> {
    const result = await tauriInvoke<ParsedServerCoreInfoRaw>("parse_server_core_key", {
      sourcePath,
    });
    return {
      coreType: result.core_type,
      mainClass: result.main_class,
      jarPath: result.jar_path,
    };
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
    startupMode: "jar" | "bat" | "sh" | "ps1" | "custom";
    executablePath?: string;
    customCommand?: string;
    coreType?: string;
    mcVersion?: string;
    jvmArgs?: string[];
    cpuPolicy?: CpuPolicyConfig;
    jvmPreset?: JvmPresetConfig;
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
      customCommand: params.customCommand,
      coreType: params.coreType,
      mcVersion: params.mcVersion,
      jvmArgs: params.jvmArgs ?? [],
      cpuPolicy: params.cpuPolicy,
      jvmPreset: params.jvmPreset,
    });
  },

  async start(id: string): Promise<void> {
    return tauriInvoke("start_server", { id });
  },

  async stop(id: string): Promise<void> {
    return tauriInvoke("stop_server", { id });
  },

  async prepareForceStop(id: string): Promise<ForceStopPreparation> {
    const result = await tauriInvoke<ForceStopPreparationRaw>("prepare_force_stop_server", { id });
    return {
      token: result.token,
      expiresAt: result.expires_at,
    };
  },

  async forceStopAll(): Promise<ForceStopAllResult> {
    // Force-stop is intentionally best-effort across the fleet: one server failing the
    // confirmation flow must not prevent attempts on the remaining active instances.
    const servers = await this.getList();
    const activeServerIds = await Promise.all(
      servers.map(async (server) => {
        try {
          const status = await this.getStatus(server.id);
          return isActiveServerStatus(status.status) ? server.id : null;
        } catch {
          return null;
        }
      }),
    );

    const filteredActiveServerIds = activeServerIds.filter(
      (serverId): serverId is string => serverId !== null,
    );
    if (filteredActiveServerIds.length === 0) {
      return {
        attemptedServerIds: [],
        failed: [],
      };
    }

    const failures = await Promise.all(
      filteredActiveServerIds.map(async (serverId) => {
        try {
          const { token } = await this.prepareForceStop(serverId);
          await this.forceStop(serverId, token);
          return null;
        } catch (err) {
          return {
            serverId,
            error: formatForceStopError(err),
          } satisfies ForceStopAllFailure;
        }
      }),
    );

    return {
      attemptedServerIds: filteredActiveServerIds,
      failed: failures.filter((failure): failure is ForceStopAllFailure => failure !== null),
    };
  },

  async forceStop(id: string, confirmationToken: string): Promise<void> {
    return tauriInvoke("force_stop_server", { id, confirmationToken });
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

  async getLocalLaunchDetail(id: string): Promise<LocalLaunchDetail> {
    const result = await tauriInvoke<LocalLaunchDetailRaw>("get_local_launch_detail", { id });
    return {
      startup_mode: result.startupMode,
      java_path: result.javaPath,
      launch_target: result.launchTarget,
      effective_jvm_args: result.effectiveJvmArgs,
      command_preview: result.commandPreview,
    };
  },

  async getDockerLaunchDetail(id: string): Promise<DockerLaunchDetail> {
    const result = await tauriInvoke<DockerLaunchDetailRaw>("get_docker_launch_detail", { id });
    return {
      runtime_kind: result.runtimeKind,
      image: result.image,
      image_tag: result.imageTag,
      container_name: result.containerName,
      cpuset_applied: result.cpusetApplied,
      jvm_preset: result.jvmPreset,
      jvm_opts_preview: result.jvmOptsPreview,
      jvm_xx_opts_preview: result.jvmXxOptsPreview,
      jvm_opts_args_count: result.jvmOptsArgsCount,
      jvm_xx_opts_args_count: result.jvmXxOptsArgsCount,
      jvm_opts_overridden_by_runtime_env: result.jvmOptsOverriddenByRuntimeEnv,
      jvm_xx_opts_overridden_by_runtime_env: result.jvmXxOptsOverriddenByRuntimeEnv,
      active_processor_count_status: result.activeProcessorCountStatus,
      active_processor_count_value: result.activeProcessorCountValue,
      docker_args_preview: result.dockerArgsPreview,
      command_preview: result.commandPreview,
    };
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

  onStructuredLogEvent(callback: (payload: ServerLogStructuredEvent) => void): Promise<UnlistenFn> {
    if (isBrowserEnv()) {
      return Promise.reject(new Error("Structured log events are only available in Tauri mode"));
    }
    return listen<ServerLogStructuredEvent>("server-log-structured", (event) => {
      callback(event.payload);
    });
  },

  onRuntimeEvent(callback: (payload: ServerRuntimeEvent) => void): Promise<UnlistenFn> {
    if (isBrowserEnv()) {
      return Promise.reject(new Error("Runtime events are only available in Tauri mode"));
    }
    return listen<ServerRuntimeEvent>("server-runtime-event", (event) => {
      callback(event.payload);
    });
  },

  onAppOperationEvent(callback: (payload: AppOperationEvent) => void): Promise<UnlistenFn> {
    if (isBrowserEnv()) {
      return Promise.reject(new Error("App operation events are only available in Tauri mode"));
    }
    return listen<AppOperationEvent>("app-operation-event", (event) => {
      callback(event.payload);
    });
  },

  async getRecentAppOperationEvents(limit?: number): Promise<AppOperationEvent[]> {
    if (isBrowserEnv()) {
      return Promise.reject(new Error("App operation events are only available in Tauri mode"));
    }
    return tauriInvoke("get_recent_app_operation_events", { limit });
  },

  /**
   * SSE 日志流订阅（浏览器/Docker 模式）
   * 返回取消订阅函数
   */
  subscribeLogStream(callback: (payload: ServerLogLineEvent) => void): Promise<UnlistenFn> {
    return new Promise((resolve, reject) => {
      void (async () => {
        try {
          await ensureBrowserSession();
        } catch (error) {
          reject(error);
          return;
        }

        const url = `${HTTP_API_BASE}/api/logs/stream`;
        const token = readBrowserAuthToken();
        if (!token) {
          reject(new Error("Missing HTTP auth token for log stream"));
          return;
        }

        let abortController: AbortController | null = null;
        let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
        let disposed = false;

        const clearReconnectTimer = () => {
          if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
          }
        };

        const connect = () => {
          if (disposed) {
            return;
          }

          abortController = new AbortController();

          void (async () => {
            try {
              const response = await fetch(url, {
                method: "GET",
                headers: {
                  Authorization: `Bearer ${token}`,
                },
                signal: abortController?.signal,
              });

              if (response.status === 401) {
                notifyBrowserUnauthorized("auth.message_session_expired");
                throw new Error("SSE unauthorized");
              }

              if (!response.ok || !response.body) {
                throw new Error(`SSE stream failed with HTTP ${response.status}`);
              }

              const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
              await readSseStream(reader, callback, () => disposed);
            } catch (error) {
              if (disposed) {
                return;
              }

              // Browser log streaming should self-heal on transient backend restarts. The
              // explicit timer also prevents recursive immediate reconnect storms on failure.
              console.warn("[SSE] Connection error, reconnecting...", error);
              abortController?.abort();
              abortController = null;
              clearReconnectTimer();
              reconnectTimer = setTimeout(() => {
                reconnectTimer = null;
                connect();
              }, 3000);
            }
          })();
        };

        connect();

        resolve(() => {
          disposed = true;
          clearReconnectTimer();
          abortController?.abort();
          abortController = null;
        });
      })();
    });
  },

  async updateServerName(id: string, name: string): Promise<void> {
    return tauriInvoke("update_server_name", { id, name });
  },

  async updateServerJavaPath(id: string, javaPath: string): Promise<ServerInstance> {
    return tauriInvoke("update_server_java_path", { id, javaPath });
  },

  async validateServerPath(newPath: string): Promise<{
    valid: boolean;
    message: string;
    messageKey: string | null;
    jarPath: string | null;
    startupMode: string | null;
  }> {
    const result = await tauriInvoke<{
      valid: boolean;
      message: string;
      message_key?: string | null;
      jar_path: string | null;
      startup_mode: string | null;
    }>("validate_server_path", { newPath });
    return {
      valid: result.valid,
      message: result.message,
      messageKey: result.message_key ?? null,
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
