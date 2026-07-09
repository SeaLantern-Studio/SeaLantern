import {
  ensureBrowserSession,
  HTTP_API_BASE,
  notifyBrowserUnauthorized,
  parseHttpEnvelope,
  readBrowserAuthToken,
  toStructuredHttpError,
} from "@api/tauri";
import type { BrowserTotpContractStatus } from "./authProbe";

export type BrowserTotpMutationPayload = Record<string, unknown>;

export type BrowserTotpSetupBeginRequest = BrowserTotpMutationPayload;
export type BrowserTotpSetupConfirmRequest = BrowserTotpMutationPayload;
export type BrowserTotpDisableRequest = BrowserTotpMutationPayload;

export type BrowserTotpSetupBeginResponse = Record<string, never>;
export type BrowserTotpSetupConfirmResponse = Record<string, never>;
export type BrowserTotpDisableResponse = Record<string, never>;

function createAuthUnreachableError() {
  return {
    code: "auth.message_unreachable",
    message: "Unable to reach the current instance.",
  };
}

async function fetchBrowserTotpEndpoint<T>(path: string, init?: RequestInit): Promise<T> {
  await ensureBrowserSession();

  const token = readBrowserAuthToken();

  let response: Response;
  try {
    response = await fetch(`${HTTP_API_BASE}${path}`, {
      ...init,
      headers: {
        ...init?.headers,
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
    });
  } catch {
    throw createAuthUnreachableError();
  }

  const payload = await parseHttpEnvelope<T>(response);

  if (!response.ok) {
    if (response.status === 401) {
      notifyBrowserUnauthorized("auth.message_session_expired");
    }

    throw toStructuredHttpError(payload, `HTTP ${response.status}: TOTP request failed`);
  }

  if (!payload?.success || payload.data === undefined) {
    throw toStructuredHttpError(payload, "TOTP request failed");
  }

  return payload.data;
}

export async function fetchBrowserTotpStatus(): Promise<BrowserTotpContractStatus> {
  return fetchBrowserTotpEndpoint<BrowserTotpContractStatus>("/api/auth/totp/status", {
    method: "GET",
  });
}

export async function beginBrowserTotpSetup(
  payload: BrowserTotpSetupBeginRequest = {},
): Promise<BrowserTotpSetupBeginResponse> {
  return fetchBrowserTotpEndpoint<BrowserTotpSetupBeginResponse>("/api/auth/totp/setup/begin", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
}

export async function confirmBrowserTotpSetup(
  payload: BrowserTotpSetupConfirmRequest = {},
): Promise<BrowserTotpSetupConfirmResponse> {
  return fetchBrowserTotpEndpoint<BrowserTotpSetupConfirmResponse>("/api/auth/totp/setup/confirm", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
}

export async function disableBrowserTotp(
  payload: BrowserTotpDisableRequest = {},
): Promise<BrowserTotpDisableResponse> {
  return fetchBrowserTotpEndpoint<BrowserTotpDisableResponse>("/api/auth/totp/disable", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
}
