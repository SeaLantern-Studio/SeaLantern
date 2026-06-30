import { isBrowserEnv } from "@api/tauri";

async function mountSelectedShell(): Promise<void> {
  if (isBrowserEnv()) {
    const { mountNextApp } = await import("../next-src/main");
    await mountNextApp();
    return;
  }

  const { mountClassicApp } = await import("./classic-entry");
  await mountClassicApp();
}

void mountSelectedShell();
