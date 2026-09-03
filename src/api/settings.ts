import { isBrowserEnv, tauriInvoke } from "@api/tauri";
import { invoke } from "@api/invoke";
import type { JavaInfo } from "@api/java";

export type SettingsGroup =
  | "General"
  | "Network"
  | "ServerDefaults"
  | "Console"
  | "Appearance"
  | "Window"
  | "Developer";

export type ProxySettings =
  | { mode: "adaptive" }
  | { mode: "preserve" }
  | { mode: "manual"; proxy_url: string }
  | { mode: "disabled" };

export interface AppSettings {
  close_servers_on_exit: boolean;
  close_servers_on_update: boolean;
  auto_accept_eula: boolean;
  auto_lightweight_minutes: number | null;
  default_max_memory: number;
  default_min_memory: number;
  default_port: number;
  default_java_path: string;
  default_jvm_args: string;
  console_font_size: number;
  console_font_family: string;
  console_letter_spacing: number;
  max_log_lines: number;
  cached_java_list: JavaInfo[];
  background_image: string;
  background_opacity: number;
  background_blur: number;
  background_brightness: number;
  background_size: string;
  window_width?: number;
  window_height?: number;
  window_x?: number | null;
  window_y?: number | null;
  window_maximized?: boolean;
  acrylic_enabled: boolean;
  theme: string;
  font_size: number;
  font_family: string;
  color: string;
  language: string;
  locales_base_url?: string;
  developer_mode: boolean;
  close_action: string;
  proxy: ProxySettings;
  last_run_path: string;
  minimal_mode: boolean;
  agreed_to_terms: boolean;
}

export interface PartialSettings {
  close_servers_on_exit?: boolean;
  close_servers_on_update?: boolean;
  auto_accept_eula?: boolean;
  auto_lightweight_minutes?: number | null;
  default_max_memory?: number;
  default_min_memory?: number;
  default_port?: number;
  default_java_path?: string;
  default_jvm_args?: string;
  console_font_size?: number;
  console_font_family?: string;
  console_letter_spacing?: number;
  max_log_lines?: number;
  cached_java_list?: JavaInfo[];
  background_image?: string;
  background_opacity?: number;
  background_blur?: number;
  background_brightness?: number;
  background_size?: string;
  window_width?: number;
  window_height?: number;
  window_x?: number | null;
  window_y?: number | null;
  window_maximized?: boolean;
  acrylic_enabled?: boolean;
  theme?: string;
  font_size?: number;
  font_family?: string;
  color?: string;
  language?: string;
  developer_mode?: boolean;
  close_action?: string;
  proxy?: ProxySettings;
  last_run_path?: string;
  minimal_mode?: boolean;
  agreed_to_terms?: boolean;
}

export interface UpdateSettingsResult {
  settings: AppSettings;
  changed_groups: SettingsGroup[];
}

export const settingsApi = {
  async get(): Promise<AppSettings> {
    return invoke("get_settings");
  },
  async save(settings: AppSettings): Promise<void> {
    // 后端 update_settings 返回 UpdateResult，这里只关心副作用，丢弃结果
    await invoke("update_settings", { settings });
  },
  async saveWithDiff(settings: AppSettings): Promise<UpdateSettingsResult> {
    return invoke("update_settings", { settings });
  },
  async updatePartial(partial: PartialSettings): Promise<UpdateSettingsResult> {
    return invoke("update_settings_partial", { partial });
  },
  async reset(): Promise<AppSettings> {
    return invoke("reset_settings");
  },
  async exportJson(): Promise<string> {
    return invoke("export_settings");
  },
  async importJson(json: string): Promise<AppSettings> {
    return invoke("import_settings", { json });
  },
  // 原生亚克力由系统合成器模糊桌面,CSS backdrop-filter 够不到窗外内容
  async applyAcrylic(enabled: boolean): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("apply_acrylic", { enabled }, { silent: true });
  },
};

export async function getSystemFonts(): Promise<string[]> {
  return tauriInvoke<string[]>("get_system_fonts");
}
