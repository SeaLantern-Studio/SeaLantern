import { isBrowserEnv } from "@api/tauri";
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

function shouldMountNextShell(): boolean {
  if (!isBrowserEnv()) {
    return false;
  }

  return NEXT_SHELL_ENTRY_PATHS.includes(normalizePathname(window.location.pathname));
}

async function mountSelectedShell(): Promise<void> {
  if (shouldMountNextShell()) {
    const { mountNextApp } = await import("../next-src/main");
    await mountNextApp();
    return;
  }

  const { mountClassicApp } = await import("./classic-entry");
  await mountClassicApp();
}

void mountSelectedShell();
