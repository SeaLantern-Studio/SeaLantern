import { tauriInvoke } from "./tauri";
import type { JavaInfo } from "./java";

export interface AppSettings {
  close_servers_on_exit: boolean;
  auto_accept_eula: boolean;
  default_max_memory: number;
  default_min_memory: number;
  default_port: number;
  default_java_path: string;
  default_jvm_args: string;
  console_font_size: number;
  max_log_lines: number;
  cached_java_list: JavaInfo[];
  background_image: string;
  background_opacity: number;
  background_blur: number;
  background_brightness: number;
  background_size: string;
  acrylic_enabled: boolean;
  theme: string;
  font_size: number;
  font_family: string;
  color: string;
  language: string;
  close_action: string; // ask, minimize, close
  bg_color?: string;
  bg_secondary_color?: string;
  bg_tertiary_color?: string;
  primary_color?: string;
  secondary_color?: string;
  bg_dark?: string;
  bg_secondary_dark?: string;
  bg_tertiary_dark?: string;
  primary_dark?: string;
  secondary_dark?: string;
  bg_acrylic?: string;
  bg_secondary_acrylic?: string;
  bg_tertiary_acrylic?: string;
  primary_acrylic?: string;
  secondary_acrylic?: string;
  bg_dark_acrylic?: string;
  bg_secondary_dark_acrylic?: string;
  bg_tertiary_dark_acrylic?: string;
  primary_dark_acrylic?: string;
  secondary_dark_acrylic?: string;
  text_primary_color?: string;
  text_secondary_color?: string;
  border_color?: string;
  text_primary_dark?: string;
  text_secondary_dark?: string;
  border_dark?: string;
  text_primary_acrylic?: string;
  text_secondary_acrylic?: string;
  border_acrylic?: string;
  text_primary_dark_acrylic?: string;
  text_secondary_dark_acrylic?: string;
  border_dark_acrylic?: string;
}

export const settingsApi = {
  async get(): Promise<AppSettings> {
    return tauriInvoke("get_settings");
  },
  async save(settings: AppSettings): Promise<void> {
    return tauriInvoke("save_settings", { settings });
  },
  async reset(): Promise<AppSettings> {
    return tauriInvoke("reset_settings");
  },
  async exportJson(): Promise<string> {
    return tauriInvoke("export_settings");
  },
  async importJson(json: string): Promise<AppSettings> {
    return tauriInvoke("import_settings", { json });
  },
};

export async function checkAcrylicSupport(): Promise<boolean> {
  return tauriInvoke<boolean>("check_acrylic_support");
}

export async function applyAcrylic(enabled: boolean, darkMode: boolean): Promise<void> {
  return tauriInvoke<void>("apply_acrylic", { enabled, darkMode });
}

export async function getSystemFonts(): Promise<string[]> {
  return tauriInvoke<string[]>("get_system_fonts");
}
