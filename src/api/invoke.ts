/**
 * 统一后端调用入口
 *
 * 按运行环境分发到 Tauri 命令或 Axum HTTP 路由。
 * 方法名直接用 Tauri 命令名（snake_case），Tauri 模式零映射透传。
 * Axum 模式通过 axumRouteMap 映射到 REST 路由，缺失的路由抛 NotImplementedError。
 */

import { isBrowserEnv, HTTP_API_BASE, NotImplementedError, type InvokeOptions } from "@api/tauri";

/** 静默模式专用选项，强制要求 defaultValue，类型上保证返回 T */
export interface SilentWithOptions<T> {
  silent: true;
  context?: string;
  defaultValue: T;
}

/** Axum 路由描述，把 Tauri 命令名映射到具体的 HTTP 请求 */
interface AxumRoute {
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  /** 从 args 构造路径，不含 /api 前缀 */
  path: (args: Record<string, unknown>) => string;
  /** 从 args 提取请求体，GET/DELETE 不需要 */
  body?: (args: Record<string, unknown>) => unknown;
}

// Axum 路由映射：key 直接用 Tauri 命令名，Tauri 模式零映射透传
// 仅含后端已实现的接口，缺失方法浏览器模式抛 NotImplemented
const axumRouteMap: Record<string, AxumRoute> = {
  list_instances: { method: "GET", path: () => "/instances" },
  get_instance: { method: "GET", path: (a) => `/instances/${encodeURIComponent(String(a.id))}` },
  create_instance: { method: "POST", path: () => "/instances", body: (a) => a.spec },
  delete_instance: {
    method: "DELETE",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}`,
  },
  rename_instance: {
    method: "PATCH",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}`,
    body: (a) => ({ name: a.name }),
  },
  update_instance_path: {
    method: "PUT",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/path`,
    body: (a) => ({ path: a.path }),
  },
  server_status: {
    method: "GET",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/status`,
  },
  start_server: {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/start`,
  },
  restart_server: {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/restart`,
  },
  stop_server: {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/stop`,
  },
  force_stop_server: {
    method: "POST",
    // 后端只收 Path(id)，confirmationToken 仅 Tauri 模式透传
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/force-stop`,
  },
  send_server_command: {
    method: "POST",
    path: (a) => `/instances/${encodeURIComponent(String(a.id))}/command`,
    body: (a) => ({ command: a.command }),
  },
  get_system_snapshot: { method: "GET", path: () => "/system" },
  get_default_run_path: { method: "GET", path: () => "/system/default-run-path" },
  get_server_resource_usage: {
    method: "GET",
    // 参数名与 Tauri 命令契约保持一致（instance_id）
    path: (a) => `/system/servers/${encodeURIComponent(String(a.instance_id))}/usage`,
  },
  list_cron_tasks: { method: "GET", path: () => "/cron-tasks" },
  create_cron_task: { method: "POST", path: () => "/cron-tasks", body: (a) => a.draft },
  update_cron_task: {
    method: "PUT",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}`,
    body: (a) => a.draft,
  },
  delete_cron_task: {
    method: "DELETE",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}`,
  },
  set_cron_task_enabled: {
    method: "PUT",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}/enabled`,
    body: (a) => ({ enabled: a.enabled }),
  },
  run_cron_task: {
    method: "POST",
    path: (a) => `/cron-tasks/${encodeURIComponent(String(a.id))}/run`,
  },
  settings_overview: { method: "GET", path: () => "/settings" },
  check_update: { method: "GET", path: () => "/update" },
  download_create: {
    method: "POST",
    path: () => "/downloads",
    // 前端按 Tauri 命令形状传 { request: {...} }，HTTP 侧解包 request 作为请求体
    body: (a) => a.request,
  },
  download_query: {
    method: "GET",
    path: (a) => `/downloads/${encodeURIComponent(String(a.id))}`,
  },
  download_cancel: {
    method: "DELETE",
    path: (a) => `/downloads/${encodeURIComponent(String(a.id))}`,
  },
  // 服务器检测
  inspect_server: {
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
    // REST 错误是 {code, message}
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

/** 通过 Axum HTTP 调用，响应直接是数据 */
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

  return (await response.json()) as T;
}

/**
 * 统一后端调用入口
 *
 * 根据运行环境自动选择 Tauri 命令或 Axum HTTP 路由。
 * Axum 模式下路由表未注册的命令抛 NotImplementedError，表示该后端尚未实现。
 *
 * 重载规则：
 * - 传 silent + defaultValue：失败时返回 defaultValue，类型保证为 T
 * - 传 silent 无 defaultValue：失败时返回 undefined，类型为 T | undefined
 * - 不传 silent 或 silent=false：失败时抛异常，类型为 T
 */
export async function invoke<T>(
  method: string,
  args: Record<string, unknown>,
  options: SilentWithOptions<T>,
): Promise<T>;
export async function invoke<T>(
  method: string,
  args?: Record<string, unknown>,
  options?: InvokeOptions,
): Promise<T>;
export async function invoke<T>(
  method: string,
  args: Record<string, unknown> = {},
  options: InvokeOptions = {},
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
      // Tauri 模式：method 直接就是命令名，零映射透传
      result = await tauriNativeInvoke<T>(method, args);
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

    if (import.meta.env.DEV) {
      console.warn(`[${isHttp ? "HTTP" : "Tauri"}] ${method} failed:`, error);
    }

    if (!options.silent) {
      // 直接抛原始错误，保留堆栈信息，不做无意义的包装
      throw error;
    }

    // 静默模式无 defaultValue 时返回 undefined，类型已通过重载表达
    return options.defaultValue as T | undefined;
  }
}
