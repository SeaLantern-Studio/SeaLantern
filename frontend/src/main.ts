import { isBrowserEnv } from "@api/tauri";
import type { UiShellId } from "@api/settings";
import {
  reportDesktopShellRuntime,
  resolveDesktopShellSelection,
  startDesktopHeartbeat,
} from "./launcher/desktopShell";
import { NEXT_SHELL_ENTRY_PATHS } from "../next-src/router/pageMeta";

function normalizePathname(pathname: string): string {
  if (!pathname) {
    return "/";
  }

  if (pathname.length > 1 && pathname.endsWith("/")) {
    return pathname.slice(0, -1);
  }

  return pathname;
}

function shouldMountBrowserNextShell(): boolean {
  return NEXT_SHELL_ENTRY_PATHS.includes(normalizePathname(window.location.pathname));
}

async function mountShell(shellId: UiShellId): Promise<void> {
  if (shellId === "next") {
    const { mountNextApp } = await import("../next-src/main");
    await mountNextApp();
    return;
  }

  const { mountClassicApp } = await import("./classic-entry");
  await mountClassicApp();
}

async function mountSelectedShell(): Promise<void> {
  if (isBrowserEnv()) {
    await mountShell(shouldMountBrowserNextShell() ? "next" : "classic");
    return;
  }

  const { effectiveShell } = await resolveDesktopShellSelection();

  startDesktopHeartbeat();
  void reportDesktopShellRuntime(effectiveShell);

  await mountShell(effectiveShell);
}

void mountSelectedShell();
