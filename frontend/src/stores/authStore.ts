import type { Router } from "vue-router";
import { computed, shallowRef } from "vue";
import { defineStore } from "pinia";
import { readBrowserEnvAuthToken, isBrowserEnv } from "@api/tauri";
import {
  notifyBrowserUnauthorized,
  onBrowserUnauthorized,
  setBrowserRuntimeToken,
} from "@stores/authRuntime";
import {
  clearPersistedTokens,
  clearRememberedToken,
  clearSessionToken,
  hasPersistedToken,
  persistToken,
  readCandidateTokens,
  type BrowserAuthSource,
} from "../services/authStorage";
import { probeBrowserAuthToken } from "../services/authProbe";
import {
  AUTH_ROUTE_NAME,
  buildRedirectQuery,
  isPublicRoute,
  sanitizeRedirectPath,
} from "@router/authRoute";

type BrowserAuthStatus =
  | "idle"
  | "unauthenticated"
  | "probing"
  | "authenticated"
  | "invalid"
  | "unreachable";

const DEFAULT_UNAUTHORIZED_ERROR = "auth.message_session_expired";

export const useAuthStore = defineStore("auth", () => {
  const token = shallowRef<string | null>(null);
  const source = shallowRef<BrowserAuthSource | null>(null);
  const status = shallowRef<BrowserAuthStatus>("idle");
  const lastErrorCode = shallowRef<string | null>(null);
  const envRejected = shallowRef(false);
  const hasSavedCredential = shallowRef(false);

  const isAuthenticated = computed(() => status.value === "authenticated" && !!token.value);
  const isSubmitting = computed(() => status.value === "probing");

  let routerRef: Router | null = null;
  let unauthorizedCleanup: (() => void) | null = null;
  let hydratePromise: Promise<boolean> | null = null;

  function refreshSavedCredentialFlag(): void {
    hasSavedCredential.value = hasPersistedToken();
  }

  function setAuthenticated(nextToken: string, nextSource: BrowserAuthSource): void {
    token.value = nextToken;
    source.value = nextSource;
    status.value = "authenticated";
    lastErrorCode.value = null;
    setBrowserRuntimeToken(nextToken);
    refreshSavedCredentialFlag();
  }

  function clearRuntimeState(nextStatus: BrowserAuthStatus, errorCode: string | null = null): void {
    token.value = null;
    source.value = null;
    status.value = nextStatus;
    lastErrorCode.value = errorCode;
    setBrowserRuntimeToken(null);
    refreshSavedCredentialFlag();
  }

  function clearInvalidSource(invalidSource: BrowserAuthSource): void {
    if (invalidSource === "session") {
      clearSessionToken();
    } else if (invalidSource === "local") {
      clearRememberedToken();
    } else if (invalidSource === "env") {
      envRejected.value = true;
    }
    refreshSavedCredentialFlag();
  }

  async function hydrate(): Promise<boolean> {
    if (!isBrowserEnv()) {
      return true;
    }

    if (isAuthenticated.value) {
      return true;
    }

    if (hydratePromise) {
      return hydratePromise;
    }

    hydratePromise = (async () => {
      status.value = "probing";
      lastErrorCode.value = null;
      refreshSavedCredentialFlag();

      const candidates = readCandidateTokens({
        envToken: readBrowserEnvAuthToken(),
        includeEnv: !envRejected.value,
      });

      if (candidates.length === 0) {
        clearRuntimeState("unauthenticated");
        return false;
      }

      for (const candidate of candidates) {
        const result = await probeBrowserAuthToken(candidate.token);

        if (result.status === "ok") {
          setAuthenticated(candidate.token, candidate.source);
          return true;
        }

        if (result.status === "unauthorized") {
          clearInvalidSource(candidate.source);

          if (candidate.source === "env") {
            lastErrorCode.value = "auth.message_preset_credential_invalid";
          } else {
            lastErrorCode.value = "auth.message_saved_credential_invalid";
          }

          continue;
        }

        clearRuntimeState("unreachable", "auth.message_unreachable");
        return false;
      }

      clearRuntimeState("invalid", lastErrorCode.value ?? "auth.message_token_invalid");
      return false;
    })().finally(() => {
      hydratePromise = null;
    });

    return hydratePromise;
  }

  async function login(nextToken: string, rememberBrowser: boolean): Promise<boolean> {
    const trimmed = nextToken.trim();
    if (!trimmed) {
      clearRuntimeState("invalid", "auth.message_token_required");
      return false;
    }

    status.value = "probing";
    lastErrorCode.value = null;

    const result = await probeBrowserAuthToken(trimmed);
    if (result.status === "ok") {
      const persistedSource = persistToken(trimmed, rememberBrowser);
      envRejected.value = false;
      setAuthenticated(trimmed, persistedSource);
      return true;
    }

    if (result.status === "unauthorized") {
      clearRuntimeState("invalid", "auth.message_token_invalid");
      return false;
    }

    clearRuntimeState("unreachable", "auth.message_unreachable");
    return false;
  }

  function logout(): void {
    clearPersistedTokens();
    envRejected.value = false;
    clearRuntimeState("unauthenticated");
  }

  function clearSavedTokens(): void {
    clearPersistedTokens();
    refreshSavedCredentialFlag();

    if (!isAuthenticated.value) {
      clearRuntimeState("unauthenticated");
    }
  }

  function resolveAuthRedirect(): string {
    if (!routerRef?.currentRoute.value) {
      return "/";
    }

    const currentRoute = routerRef.currentRoute.value;
    if (currentRoute.name === AUTH_ROUTE_NAME || isPublicRoute(currentRoute)) {
      return "/";
    }

    return sanitizeRedirectPath(buildRedirectQuery(currentRoute));
  }

  function redirectToAuth(): void {
    if (!routerRef) {
      return;
    }

    const redirect = resolveAuthRedirect();
    void routerRef.replace({
      name: AUTH_ROUTE_NAME,
      query: redirect === "/" ? {} : { redirect },
    });
  }

  function markUnauthorized(reason = DEFAULT_UNAUTHORIZED_ERROR): void {
    if (!isBrowserEnv()) {
      return;
    }

    clearPersistedTokens();
    clearRuntimeState("invalid", reason);
    redirectToAuth();
  }

  function attachRouter(router: Router): void {
    routerRef = router;

    unauthorizedCleanup?.();
    unauthorizedCleanup = onBrowserUnauthorized((reason) => {
      markUnauthorized(reason ?? DEFAULT_UNAUTHORIZED_ERROR);
    });
  }

  return {
    token,
    source,
    status,
    lastErrorCode,
    envRejected,
    hasSavedCredential,
    isAuthenticated,
    isSubmitting,
    hydrate,
    login,
    logout,
    clearSavedTokens,
    markUnauthorized,
    attachRouter,
    notifyBrowserUnauthorized,
  };
});
