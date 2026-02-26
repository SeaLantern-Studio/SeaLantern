import { handleError, AppError, ErrorType } from "@utils/errorHandler";

// Tauri 全局类型声明
declare global {
  interface Window {
    __TAURI__?: any;
  }
}

// 环境检测：判断是否在浏览器环境（Docker 模式）
const isBrowserEnv = (): boolean => {
  return typeof window !== "undefined" && !window.__TAURI__;
};

// HTTP API 基础 URL（Docker 模式下使用）
export const HTTP_API_BASE = import.meta.env.VITE_API_BASE_URL || "http://localhost:3000";

/**
 * 通过 HTTP API 调用命令（Docker/浏览器模式）
 */
async function httpInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const url = `${HTTP_API_BASE}/api/${command}`;

  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ params: args || {} }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errorText}`);
  }

  const result = await response.json();

  if (!result.success) {
    throw new Error(result.error || "Unknown error");
  }

  return result.data as T;
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
 * 增强的 Tauri 命令调用函数
 * 提供统一的错误处理和日志记录
 * 自动检测环境，在浏览器模式下使用 HTTP API，在 Tauri 模式下使用 invoke
 */
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
  options: InvokeOptions = {},
): Promise<T> {
  const isHttp = isBrowserEnv();

  try {
    // 根据环境选择调用方式
    const result = isHttp
      ? await httpInvoke<T>(command, args)
      : await tauriInvokeNative<T>(command, args);

    if (import.meta.env.DEV) {
      console.debug(`[${isHttp ? "HTTP" : "Tauri"}] Command "${command}" succeeded`);
    }

    return result;
  } catch (error) {
    const errorMessage = handleError(error, options.context || command);

    if (import.meta.env.DEV) {
      console.warn(`[${isHttp ? "HTTP" : "Tauri"}] Command "${command}" failed:`, errorMessage);
    }

    if (!options.silent) {
      throw new AppError(errorMessage, ErrorType.SERVER, options.context);
    }

    return options.defaultValue as T;
  }
}

/**
 * 批量 Tauri 命令调用
 */
export async function tauriInvokeAll(
  commands: Array<{
    command: string;
    args?: Record<string, unknown>;
    key?: string;
  }>,
  options: InvokeOptions = {},
): Promise<Record<string, unknown> | unknown[]> {
  const promises = commands.map(({ command, args, key }) =>
    tauriInvoke<unknown>(command, args, options).then((result) => ({ key, result })),
  );

  const results = await Promise.all(promises);

  if (commands.every((c) => c.key !== undefined)) {
    return results.reduce(
      (acc, { key, result }) => {
        acc[key as string] = result;
        return acc;
      },
      {} as Record<string, unknown>,
    );
  }

  return results.map((r) => r.result);
}

/**
 * 创建带缓存的 Tauri 调用包装器
 */
export function createCachedInvoke<T>(command: string, cacheTime: number = 5000) {
  let cache: { data: T; timestamp: number } | null = null;

  return async (args?: Record<string, unknown>, options?: InvokeOptions): Promise<T> => {
    const now = Date.now();

    if (cache && now - cache.timestamp < cacheTime) {
      return cache.data;
    }

    const data = await tauriInvoke<T>(command, args, options);
    cache = { data, timestamp: now };
    return data;
  };
}
