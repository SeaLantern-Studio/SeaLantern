/**
 * 统一错误处理工具
 */

import { MESSAGES } from "@utils/constants";

/**
 * 错误类型
 */
export enum ErrorType {
  /** 网络错误 */
  NETWORK = "NETWORK",
  /** 验证错误 */
  VALIDATION = "VALIDATION",
  /** 服务器错误 */
  SERVER = "SERVER",
  /** 未知错误 */
  UNKNOWN = "UNKNOWN",
}

/**
 * 应用错误类
 */
export class AppError extends Error {
  constructor(
    message: string,
    public type: ErrorType = ErrorType.UNKNOWN,
    public context?: string,
  ) {
    super(message);
    this.name = "AppError";
  }
}

/**
 * 格式化错误消息
 */
export function formatError(error: unknown): string {
  if (error instanceof AppError) {
    return error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return MESSAGES.ERROR.UNKNOWN_ERROR;
}

/**
 * 处理错误并返回用户友好的消息
 */
export function handleError(error: unknown, context?: string): string {
  const message = formatError(error);

  // 开发环境打印详细错误
  if (import.meta.env.DEV) {
    console.error(`[Error${context ? ` in ${context}` : ""}]:`, error);
  }

  return message;
}

/**
 * 验证玩家名称
 */
export function validatePlayerName(name: string): { valid: boolean; error?: string } {
  if (!name || name.trim().length === 0) {
    return { valid: false, error: MESSAGES.HINT.ENTER_PLAYER_NAME };
  }

  if (name.length < 3 || name.length > 16) {
    return { valid: false, error: "玩家名长度必须在3-16个字符之间" };
  }

  if (!/^[a-zA-Z0-9_]+$/.test(name)) {
    return { valid: false, error: "玩家名只能包含字母、数字和下划线" };
  }

  return { valid: true };
}

/**
 * 安全的JSON解析
 */
export function safeJsonParse<T>(json: string, fallback: T): T {
  try {
    return JSON.parse(json) as T;
  } catch {
    return fallback;
  }
}

/**
 * 重试函数
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  delay: number = 1000,
): Promise<T> {
  let lastError: unknown;

  // eslint-disable-next-line no-await-in-loop
  for (let i = 0; i < maxRetries; i++) {
    try {
      // eslint-disable-next-line no-await-in-loop
      return await fn();
    } catch (error) {
      lastError = error;
      if (i < maxRetries - 1) {
        // eslint-disable-next-line no-await-in-loop
        await new Promise((resolve) => setTimeout(resolve, delay));
      }
    }
  }

  throw lastError;
}
