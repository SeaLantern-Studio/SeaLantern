import type { RouteLocationNormalized } from "vue-router";

export const AUTH_ROUTE_NAME = "auth";
const DEFAULT_REDIRECT_PATH = "/";

export function sanitizeRedirectPath(raw: unknown): string {
  if (typeof raw !== "string") {
    return DEFAULT_REDIRECT_PATH;
  }

  const trimmed = raw.trim();
  if (!trimmed.startsWith("/") || trimmed.startsWith("//")) {
    return DEFAULT_REDIRECT_PATH;
  }

  try {
    const normalized = new URL(trimmed, window.location.origin);
    if (normalized.origin !== window.location.origin) {
      return DEFAULT_REDIRECT_PATH;
    }

    return `${normalized.pathname}${normalized.search}${normalized.hash}` || DEFAULT_REDIRECT_PATH;
  } catch {
    return DEFAULT_REDIRECT_PATH;
  }
}

export function buildRedirectQuery(to: RouteLocationNormalized): string {
  return to.fullPath === "/auth" ? DEFAULT_REDIRECT_PATH : to.fullPath || DEFAULT_REDIRECT_PATH;
}

export function isPublicRoute(to: RouteLocationNormalized): boolean {
  return to.matched.some((record) => record.meta?.public === true);
}
