import {
  reportDesktopShellRuntime,
  startDesktopHeartbeat,
} from "./launcher/desktopShell";
import { DESKTOP_PRIMARY_SHELL } from "./launcher/desktopShell";

async function mountNextShell(): Promise<void> {
  const { mountNextApp } = await import("../next-src/main");
  await mountNextApp();
}

async function mountSelectedShell(): Promise<void> {
  startDesktopHeartbeat();
  void reportDesktopShellRuntime(DESKTOP_PRIMARY_SHELL);

  await mountNextShell();
}

void mountSelectedShell();
