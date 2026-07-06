import { startDesktopHeartbeat } from "./launcher/desktopShell";

async function mountNextShell(): Promise<void> {
  const { mountNextApp } = await import("./main.next");
  await mountNextApp();
}

async function mountSelectedShell(): Promise<void> {
  startDesktopHeartbeat();

  await mountNextShell();
}

void mountSelectedShell();
