export type BrowserAuthSource = "runtime" | "session" | "local" | "env";

export interface BrowserAuthCandidate {
  token: string;
  source: BrowserAuthSource;
}

const SESSION_TOKEN_KEY = "sealantern.auth.session_token";
const LOCAL_TOKEN_KEY = "sealantern.auth.remembered_token";

function normalizeToken(token: string | null | undefined): string | null {
  const trimmed = token?.trim();
  return trimmed ? trimmed : null;
}

function safeRead(storage: Storage, key: string): string | null {
  try {
    return normalizeToken(storage.getItem(key));
  } catch {
    return null;
  }
}

function safeWrite(storage: Storage, key: string, token: string): void {
  try {
    storage.setItem(key, token);
  } catch {
    // ignore storage write failure
  }
}

function safeRemove(storage: Storage, key: string): void {
  try {
    storage.removeItem(key);
  } catch {
    // ignore storage removal failure
  }
}

export function readSessionToken(): string | null {
  return safeRead(window.sessionStorage, SESSION_TOKEN_KEY);
}

export function readRememberedToken(): string | null {
  return safeRead(window.localStorage, LOCAL_TOKEN_KEY);
}

export function writeSessionToken(token: string): void {
  safeWrite(window.sessionStorage, SESSION_TOKEN_KEY, token);
}

export function writeRememberedToken(token: string): void {
  safeWrite(window.localStorage, LOCAL_TOKEN_KEY, token);
}

export function clearSessionToken(): void {
  safeRemove(window.sessionStorage, SESSION_TOKEN_KEY);
}

export function clearRememberedToken(): void {
  safeRemove(window.localStorage, LOCAL_TOKEN_KEY);
}

export function clearPersistedTokens(): void {
  clearSessionToken();
  clearRememberedToken();
}

export function hasPersistedToken(): boolean {
  return readSessionToken() !== null || readRememberedToken() !== null;
}

export function persistToken(token: string, rememberBrowser: boolean): BrowserAuthSource {
  clearPersistedTokens();

  if (rememberBrowser) {
    writeRememberedToken(token);
    return "local";
  }

  writeSessionToken(token);
  return "session";
}

export function readCandidateTokens(options: {
  envToken?: string | null;
  includeEnv?: boolean;
}): BrowserAuthCandidate[] {
  const result: BrowserAuthCandidate[] = [];
  const seen = new Set<string>();

  const pushCandidate = (token: string | null, source: BrowserAuthSource) => {
    const normalized = normalizeToken(token);
    if (!normalized || seen.has(normalized)) {
      return;
    }

    seen.add(normalized);
    result.push({ token: normalized, source });
  };

  pushCandidate(readSessionToken(), "session");
  pushCandidate(readRememberedToken(), "local");

  if (options.includeEnv !== false) {
    pushCandidate(options.envToken ?? null, "env");
  }

  return result;
}
