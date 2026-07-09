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

export class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public args?: Record<string, unknown>,
    public cause?: unknown,
  ) {
    super(message);
    this.name = "AppError";
  }
}

const DEFAULT_ERROR_CODE = "common.message_unknown_error";

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function tryParseStructuredPayload(error: unknown): BackendErrorPayload | null {
  if (typeof error !== "string") {
    return null;
  }

  try {
    const parsed = JSON.parse(error) as unknown;
    if (!isObject(parsed)) {
      return null;
    }
    const hasStructuredField =
      typeof parsed.code === "string" ||
      typeof parsed.message === "string" ||
      typeof parsed.error_kind === "string";
    return hasStructuredField ? (parsed as BackendErrorPayload) : null;
  } catch {
    return null;
  }
}

export function normalizeAppError(error: unknown): NormalizedAppError {
  const parsedPayload = tryParseStructuredPayload(error);
  if (parsedPayload) {
    return normalizeAppError(parsedPayload);
  }

  if (isObject(error)) {
    const payload = error as BackendErrorPayload;
    const code = payload.code || payload.error_kind || DEFAULT_ERROR_CODE;
    const rawMessage = payload.message || (typeof error === "string" ? error : "");
    const message =
      code !== DEFAULT_ERROR_CODE ? resolveErrorMessage(code, payload.args) : rawMessage;
    return {
      code,
      message: message || rawMessage || resolveErrorMessage(code, payload.args),
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
