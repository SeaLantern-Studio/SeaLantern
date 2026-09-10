import { isBrowserEnv, tauriInvoke } from "@api/tauri";

export type WindowMaterial = "solid" | "acrylic" | "vibrancy" | "liquid_glass";
export type WindowTheme = "auto" | "light" | "dark";

/** Window lifecycle and native-material capabilities available only in the desktop host. */
export const desktopApi = {
  async markFrontendReady(): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("frontend_ready");
  },

  async setCloseRequestListenerReady(ready: boolean): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("set_close_request_listener_ready", { ready });
  },

  async setWindowMaterial(material: WindowMaterial, theme: WindowTheme): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("set_window_material", { material, theme });
  },

  async supportsLiquidGlass(): Promise<boolean> {
    if (isBrowserEnv()) return false;
    return tauriInvoke<boolean>("supports_liquid_glass");
  },

  async hideMainWindow(): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("hide_main_window");
  },

  async restoreMainWindow(): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("restore_main_window");
  },

  async toggleLightWeight(): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("toggle_light_weight");
  },
};
