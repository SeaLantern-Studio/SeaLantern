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

/** 服务器检测报告（对应后端 ServerInspectionReport） */
interface ServerInspectionReport {
  schema_version: number;
  subject: InspectionSubject;
  artifact: ArtifactInfo;
  identity: ServerIdentityInfo;
  minecraft?: MinecraftVersionInfo;
  java: JavaRequirementInfo;
  components: Array<Attributed<ServerComponent>>;
  launches: Array<Attributed<LaunchProfile>>;
  evidence: DetectionEvidence[];
  diagnostics: InspectionDiagnostic[];
}

interface InspectionSubject {
  path: string;
  kind: "file" | "directory";
  size_bytes?: number;
  modified_at_unix_secs?: number;
}

interface ArtifactInfo {
  format: Detected<string>;
  roles: Array<Attributed<string>>;
  main_class: Detected<string>;
}

interface ServerIdentityInfo {
  category: Detected<string>;
  implementation: Detected<string>;
  version: Detected<string>;
}

interface MinecraftVersionInfo {
  version?: Detected<string>;
}

interface JavaRequirementInfo {
  minimum_major?: number;
  maximum_major?: number;
}

interface LaunchProfile {
  id: string;
  platform: "any" | "windows" | "unix";
  working_directory?: string;
  target: LaunchTarget;
  jvm_arguments: string[];
  program_arguments: string[];
}

interface LaunchTarget {
  kind: "jar" | "main_class" | "argument_files" | "script";
  path?: string;
  class_name?: string;
  paths?: string[];
}

interface Attributed<T> {
  value: T;
  confidence: number;
}

interface Detected<T> {
  value?: T;
  confidence: number;
  alternatives?: Array<{ value: T; confidence: number }>;
}

interface ServerComponent {
  kind: string;
}

interface DetectionEvidence {
  id: number;
  detector: string;
}

interface InspectionDiagnostic {
  level: string;
  message: string;
}

/** 从路径中提取文件名 */
function getFileName(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

/** 从检测报告中提取核心类型选项 */
function extractCoreTypeOptions(report: ServerInspectionReport): string[] {
  const options: Set<string> = new Set();

  // 主检测结果
  if (report.identity.implementation.value) {
    options.add(report.identity.implementation.value);
  }

  // 备选检测结果
  if (report.identity.implementation.alternatives) {
    for (const alt of report.identity.implementation.alternatives) {
      if (alt.value) {
        options.add(alt.value);
      }
    }
  }

  // 从组件中提取
  for (const component of report.components) {
    if (component.value.kind) {
      options.add(component.value.kind);
    }
  }

  return Array.from(options);
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

  /**
   * 扫描服务器目录的启动候选项
   *
   * 使用 inspect_server 命令检测服务器信息，从 LaunchProfile 构建候选列表
   */
  async scanStartupCandidates(
    sourcePath: string,
    _sourceType: "archive" | "folder",
  ): Promise<StartupScanResult> {
    // 使用 inspect_server 获取服务器检测报告
    const report = await invoke<ServerInspectionReport>("inspect_server", { path: sourcePath });

    // 从 LaunchProfile 构建启动候选项列表
    const candidates: StartupCandidateItem[] = report.launches.map((launch, index) => {
      const target = launch.value.target;
      let mode: StartupCandidateItem["mode"] = "jar";
      let path = "";
      let label = "";

      switch (target.kind) {
        case "jar":
          mode = "jar";
          path = target.path;
          label = `JAR: ${getFileName(path)}`;
          break;
        case "script":
          // 根据脚本扩展名确定模式
          const scriptPath = target.path;
          if (scriptPath.endsWith(".bat")) {
            mode = "bat";
          } else if (scriptPath.endsWith(".sh")) {
            mode = "sh";
          } else if (scriptPath.endsWith(".ps1")) {
            mode = "ps1";
          }
          path = scriptPath;
          label = `Script: ${getFileName(scriptPath)}`;
          break;
        case "main_class":
          mode = "jar";
          path = report.subject.path;
          label = `Main Class: ${target.class_name}`;
          break;
        default:
          path = report.subject.path;
          label = `Launch ${index + 1}`;
      }

      return {
        id: launch.value.id,
        mode,
        label,
        detail: `Platform: ${launch.value.platform}`,
        path,
        recommended: 100 - index * 10, // 第一个推荐度最高
      };
    });

    // 如果没有检测到启动项,添加默认候选项
    if (candidates.length === 0) {
      candidates.push({
        id: "default",
        mode: "jar",
        label: "Default JAR",
        detail: "Use detected JAR file",
        path: report.subject.path,
        recommended: 50,
      });
    }

    // 提取核心类型信息
    const coreTypeValue = report.identity.implementation.value;
    const coreTypeOptions = extractCoreTypeOptions(report);

    // 提取 Minecraft 版本信息
    const mcVersion = report.minecraft?.version?.value ?? null;
    const mcVersionOptions = report.minecraft?.version?.alternatives?.map((a) => a.value) ?? [];

    return {
      parsedCore: {
        coreType: coreTypeValue ?? "",
        mainClass: report.artifact.main_class.value ?? null,
        jarPath: report.subject.path,
      },
      candidates,
      detectedCoreTypeKey: coreTypeValue ?? null,
      coreTypeOptions,
      mcVersionOptions,
      detectedMcVersion: mcVersion,
      mcVersionDetectionFailed: !mcVersion && report.minecraft === null,
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
