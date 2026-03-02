import { invoke } from "@tauri-apps/api/core";
import { handleError, AppError, ErrorType } from "@utils/errorHandler";

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
 */
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
  options: InvokeOptions = {},
): Promise<T> {
  try {
    const result = await invoke<T>(command, args);

    if (import.meta.env.DEV) {
      console.debug(`[Tauri] Command "${command}" succeeded`);
    }

    return result;
  } catch (error) {
    const errorMessage = handleError(error, options.context || command);

    if (!options.silent) {
      throw new AppError(errorMessage, ErrorType.SERVER, options.context);
    }

    if (import.meta.env.DEV) {
      console.warn(`[Tauri] Command "${command}" failed (silent):`, errorMessage);
    }

    return options.defaultValue as T;
  }
}

/**
 * 批量 Tauri 命令调用
 * 并行执行多个命令，返回结果数组
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
 * 用于避免重复调用相同的命令
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
