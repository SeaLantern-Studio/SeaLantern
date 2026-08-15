// Tauri 命令调用层
// 提供环境检测、NotImplementedError 和原生 invoke 包装

// Tauri 全局类型声明
declare global {
  interface Window {
    __TAURI__?: any;
    // Tauri v2 始终注入此对象，无需 withGlobalTauri 配置
    __TAURI_INTERNALS__?: any;
  }
}

// 环境检测：判断是否在浏览器环境（Docker 模式）
// Tauri v2 默认不注入 window.__TAURI__（需要 withGlobalTauri: true 才有）
// 但 window.__TAURI_INTERNALS__ 在 Tauri v2 中始终存在，用它来可靠判断
export const isBrowserEnv = (): boolean => {
  return typeof window !== "undefined" && !window.__TAURI_INTERNALS__;
};

// HTTP API 基础 URL（Docker 模式下使用）
// 使用相对路径，这样在 Docker 环境下浏览器会自动使用当前页面的域名
export const HTTP_API_BASE = import.meta.env.VITE_API_BASE_URL || "";

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

/**
 * 通过 Tauri invoke 调用命令（原生应用模式）
 */
async function tauriInvokeNative<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  // 动态导入，避免在浏览器环境下加载 @tauri-apps/api/core 导致报错
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

/**
 * Tauri 命令调用选项
 */
export interface InvokeOptions {
  /** 是否静默错误（不抛出异常） */
  silent?: boolean;
  /** 错误上下文描述 */
  context?: string;
  /** 默认返回值（当 silent 为 true 时使用） */
  defaultValue?: unknown;
}

/**
 * Tauri 命令调用，自动检测环境
 *
 * 浏览器模式下这些老命令后端未实现，直接抛 NotImplementedError
 * 后期后端补了路由，把对应命令迁移到 invoke.ts 的 axumRouteMap 即可
 */
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
  options: InvokeOptions = {},
): Promise<T> {
  // 浏览器模式：老命令后端未实现，按 silent 选项决定抛出或返回默认值
  if (isBrowserEnv()) {
    if (!options.silent) {
      throw new NotImplementedError(command, "axum");
    }
    return options.defaultValue as T;
  }

  try {
    const result = await tauriInvokeNative<T>(command, args);

    if (import.meta.env.DEV) {
      console.debug(`[Tauri] Command "${command}" succeeded`);
    }

    return result;
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn(`[Tauri] Command "${command}" failed:`, error);
    }

    if (!options.silent) {
      throw error;
    }

    return options.defaultValue as T;
  }
}

// 批量调用和缓存包装器已移除，没有任何调用方
// 如后期需要可用 Promise.all + tauriInvoke 组合替代
