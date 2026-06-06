import { isBrowserEnv, tauriInvoke } from "@api/tauri";
import type { JavaInfo } from "@api/java";
import type { CpuPolicyConfig, JvmPresetConfig } from "@type/server";

export type WindowEffect = "off" | "auto" | "blur" | "acrylic" | "mica" | "vibrancy";

export interface TextColorOverrides {
  title: string;
  text: string;
  description: string;
}

export type SettingsGroup =
  | "General"
  | "ServerDefaults"
  | "Console"
  | "Appearance"
  | "Window"
  | "Developer";

export interface AppSettings {
  close_servers_on_exit: boolean;
  close_servers_on_update: boolean;
  auto_accept_eula: boolean;
  default_max_memory: number;
  default_min_memory: number;
  default_port: number;
  default_java_path: string;
  default_jvm_args: string[];
  default_cpu_policy: CpuPolicyConfig;
  default_jvm_preset: JvmPresetConfig;
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
  window_effect: WindowEffect;
  acrylic_enabled: boolean;
  theme: string;
  font_size: number;
  font_family: string;
  memory_display_precision: number;
  color: string;
  text_color_overrides: TextColorOverrides;
  app_display_name: string;
  language: string;
  locales_base_url?: string;
  developer_mode: boolean;
  close_action: string;
  last_run_path: string;
  minimal_mode: boolean;
  agreed_to_terms: boolean;
}

export interface PartialSettings {
  close_servers_on_exit?: boolean;
  close_servers_on_update?: boolean;
  auto_accept_eula?: boolean;
  default_max_memory?: number;
  default_min_memory?: number;
  default_port?: number;
  default_java_path?: string;
  default_jvm_args?: string[];
  default_cpu_policy?: CpuPolicyConfig;
  default_jvm_preset?: JvmPresetConfig;
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
  window_effect?: WindowEffect;
  acrylic_enabled?: boolean;
  theme?: string;
  font_size?: number;
  font_family?: string;
  memory_display_precision?: number;
  color?: string;
  text_color_overrides?: TextColorOverrides;
  app_display_name?: string;
  language?: string;
  developer_mode?: boolean;
  close_action?: string;
  last_run_path?: string;
  minimal_mode?: boolean;
  agreed_to_terms?: boolean;
}

export interface UpdateSettingsResult {
  settings: AppSettings;
  changed_groups: SettingsGroup[];
}

export interface ImportPersonalizationResult {
  settings: AppSettings;
  changed_groups: SettingsGroup[];
  imported_plugins: string[];
  skipped_plugins: string[];
}

export const settingsApi = {
  async get(): Promise<AppSettings> {
    return tauriInvoke("get_settings");
  },
  async save(settings: AppSettings): Promise<void> {
    return tauriInvoke("save_settings", { settings });
  },
  async saveWithDiff(settings: AppSettings): Promise<UpdateSettingsResult> {
    return tauriInvoke("save_settings_with_diff", { settings });
  },
  async updatePartial(partial: PartialSettings): Promise<UpdateSettingsResult> {
    return tauriInvoke("update_settings_partial", { partial });
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
  async exportPersonalizationPackage(path: string): Promise<void> {
    return tauriInvoke("export_personalization_package", { path });
  },
  async importPersonalizationPackage(path: string): Promise<ImportPersonalizationResult> {
    return tauriInvoke("import_personalization_package", { path });
  },
  async getPersonalizationPackageSuggestedName(): Promise<string> {
    return tauriInvoke("get_personalization_package_suggested_name");
  },
  async applyAcrylic(enabled: boolean): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("apply_acrylic", { enabled }, { silent: true });
  },
  async applyWindowEffect(effect: WindowEffect, dark?: boolean): Promise<void> {
    if (isBrowserEnv()) return;
    await tauriInvoke("apply_window_effect", { effect, dark }, { silent: true });
  },
};

export async function getSystemFonts(): Promise<string[]> {
  return tauriInvoke<string[]>("get_system_fonts");
}
