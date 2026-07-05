let runtimeToken: string | null = null;

const unauthorizedListeners = new Set<(reason?: string) => void>();

function normalizeToken(token: string | null | undefined): string | null {
  const trimmed = token?.trim();
  return trimmed ? trimmed : null;
}

export function getBrowserRuntimeToken(): string | null {
  return runtimeToken;
}

export function setBrowserRuntimeToken(token: string | null | undefined): void {
  runtimeToken = normalizeToken(token);
}

export function onBrowserUnauthorized(listener: (reason?: string) => void): () => void {
  unauthorizedListeners.add(listener);
  return () => {
    unauthorizedListeners.delete(listener);
  };
}

export function notifyBrowserUnauthorized(reason?: string): void {
  unauthorizedListeners.forEach((listener) => listener(reason));
}
