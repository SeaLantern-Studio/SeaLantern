/**
 * 统一 RPC 适配层
 *
 * 前端用点分方法名调用，适配层按运行环境分发到 Tauri 命令或 Axum 路由。
 * 后端缺失的方法抛 NotImplementedError，等后期补映射表即可接通。
 * application 层不动，两侧后端各自映射。
 */

import { isBrowserEnv, HTTP_API_BASE } from "@api/tauri";
import { handleError, AppError, ErrorType } from "@utils/errorHandler";

/** 请求选项，语义和 tauriInvoke 一致方便迁移 */
export interface RpcInvokeOptions {
  silent?: boolean;
  context?: string;
  defaultValue?: unknown;
}

/** 静默模式专用选项，强制要求 defaultValue，类型上保证返回 T */
export interface RpcSilentWithOptions<T> {
  silent: true;
  context?: string;
  defaultValue: T;
}

/** 后端未实现该方法，调用方可据此禁用 UI 或提示 */
export class NotImplementedError extends Error {
  readonly method: string;
  readonly transport: "tauri" | "axum";

  constructor(method: string, transport: "tauri" | "axum") {
    super(`[${transport}] 方法 ${method} 尚未实现`);
    this.name = "NotImplementedError";
    this.method = method;
    this.transport = transport;
  }
}

/** Axum 路由描述，把方法名映射到具体的 HTTP 请求 */
interface AxumRoute {
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  /** 从 args 构造路径，不含 /api 前缀；RPC 路由含 /rpc */
  path: (args: Record<string, unknown>) => string;
  /** 从 args 提取请求体，GET/DELETE 不需要 */
  body?: (args: Record<string, unknown>) => unknown;
  /** 是否走 RPC 信封，响应解包方式不同 */
  isRpc?: boolean;
}

// Tauri 命令映射：统一方法名 -> 后端命令名，参数平铺透传
const tauriCommandMap: Record<string, string> = {
  // 实例管理
  "instance.list": "list_instances",
  "instance.get": "get_instance",
  "instance.create": "create_instance",
  "instance.delete": "delete_instance",
  "instance.rename": "rename_instance",
  "instance.updatePath": "update_instance_path",
  // 服务器进程生命周期
  "server.status": "server_status",
  "server.start": "start_server",
  "server.restart": "restart_server",
  "server.stop": "stop_server",
  "server.forceStop": "force_stop_server",
  "server.console.send": "send_server_command",
  // 系统资源
  "system.snapshot": "get_system_snapshot",
  // 定时任务
  "cron.list": "list_cron_tasks",
  "cron.create": "create_cron_task",
  "cron.update": "update_cron_task",
  "cron.delete": "delete_cron_task",
  "cron.setEnabled": "set_cron_task_enabled",
  "cron.run": "run_cron_task",
  // 设置，Axum 仅 overview 其余 Tauri 独有
  "settings.overview": "settings_overview",
  "settings.get": "get_settings",
  "settings.update": "update_settings",
  "settings.updatePartial": "update_settings_partial",
  "settings.reset": "reset_settings",
  "settings.export": "export_settings",
  "settings.import": "import_settings",
  // 服务器核心目录
  "catalog.serverTypes": "catalog_server_types",
  "catalog.versions": "catalog_versions",
  "catalog.details": "catalog_details",
  // 下载任务
  "download.create": "download_create",
  "download.query": "download_query",
  "download.cancel": "download_cancel",
  // 在线隧道
  "tunnel.host": "online_tunnel_host",
  "tunnel.join": "online_tunnel_join",
  "tunnel.stop": "online_tunnel_stop",
  "tunnel.status": "online_tunnel_status",
  // 供给计划
  "provisioning.inspect": "inspect_server",
  "provisioning.parseStartup": "parse_startup_script",
  "provisioning.planExisting": "plan_existing_instance",
  "provisioning.planCopy": "plan_instance_copy",
  "provisioning.planModpack": "plan_modpack_provision",
  "provisioning.importExisting": "import_existing_server",
  // Java 运行时
  "java.detect": "java_detect",
  "java.validate": "java_validate",
  // 应用更新
  "update.check": "check_update",
  "update.download": "update_download",
  "update.pending": "update_pending",
  "update.clearPending": "update_clear_pending",
  "update.install": "update_install",
  // 插件 v2
  "plugin.discover": "plugin_v2_discover",
  "plugin.load": "plugin_v2_load",
  "plugin.enable": "plugin_v2_enable",
  "plugin.disable": "plugin_v2_disable",
  "plugin.unload": "plugin_v2_unload",
  "plugin.list": "plugin_v2_plugins",
  "plugin.grantPersistent": "plugin_v2_grant_persistent",
  "plugin.revokePersistent": "plugin_v2_revoke_persistent",
  "plugin.setTrust": "plugin_v2_set_trust",
  "plugin.grantSession": "plugin_v2_grant_session",
  "plugin.approveSession": "plugin_v2_approve_session",
  "plugin.issueApprovalToken": "plugin_v2_issue_approval_token",
  "plugin.endSession": "plugin_v2_end_session",
  "plugin.audit": "plugin_v2_audit",
  "plugin.invoke": "plugin_v2_invoke",
};

// Axum 路由映射：仅含后端已实现的接口，缺失方法浏览器模式抛 NotImplemented
const axumRouteMap: Record<string, AxumRoute> = {
  "instance.list": { method: "GET", path: () => "/instances" },
  "instance.get": { method: "GET", path: (a) => `/instances/${encodeURIComponent(String(a.id))}` },
  "instance.create": { method: "POST", path: () => "/instances", body: (a) => a.spec },
  "instance.delete": {
    method: "DELETE",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}`,
  },
  "instance.rename": {
    method: "PATCH",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}`,
    body: (a) => ({ name: a.name }),
  },
  "instance.updatePath": {
    method: "PUT",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/path`,
    body: (a) => ({ path: a.path }),
  },
  "server.status": {
    method: "GET",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/status`,
  },
  "server.start": {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/start`,
  },
  "server.restart": {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/restart`,
  },
  "server.stop": {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/stop`,
  },
  "server.forceStop": {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/force-stop`,
    body: (a) => ({ confirmationToken: a.confirmationToken }),
  },
  "server.console.send": {
    method: "POST",
    path: () => "/rpc/server/console/send",
    body: (a) => ({ instanceId: a.id, command: a.command }),
    isRpc: true,
  },
  "system.snapshot": { method: "GET", path: () => "/system" },
  "system.processUsage": {
    method: "GET",
    path: (a) => `/system/process/${encodeURIComponent(String(a.pid))}`,
  },
  "system.directoryUsage": {
    method: "GET",
    path: (a) => `/system/directory/${encodeURIComponent(String(a.path))}`,
  },
  "cron.list": { method: "GET", path: () => "/cron-tasks" },
  "cron.create": { method: "POST", path: () => "/cron-tasks", body: (a) => a.draft },
  "cron.update": {
    method: "PUT",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}`,
    body: (a) => a.draft,
  },
  "cron.delete": {
    method: "DELETE",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}`,
  },
  "cron.setEnabled": {
    method: "PUT",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}/enabled`,
    body: (a) => ({ enabled: a.enabled }),
  },
  "cron.run": {
    method: "POST",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}/run`,
  },
  "settings.overview": { method: "GET", path: () => "/settings" },
  "update.check": { method: "GET", path: () => "/update" },
  "provisioning.importExisting": {
    method: "POST",
    path: () => "/instances/import-existing",
    body: (a) => a.request,
  },
  "provisioning.inspect": {
    method: "POST",
    path: () => "/provisioning/inspect",
    body: (a) => ({ path: a.path }),
  },
};

/** Tauri 原生 invoke，动态导入避免浏览器环境加载失败 */
async function tauriNativeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

/** 从错误响应里尽量提取有意义的消息，兼容 JSON/纯文本/HTML */
async function extractErrorMessage(response: Response): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";

  if (contentType.includes("application/json")) {
    // REST 错误是 {code, message}，RPC 错误是 {requestId, code, message, retryable}
    try {
      const errorBody = await response.json();
      return errorBody?.message || errorBody?.code || response.statusText;
    } catch {
      // JSON 解析失败时回退到文本
      const text = await response.text().catch(() => "");
      return text || response.statusText;
    }
  }

  // 非 JSON 响应，直接读文本保留服务端错误详情
  const text = await response.text().catch(() => "");
  return text || response.statusText;
}

/** 通过 Axum HTTP 调用，处理 REST 和 RPC 两种响应信封 */
async function axumFetch<T>(route: AxumRoute, args: Record<string, unknown>): Promise<T> {
  const url = `${HTTP_API_BASE}/api${route.path(args)}`;
  const init: RequestInit = { method: route.method };

  if (route.body) {
    init.headers = { "Content-Type": "application/json" };
    init.body = JSON.stringify(route.body(args));
  }

  const response = await fetch(url, init);

  if (!response.ok) {
    const message = await extractErrorMessage(response);
    throw new Error(message || `HTTP ${response.status}`);
  }

  // 204 无内容
  if (response.status === 204) {
    return undefined as T;
  }

  const json = await response.json();
  // RPC 路由响应包在 {requestId, data} 里，REST 直接是数据
  return (route.isRpc ? json.data : json) as T;
}

/**
 * 统一 RPC 调用入口
 *
 * 根据运行环境自动选择 Tauri 命令或 Axum HTTP 路由。
 * 方法名不在映射表里时抛 NotImplementedError，表示该后端尚未实现。
 *
 * 重载规则：
 * - 传 silent + defaultValue：失败时返回 defaultValue，类型保证为 T
 * - 传 silent 无 defaultValue：失败时返回 undefined，类型为 T | undefined
 * - 不传 silent 或 silent=false：失败时抛异常，类型为 T
 */
export async function rpcInvoke<T>(
  method: string,
  args: Record<string, unknown>,
  options: RpcSilentWithOptions<T>,
): Promise<T>;
export async function rpcInvoke<T>(
  method: string,
  args?: Record<string, unknown>,
  options?: RpcInvokeOptions,
): Promise<T>;
export async function rpcInvoke<T>(
  method: string,
  args: Record<string, unknown> = {},
  options: RpcInvokeOptions = {},
): Promise<T | undefined> {
  const isHttp = isBrowserEnv();

  try {
    let result: T;

    if (isHttp) {
      const route = axumRouteMap[method];
      if (!route) {
        throw new NotImplementedError(method, "axum");
      }
      result = await axumFetch<T>(route, args);
    } else {
      const command = tauriCommandMap[method];
      if (!command) {
        throw new NotImplementedError(method, "tauri");
      }
      result = await tauriNativeInvoke<T>(command, args);
    }

    if (import.meta.env.DEV) {
      console.debug(`[${isHttp ? "HTTP" : "Tauri"}] ${method} ok`);
    }

    return result;
  } catch (error) {
    // NotImplementedError 直接向上抛，方便调用方识别并禁用功能
    if (error instanceof NotImplementedError) {
      if (!options.silent) {
        throw error;
      }
      return options.defaultValue as T;
    }

    const errorMessage = handleError(error, options.context || method);

    if (import.meta.env.DEV) {
      console.warn(`[${isHttp ? "HTTP" : "Tauri"}] ${method} failed:`, errorMessage);
    }

    if (!options.silent) {
      throw new AppError(errorMessage, ErrorType.SERVER, options.context);
    }

    // 静默模式无 defaultValue 时返回 undefined，类型已通过重载表达
    return options.defaultValue as T | undefined;
  }
}
