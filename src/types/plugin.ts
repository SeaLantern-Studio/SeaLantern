export interface PluginSettingOption {
  value: string;
  label: string;
}

export interface PluginSettingField {
  key: string;
  label: string;
  type: "string" | "number" | "boolean" | "select" | "textarea" | "checkbox" | "color";
  display?: "button-group";
  default?: any;
  description?: string;
  options?: PluginSettingOption[];

  rows?: number;

  maxlength?: number;
}

export interface PluginAuthor {
  name: string;
  email?: string;
  url?: string;
}

export interface PluginPage {
  id: string;
  title: string;
  icon?: string;
  path: string;
}

export interface PluginSidebarConfig {
  label: string;
  icon?: string;
  priority?: number;
}

export type SidebarMode = "none" | "self" | "category" | "child";

export interface SidebarCategoryConfig {
  mode?: SidebarMode;
  label: string;
  icon?: string;
  show_dependents?: boolean;
  priority?: number;

  after?: string;

  parent?: string;
}

export interface PluginUiConfig {
  pages?: PluginPage[];
  sidebar?: PluginSidebarConfig;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: PluginAuthor;
  main: string;
  icon?: string;
  repository?: string;
  permissions?: string[];
  events?: string[];
  settings?: PluginSettingField[];
  ui?: PluginUiConfig;
  dependencies?: PluginDependency[];
  optional_dependencies?: PluginDependency[];

  sidebar?: SidebarCategoryConfig;

  locales?: Record<string, { name?: string; description?: string }>;

  capabilities?: string[];

  include?: string[];

  theme_var_map?: Record<string, string>;

  presets?: Record<string, Record<string, string>>;
}

export type PluginState = "loaded" | "enabled" | "disabled" | { error: string };

export interface PluginInfo {
  manifest: PluginManifest;
  state: PluginState;
  path: string;

  missing_dependencies?: MissingDependency[];
}

export interface PluginNavItem {
  plugin_id: string;
  label: string;
  icon?: string;
  priority: number;
}

export interface PluginUpdateInfo {
  plugin_id: string;
  current_version: string;
  latest_version: string;
  download_url?: string;
  changelog?: string;
}

export type PluginDependency =
  | string
  | {
      id: string;
      version?: string;
    };

export interface MissingDependency {
  id: string;
  version_requirement?: string;
  required: boolean;
}

export interface PluginInstallResult {
  plugin: PluginInfo;
  missing_dependencies: MissingDependency[];
  untrusted_url?: boolean;
}

export interface BatchInstallError {
  path: string;
  error: string;
}

export interface BatchInstallResult {
  success: PluginInstallResult[];
  failed: BatchInstallError[];
}

export interface SidebarItem {
  pluginId: string;
  label: string;
  icon?: string;
  mode: SidebarMode;
  showDependents: boolean;
  priority: number;

  isDefault?: boolean;

  after?: string;

  parent?: string;

  children?: SidebarItem[];
}

export type PluginUiAction =
  | "inject"
  | "remove"
  | "update"
  | "remove_all"
  | "hide"
  | "show"
  | "disable"
  | "enable"
  | "insert"
  | "remove_selector"
  | "set_style"
  | "set_attribute"
  | "query"
  | "element_get_text"
  | "element_get_value"
  | "element_get_attribute"
  | "element_get_attributes"
  | "element_click"
  | "element_set_value"
  | "element_check"
  | "element_select"
  | "element_focus"
  | "element_blur"
  | "element_on_change"
  | "inject_css"
  | "remove_css"
  | "toast";

export interface PluginPermissionLog {
  plugin_id: string;
  log_type: "command" | "api_call";
  action: string;
  detail: string;
  timestamp: number;
}

export interface PluginLogEvent {
  plugin_id: string;
  level: string;
  message: string;
}

export type PermissionDangerLevel = "normal" | "dangerous" | "critical";

export interface PermissionMetadata {
  id: string;
  name: string;
  description: string;
  danger_level: PermissionDangerLevel;
  icon: string;
}

export const PERMISSION_METADATA: Record<string, PermissionMetadata> = {
  network: {
    id: "network",
    name: "plugins.permission.network",
    description: "plugins.permission.network_desc",
    danger_level: "dangerous",
    icon: "Globe",
  },
  fs: {
    id: "fs",
    name: "plugins.permission.fs",
    description: "plugins.permission.fs_desc",
    danger_level: "normal",
    icon: "Folder",
  },
  server: {
    id: "server",
    name: "plugins.permission.server",
    description: "plugins.permission.server_desc",
    danger_level: "dangerous",
    icon: "Server",
  },
  console: {
    id: "console",
    name: "plugins.permission.console",
    description: "plugins.permission.console_desc",
    danger_level: "dangerous",
    icon: "Terminal",
  },
  element: {
    id: "element",
    name: "plugins.permission.element",
    description: "plugins.permission.element_desc",
    danger_level: "dangerous",
    icon: "MousePointer",
  },
  system: {
    id: "system",
    name: "plugins.permission.system",
    description: "plugins.permission.system_desc",
    danger_level: "dangerous",
    icon: "Monitor",
  },
  log: {
    id: "log",
    name: "plugins.permission.log",
    description: "plugins.permission.log_desc",
    danger_level: "normal",
    icon: "FileText",
  },
  storage: {
    id: "storage",
    name: "plugins.permission.storage",
    description: "plugins.permission.storage_desc",
    danger_level: "normal",
    icon: "HardDrive",
  },
  api: {
    id: "api",
    name: "plugins.permission.api",
    description: "plugins.permission.api_desc",
    danger_level: "normal",
    icon: "Plug",
  },
  ui: {
    id: "ui",
    name: "plugins.permission.ui",
    description: "plugins.permission.ui_desc",
    danger_level: "normal",
    icon: "Palette",
  },
  "ui.component.read": {
    id: "ui.component.read",
    name: "plugins.permission.ui_component_read",
    description: "plugins.permission.ui_component_read_desc",
    danger_level: "normal",
    icon: "FileText",
  },
  "ui.component.write": {
    id: "ui.component.write",
    name: "plugins.permission.ui_component_write",
    description: "plugins.permission.ui_component_write_desc",
    danger_level: "dangerous",
    icon: "FileText",
  },
  "ui.component.proxy": {
    id: "ui.component.proxy",
    name: "plugins.permission.ui_component_proxy",
    description: "plugins.permission.ui_component_proxy_desc",
    danger_level: "dangerous",
    icon: "AlertTriangle",
  },
  "ui.component.create": {
    id: "ui.component.create",
    name: "plugins.permission.ui_component_create",
    description: "plugins.permission.ui_component_create_desc",
    danger_level: "dangerous",
    icon: "LayoutGrid",
  },
  execute_program: {
    id: "execute_program",
    name: "plugins.permission.execute_program",
    description: "plugins.permission.execute_program_desc",
    danger_level: "critical",
    icon: "AlertTriangle",
  },
  plugin_folder_access: {
    id: "plugin_folder_access",
    name: "plugins.permission.plugin_folder_access",
    description: "plugins.permission.plugin_folder_access_desc",
    danger_level: "critical",
    icon: "LockOpen",
  },
};

export function getPermissionMetadata(permissionId: string): PermissionMetadata {
  return (
    PERMISSION_METADATA[permissionId] || {
      id: permissionId,
      name: permissionId,
      description: "",
      danger_level: "normal",
      icon: "HelpCircle",
    }
  );
}

export function groupPermissionsByDangerLevel(permissions: string[]): {
  critical: PermissionMetadata[];
  dangerous: PermissionMetadata[];
  normal: PermissionMetadata[];
} {
  const result = {
    critical: [] as PermissionMetadata[],
    dangerous: [] as PermissionMetadata[],
    normal: [] as PermissionMetadata[],
  };

  for (const perm of permissions) {
    const metadata = getPermissionMetadata(perm);
    result[metadata.danger_level].push(metadata);
  }

  return result;
}

export function hasDangerousPermissions(permissions: string[]): boolean {
  return permissions.some((perm) => {
    const metadata = getPermissionMetadata(perm);
    return metadata.danger_level === "dangerous" || metadata.danger_level === "critical";
  });
}

export function getLocalizedPluginName(manifest: PluginManifest, locale: string): string {
  return manifest.locales?.[locale]?.name ?? manifest.name;
}

export function getLocalizedPluginDescription(manifest: PluginManifest, locale: string): string {
  return manifest.locales?.[locale]?.description ?? manifest.description;
}
