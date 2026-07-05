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
  | "Developer"
  | "Online";

export type OneBotTargetType = "group" | "private";

export interface OneBotTarget {
  type: OneBotTargetType;
  id: string;
}

export interface OneBot11Settings {
  enabled: boolean;
  api_base_url: string;
  access_token: string;
  event_classes: string[];
  structured_event_kinds: string[];
  server_ids: string[];
  targets: OneBotTarget[];
  message_template: string;
}

export interface NextHomeLayoutItem {
  instanceId: string;
  kind: string;
  x: number;
  y: number;
  width: number;
  height: number;
  colStart: number;
  rowStart: number;
  colSpan: number;
  rowSpan: number;
  zIndex: number;
}

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
  locale_layer_order: string[];
  locales_base_url?: string;
  developer_mode: boolean;
  enable_desktop_web_ui: boolean;
  close_action: string;
  last_run_path: string;
  minimal_mode: boolean;
  next_home_layout: NextHomeLayoutItem[];
  agreed_to_terms: boolean;
  onebot_11: OneBot11Settings;
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
  locale_layer_order?: string[];
  developer_mode?: boolean;
  enable_desktop_web_ui?: boolean;
  close_action?: string;
  last_run_path?: string;
  minimal_mode?: boolean;
  next_home_layout?: NextHomeLayoutItem[];
  agreed_to_terms?: boolean;
  onebot_11?: OneBot11Settings;
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

export interface DataDirStatus {
  current_data_dir: string;
  default_data_dir: string;
  locator_path: string;
  resolution_source: string;
  locator_exists: boolean;
  needs_initial_selection: boolean;
  recommended_data_dir: string;
}

export interface DataDirChangeResult {
  status: DataDirStatus;
  migrated_entries: string[];
}

export interface PluginDirStatus {
  current_plugin_dir: string;
  default_plugin_dir: string;
  locator_path: string;
  resolution_source: string;
  locator_exists: boolean;
  recommended_plugin_dir: string;
}

export interface PluginDirChangeResult {
  status: PluginDirStatus;
  migrated_entries: string[];
}

interface WebOneBot11SettingsDto {
  enabled: boolean;
  api_base_url: string;
  event_classes: string[];
  structured_event_kinds: string[];
  server_ids: string[];
  targets: OneBotTarget[];
  message_template: string;
  access_token_configured: boolean;
}

interface WebSettingsDto {
  close_servers_on_exit: boolean;
  close_servers_on_update: boolean;
  auto_accept_eula: boolean;
  default_max_memory: number;
  default_min_memory: number;
  default_port: number;
  default_jvm_args: string[];
  default_cpu_policy: CpuPolicyConfig;
  default_jvm_preset: JvmPresetConfig;
  console_font_size: number;
  console_font_family: string;
  console_letter_spacing: number;
  max_log_lines: number;
  background_opacity: number;
  background_blur: number;
  background_brightness: number;
  background_size: string;
  window_effect: WindowEffect;
  theme: string;
  font_size: number;
  font_family: string;
  memory_display_precision: number;
  color: string;
  text_color_overrides: TextColorOverrides;
  app_display_name: string;
  language: string;
  locale_layer_order: string[];
  developer_mode: boolean;
  close_action: string;
  minimal_mode: boolean;
  next_home_layout: NextHomeLayoutItem[];
  agreed_to_terms: boolean;
  onebot_11: WebOneBot11SettingsDto;
}

const DEFAULT_APP_SETTINGS: AppSettings = {
  close_servers_on_exit: true,
  close_servers_on_update: true,
  auto_accept_eula: false,
  default_max_memory: 4096,
  default_min_memory: 1024,
  default_port: 25565,
  default_java_path: "",
  default_jvm_args: [],
  default_cpu_policy: {
    mode: "off",
    count: null,
    explicit_set: null,
    sync_active_processor_count: true,
  },
  default_jvm_preset: {
    preset: "none",
  },
  console_font_size: 12,
  console_font_family: "",
  console_letter_spacing: 0,
  max_log_lines: 1000,
  cached_java_list: [],
  background_image: "",
  background_opacity: 0.3,
  background_blur: 0,
  background_brightness: 1,
  background_size: "cover",
  window_effect: "off",
  acrylic_enabled: false,
  theme: "auto",
  font_size: 14,
  font_family: "",
  memory_display_precision: 2,
  color: "default",
  text_color_overrides: {
    title: "",
    text: "",
    description: "",
  },
  app_display_name: "",
  language: "zh-CN",
  locale_layer_order: [],
  developer_mode: false,
  enable_desktop_web_ui: false,
  close_action: "ask",
  last_run_path: "",
  minimal_mode: false,
  next_home_layout: [],
  agreed_to_terms: false,
  onebot_11: {
    enabled: false,
    api_base_url: "",
    access_token: "",
    event_classes: ["output", "lifecycle"],
    structured_event_kinds: ["server_ready", "player_join", "player_leave", "chat", "error"],
    server_ids: [],
    targets: [],
    message_template: "[{server_id}] {kind}: {summary}",
  },
};

function normalizeBrowserSettings(dto: WebSettingsDto): AppSettings {
  return {
    ...DEFAULT_APP_SETTINGS,
    ...dto,
    onebot_11: {
      ...DEFAULT_APP_SETTINGS.onebot_11,
      enabled: dto.onebot_11.enabled,
      api_base_url: dto.onebot_11.api_base_url,
      event_classes: dto.onebot_11.event_classes,
      structured_event_kinds: dto.onebot_11.structured_event_kinds,
      server_ids: dto.onebot_11.server_ids,
      targets: dto.onebot_11.targets,
      message_template: dto.onebot_11.message_template,
    },
  };
}

function normalizeBrowserSettingsResult(result: UpdateSettingsResult): UpdateSettingsResult {
  return {
    settings: normalizeBrowserSettings(result.settings as unknown as WebSettingsDto),
    changed_groups: result.changed_groups,
  };
}

function browserCommand(desktopCommand: string, webCommand: string): string {
  return isBrowserEnv() ? webCommand : desktopCommand;
}

export const settingsApi = {
  async get(): Promise<AppSettings> {
    if (isBrowserEnv()) {
      const result = await tauriInvoke<WebSettingsDto>("get_web_settings");
      return normalizeBrowserSettings(result);
    }

    return tauriInvoke("get_settings");
  },
  async getDataDirStatus(): Promise<DataDirStatus> {
    return tauriInvoke("get_data_dir_status");
  },
  async initializeDataDir(path: string): Promise<DataDirChangeResult> {
    return tauriInvoke("initialize_data_dir", { path });
  },
  async changeDataDir(path: string, migrateExisting = true): Promise<DataDirChangeResult> {
    return tauriInvoke("change_data_dir", {
      request: {
        path,
        migrate_existing: migrateExisting,
      },
    });
  },
  async getPluginDirStatus(): Promise<PluginDirStatus> {
    return tauriInvoke("get_plugin_dir_status");
  },
  async changePluginDir(path: string, migrateExisting = true): Promise<PluginDirChangeResult> {
    return tauriInvoke("change_plugin_dir", {
      request: {
        path,
        migrate_existing: migrateExisting,
      },
    });
  },
  async save(settings: AppSettings): Promise<void> {
    await tauriInvoke(browserCommand("save_settings", "save_web_settings"), { settings });
  },
  async saveWithDiff(settings: AppSettings): Promise<UpdateSettingsResult> {
    if (isBrowserEnv()) {
      const result = await tauriInvoke<UpdateSettingsResult>("save_web_settings", { settings });
      return normalizeBrowserSettingsResult(result);
    }

    return tauriInvoke("save_settings_with_diff", { settings });
  },
  async updatePartial(partial: PartialSettings): Promise<UpdateSettingsResult> {
    const result = await tauriInvoke<UpdateSettingsResult>(
      browserCommand("update_settings_partial", "update_web_settings_partial"),
      { partial },
    );

    return isBrowserEnv() ? normalizeBrowserSettingsResult(result) : result;
  },
  async reset(): Promise<AppSettings> {
    if (isBrowserEnv()) {
      const result = await tauriInvoke<WebSettingsDto>("reset_web_settings");
      return normalizeBrowserSettings(result);
    }

    return tauriInvoke("reset_settings");
  },
  async exportJson(): Promise<string> {
    return tauriInvoke(browserCommand("export_settings", "export_web_settings"));
  },
  async importJson(json: string): Promise<AppSettings> {
    if (isBrowserEnv()) {
      const result = await tauriInvoke<WebSettingsDto>("import_web_settings", { json });
      return normalizeBrowserSettings(result);
    }

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
