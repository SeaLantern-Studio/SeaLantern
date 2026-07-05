import { HTTP_API_BASE, parseHttpEnvelope, toStructuredHttpError } from "@api/tauri";

export type BrowserAuthBaseState = "uninitialized" | "setup_pending" | "initialized";

export type BrowserAuthState = BrowserAuthBaseState | "recovery_active";

export interface BrowserAuthContractStatus {
  state: BrowserAuthState;
  base_state: BrowserAuthBaseState;
  recovery_active: boolean;
  setup_required: boolean;
  password_login_enabled: boolean;
  session: {
    ttl_seconds: number;
  };
  next_bridge: {
    enabled: boolean;
    exchange_ttl_seconds: number;
  };
}

export interface BrowserSessionPayload {
  session_token: string;
  token: string;
  expires_at: number;
  purpose: string;
  state?: string;
}

export type AuthProbeResult =
  | { status: "ok" }
  | { status: "unauthorized" }
  | { status: "unreachable" };

function createAuthUnreachableError() {
  return {
    code: "auth.message_unreachable",
    message: "Unable to reach the current instance.",
  };
}

export async function probeBrowserAuthToken(token: string): Promise<AuthProbeResult> {
  try {
    const response = await fetch(`${HTTP_API_BASE}/api/list`, {
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    if (response.ok) {
      return { status: "ok" };
    }

    if (response.status === 401) {
      return { status: "unauthorized" };
    }

    return { status: "unreachable" };
  } catch {
    return { status: "unreachable" };
  }
}

export async function fetchBrowserAuthStatus(): Promise<BrowserAuthContractStatus> {
  let response: Response;
  try {
    response = await fetch(`${HTTP_API_BASE}/api/auth/status`, {
      method: "GET",
    });
  } catch {
    throw createAuthUnreachableError();
  }

  const payload = await parseHttpEnvelope<BrowserAuthContractStatus>(response);
  if (!response.ok || !payload?.success || !payload.data) {
    throw toStructuredHttpError(payload, `HTTP ${response.status}: auth status failed`);
  }

  return payload.data;
}

export async function initializeBrowserAuth(
  setupToken: string,
  password: string,
): Promise<BrowserSessionPayload> {
  let response: Response;
  try {
    response = await fetch(`${HTTP_API_BASE}/api/auth/setup/initialize`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        setup_token: setupToken,
        password,
      }),
    });
  } catch {
    throw createAuthUnreachableError();
  }

  const payload = await parseHttpEnvelope<BrowserSessionPayload>(response);
  if (!response.ok || !payload?.success || !payload.data) {
    throw toStructuredHttpError(payload, `HTTP ${response.status}: auth setup failed`);
  }

  return payload.data;
}

export async function loginBrowserAuth(password: string): Promise<BrowserSessionPayload> {
  let response: Response;
  try {
    response = await fetch(`${HTTP_API_BASE}/api/auth/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        password,
      }),
    });
  } catch {
    throw createAuthUnreachableError();
  }

  const payload = await parseHttpEnvelope<BrowserSessionPayload>(response);
  if (!response.ok || !payload?.success || !payload.data) {
    throw toStructuredHttpError(payload, `HTTP ${response.status}: auth login failed`);
  }

  return payload.data;
}

export async function resetBrowserAuthRecovery(
  recoveryToken: string,
  newPassword: string,
): Promise<BrowserSessionPayload> {
  let response: Response;
  try {
    response = await fetch(`${HTTP_API_BASE}/api/auth/recovery/reset`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        recovery_token: recoveryToken,
        new_password: newPassword,
      }),
    });
  } catch {
    throw createAuthUnreachableError();
  }

  const payload = await parseHttpEnvelope<BrowserSessionPayload>(response);
  if (!response.ok || !payload?.success || !payload.data) {
    throw toStructuredHttpError(payload, `HTTP ${response.status}: auth recovery failed`);
  }

  return payload.data;
}
