import { HTTP_API_BASE } from "@api/tauri";

export type AuthProbeResult =
  | { status: "ok" }
  | { status: "unauthorized" }
  | { status: "unreachable" };

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
