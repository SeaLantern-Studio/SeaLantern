import { i18n } from "@language";

export interface BackendErrorPayload {
  code?: string;
  message?: string;
  args?: Record<string, unknown>;
  error_kind?: string;
}

export interface NormalizedAppError {
  code: string;
  message: string;
  args?: Record<string, unknown>;
  cause?: unknown;
}

const DEFAULT_ERROR_CODE = "common.message_unknown_error";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function normalizeAppError(error: unknown): NormalizedAppError {
  if (isObject(error)) {
    const payload = error as BackendErrorPayload;
    const code = payload.code || payload.error_kind || DEFAULT_ERROR_CODE;
    const message = payload.message || (typeof error === "string" ? error : "");
    return {
      code,
      message: message || resolveErrorMessage(code, payload.args),
      args: payload.args,
      cause: error,
    };
  }

  if (error instanceof Error) {
    return {
      code: DEFAULT_ERROR_CODE,
      message: error.message || resolveErrorMessage(DEFAULT_ERROR_CODE),
      cause: error,
    };
  }

  if (typeof error === "string") {
    return {
      code: DEFAULT_ERROR_CODE,
      message: error,
      cause: error,
    };
  }

  return {
    code: DEFAULT_ERROR_CODE,
    message: resolveErrorMessage(DEFAULT_ERROR_CODE),
    cause: error,
  };
}

export function resolveErrorMessage(code: string, args?: Record<string, unknown>): string {
  return i18n.te(code) ? i18n.t(code, args) : code;
}
