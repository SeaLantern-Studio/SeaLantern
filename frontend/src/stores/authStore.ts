import type { Router } from "vue-router";
import { computed, shallowRef } from "vue";
import { defineStore } from "pinia";
import { isBrowserEnv } from "@api/tauri";
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
import {
  fetchBrowserAuthStatus,
  initializeBrowserAuth,
  loginBrowserAuth,
  probeBrowserAuthToken,
  resetBrowserAuthRecovery,
  type BrowserAuthBaseState,
  type BrowserAuthContractStatus,
  type BrowserSessionPayload,
} from "../services/authProbe";
import {
  AUTH_ROUTE_NAME,
  buildRedirectQuery,
  isPublicRoute,
  sanitizeRedirectPath,
} from "@router/authRoute";
import { normalizeAppError } from "@utils/appError";

type BrowserAuthStatus =
  | "idle"
  | "unauthenticated"
  | "probing"
  | "authenticated"
  | "invalid"
  | "unreachable";

type BrowserAuthFlow = Exclude<BrowserAuthBaseState, "uninitialized"> | "recovery_active";

const DEFAULT_UNAUTHORIZED_ERROR = "auth.message_session_expired";

function createDefaultAuthContractStatus(): BrowserAuthContractStatus {
  return {
    state: "setup_pending",
    base_state: "setup_pending",
    recovery_active: false,
    setup_required: true,
    password_login_enabled: false,
    session: {
      ttl_seconds: 0,
    },
    next_bridge: {
      enabled: false,
      exchange_ttl_seconds: 0,
    },
  };
}

function resolveBrowserAuthFlow(status: BrowserAuthContractStatus): BrowserAuthFlow {
  if (status.state === "recovery_active") {
    return "recovery_active";
  }

  if (status.state === "initialized") {
    return "initialized";
  }

  return "setup_pending";
}

export const useAuthStore = defineStore("auth", () => {
  const token = shallowRef<string | null>(null);
  const source = shallowRef<BrowserAuthSource | null>(null);
  const status = shallowRef<BrowserAuthStatus>("idle");
  const lastErrorCode = shallowRef<string | null>(null);
  const hasSavedCredential = shallowRef(false);
  const authContractStatus = shallowRef<BrowserAuthContractStatus>(createDefaultAuthContractStatus());
  const isLoadingAuthStatus = shallowRef(false);

  const isAuthenticated = computed(() => status.value === "authenticated" && !!token.value);
  const isSubmitting = computed(() => status.value === "probing");
  const currentFlow = computed<BrowserAuthFlow>(() =>
    resolveBrowserAuthFlow(authContractStatus.value),
  );

  let routerRef: Router | null = null;
  let unauthorizedCleanup: (() => void) | null = null;
  let hydratePromise: Promise<boolean> | null = null;
  let authStatusPromise: Promise<BrowserAuthContractStatus> | null = null;

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
    }

    refreshSavedCredentialFlag();
  }

  function setAuthError(error: unknown, fallbackCode: string): void {
    const normalized = normalizeAppError(error);
    if (normalized.code === "common.message_unknown_error") {
      lastErrorCode.value = fallbackCode;
      return;
    }

    lastErrorCode.value = normalized.code;
  }

  async function restorePersistedSession(
    candidates: ReturnType<typeof readCandidateTokens>,
    index = 0,
  ): Promise<boolean> {
    const candidate = candidates[index];
    if (!candidate) {
      clearRuntimeState("invalid", lastErrorCode.value ?? "auth.message_session_expired");
      return false;
    }

    const result = await probeBrowserAuthToken(candidate.token);

    if (result.status === "ok") {
      setAuthenticated(candidate.token, candidate.source);
      return true;
    }

    if (result.status === "unauthorized") {
      clearInvalidSource(candidate.source);
      lastErrorCode.value = "auth.message_saved_credential_invalid";
      return restorePersistedSession(candidates, index + 1);
    }

    clearRuntimeState("unreachable", "auth.message_unreachable");
    return false;
  }

  async function refreshAuthStatus(force = false): Promise<BrowserAuthContractStatus> {
    if (!isBrowserEnv()) {
      return authContractStatus.value;
    }

    if (authStatusPromise && !force) {
      return authStatusPromise;
    }

    isLoadingAuthStatus.value = true;
    authStatusPromise = fetchBrowserAuthStatus()
      .then((nextStatus) => {
        authContractStatus.value = nextStatus;
        return nextStatus;
      })
      .catch((error) => {
        setAuthError(error, "auth.message_unreachable");
        throw error;
      })
      .finally(() => {
        isLoadingAuthStatus.value = false;
        authStatusPromise = null;
      });

    return authStatusPromise;
  }

  async function acceptSession(
    session: Pick<BrowserSessionPayload, "session_token" | "token">,
    rememberBrowser: boolean,
  ): Promise<boolean> {
    const nextToken = session.session_token?.trim() || session.token?.trim();
    if (!nextToken) {
      clearRuntimeState("invalid", "auth.message_session_invalid");
      return false;
    }

    const persistedSource = persistToken(nextToken, rememberBrowser);
    setAuthenticated(nextToken, persistedSource);

    try {
      await refreshAuthStatus(true);
    } catch {
      // keep the accepted session even if status refresh is temporarily unavailable
    }

    return true;
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

      try {
        await refreshAuthStatus();
      } catch {
        clearRuntimeState("unreachable", "auth.message_unreachable");
        return false;
      }

      const candidates = readCandidateTokens();
      if (candidates.length === 0) {
        clearRuntimeState("unauthenticated");
        return false;
      }

      return restorePersistedSession(candidates);
    })().finally(() => {
      hydratePromise = null;
    });

    return hydratePromise;
  }

  async function initializePassword(setupToken: string, password: string, rememberBrowser: boolean): Promise<boolean> {
    if (!setupToken.trim()) {
      clearRuntimeState("invalid", "auth.message_setup_token_required");
      return false;
    }

    if (!password.trim()) {
      clearRuntimeState("invalid", "auth.message_password_required");
      return false;
    }

    status.value = "probing";
    lastErrorCode.value = null;

    try {
      const session = await initializeBrowserAuth(setupToken, password);
      await acceptSession(session, rememberBrowser);
      return true;
    } catch (error) {
      setAuthError(error, "auth.message_setup_failed");
      clearRuntimeState(lastErrorCode.value === "auth.message_unreachable" ? "unreachable" : "invalid", lastErrorCode.value);
      return false;
    }
  }

  async function loginWithPassword(password: string, rememberBrowser: boolean): Promise<boolean> {
    if (!password.trim()) {
      clearRuntimeState("invalid", "auth.message_password_required");
      return false;
    }

    status.value = "probing";
    lastErrorCode.value = null;

    try {
      const session = await loginBrowserAuth(password);
      await acceptSession(session, rememberBrowser);
      return true;
    } catch (error) {
      setAuthError(error, "auth.message_password_invalid");
      clearRuntimeState(lastErrorCode.value === "auth.message_unreachable" ? "unreachable" : "invalid", lastErrorCode.value);
      return false;
    }
  }

  async function resetRecovery(
    recoveryToken: string,
    newPassword: string,
    rememberBrowser: boolean,
  ): Promise<boolean> {
    if (!recoveryToken.trim()) {
      clearRuntimeState("invalid", "auth.message_recovery_token_required");
      return false;
    }

    if (!newPassword.trim()) {
      clearRuntimeState("invalid", "auth.message_password_required");
      return false;
    }

    status.value = "probing";
    lastErrorCode.value = null;

    try {
      const session = await resetBrowserAuthRecovery(recoveryToken, newPassword);
      await acceptSession(session, rememberBrowser);
      return true;
    } catch (error) {
      setAuthError(error, "auth.message_recovery_failed");
      clearRuntimeState(lastErrorCode.value === "auth.message_unreachable" ? "unreachable" : "invalid", lastErrorCode.value);
      return false;
    }
  }

  function logout(): void {
    clearPersistedTokens();
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
    hasSavedCredential,
    authContractStatus,
    isAuthenticated,
    isSubmitting,
    isLoadingAuthStatus,
    currentFlow,
    refreshAuthStatus,
    hydrate,
    initializePassword,
    loginWithPassword,
    resetRecovery,
    acceptSession,
    logout,
    clearSavedTokens,
    markUnauthorized,
    attachRouter,
    notifyBrowserUnauthorized,
  };
});
