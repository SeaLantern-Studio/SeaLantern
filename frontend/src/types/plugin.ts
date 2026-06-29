import permissionCatalog from "@shared/plugin-permissions.json";

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

export interface PluginProgram {
  path: string;
}

export type PluginSource = "local" | "builtin";

export type PluginRuntimeKind = "lua" | "rust";

export type PluginTrustLevelDisplay = "builtin" | "trusted" | "standard_sandbox" | "unreviewed";

export type PluginExecutionClass = "builtin_full" | "trusted_full" | "sandboxed" | "restricted";

export type PluginReviewStatus =
  | "builtin"
  | "sealantern_reviewed"
  | "unreviewed"
  | "revoked";

export type PluginIntegrityStatus =
  | "bundled"
  | "verified_hash"
  | "verified_signature"
  | "unsigned"
  | "mismatch"
  | "unknown";

export type PluginTrustedPolicySource =
  | "builtin"
  | "bundled_snapshot"
  | "remote_signed_catalog"
  | "local_attestation"
  | "none";

export type PluginDistributionClass =
  | "builtin"
  | "market"
  | "standard_package"
  | "manual_import"
  | "local_directory"
  | "trusted_catalog"
  | "unknown";

export type PluginPermissionProfile =
  | "builtin_full"
  | "trusted_full"
  | "sandboxed_normal"
  | "sandboxed_extended"
  | "unreviewed";

export type PluginEnableGrantScope = "once" | "version" | "hash";

export interface PluginEnableConfirmation {
  grant_scope: PluginEnableGrantScope;
}

export type PluginEnableBlockReason = "user_confirmation_required" | "revoked";

export interface PluginActions {
  can_toggle: boolean;
  can_delete: boolean;
  can_check_update: boolean;
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
  engines?: {
    sealantern?: string;
  };
  permissions?: string[];
  events?: string[];
  programs?: PluginProgram[];
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
  source: PluginSource;
  runtime: PluginRuntimeKind;
  actions: PluginActions;

  missing_dependencies?: MissingDependency[];
  trust_level_display: PluginTrustLevelDisplay;
  execution_class: PluginExecutionClass;
  review_status: PluginReviewStatus;
  integrity_status: PluginIntegrityStatus;
  trusted_policy_source: PluginTrustedPolicySource;
  permission_profile: PluginPermissionProfile;
  publisher_id?: string | null;
  distribution_class: PluginDistributionClass;
  trusted_catalog_matched: boolean;
  hash_matched: boolean;
  verified_hash?: string | null;
  verified_signature: boolean;
  reviewed_at?: string | null;
  revoked: boolean;
  exceeds_standard_sandbox: boolean;
  requires_explicit_consent: boolean;
}

export function isBuiltinPlugin(plugin: PluginInfo): boolean {
  return plugin.source === "builtin";
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
  suggested_trust_level?: PluginTrustLevelDisplay | null;
  integrity_status?: PluginIntegrityStatus | null;
  review_status?: PluginReviewStatus | null;
  distribution_class?: PluginDistributionClass | null;
  permission_profile?: PluginPermissionProfile | null;
  trusted_catalog_matched?: boolean;
  hash_matched?: boolean;
  exceeds_standard_sandbox?: boolean;
  install_notices?: PluginInstallIssue[];
}

export interface PluginInstallIssue {
  code: string;
  args?: Record<string, unknown>;
}

export interface PluginEnableResult {
  success: boolean;
  disabled_plugins: string[];
  confirmation_required: boolean;
  block_reason?: PluginEnableBlockReason | null;
  plugin?: PluginInfo | null;
  grant_scope?: PluginEnableGrantScope | null;
  message?: string | null;
}

export interface BatchInstallError {
  path: string;
  error: string;
  issue?: PluginInstallIssue | null;
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
  | "element_exists"
  | "element_is_visible"
  | "element_is_enabled"
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
  | "element_off_change"
  | "element_form_fill"
  | "inject_css"
  | "remove_css"
  | "toast";

export interface PluginPermissionLog {
  plugin_id: string;
  log_type: string;
  action: string;
  detail: string;
  timestamp: number;
}

export interface PluginPermissionLogGroup {
  name: string;
  count: number;
  lastTimestamp: number;
  details: string[];
}

export interface PluginLogEvent {
  plugin_id: string;
  level: string;
  message: string;
}

export type PermissionDangerLevel = "normal" | "dangerous" | "critical";

export type PermissionRiskGroup =
  | "standard_sandbox_allowed"
  | "escalated_sandbox"
  | "trusted_only"
  | "unknown";

export interface PermissionMetadata {
  id: string;
  name: string;
  description: string;
  danger_level: PermissionDangerLevel;
  category: string;
  icon: string;
  aliases: string[];
  risk_group: PermissionRiskGroup;
  trusted_only: boolean;
  within_standard_ceiling: boolean;
  requires_explicit_consent: boolean;
}

export interface PermissionSemanticsSummary {
  standardCount: number;
  escalatedCount: number;
  trustedOnlyCount: number;
  outsideStandardCount: number;
  requiresConsentCount: number;
}

export const PERMISSION_METADATA: Record<string, PermissionMetadata> = Object.fromEntries(
  (permissionCatalog as PermissionMetadata[]).flatMap((permission) => {
    const entries: Array<[string, PermissionMetadata]> = [[permission.id, permission]];
    for (const alias of permission.aliases) {
      entries.push([alias, permission]);
    }
    return entries;
  }),
);

export function getPermissionMetadata(permissionId: string): PermissionMetadata {
  return (
    PERMISSION_METADATA[permissionId] || {
      id: permissionId,
      name: permissionId,
      description: "",
      danger_level: "normal",
      category: "custom",
      icon: "HelpCircle",
      aliases: [],
      risk_group: "unknown",
      trusted_only: false,
      within_standard_ceiling: false,
      requires_explicit_consent: false,
    }
  );
}

export function getPluginTrustLabel(plugin: PluginInfo): string {
  switch (plugin.trust_level_display) {
    case "builtin":
      return "builtin";
    case "trusted":
      return "trusted";
    case "unreviewed":
      return "unreviewed";
    case "standard_sandbox":
    default:
      return "sandboxed";
  }
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

export function summarizePermissionSemantics(permissions: string[]): PermissionSemanticsSummary {
  const summary: PermissionSemanticsSummary = {
    standardCount: 0,
    escalatedCount: 0,
    trustedOnlyCount: 0,
    outsideStandardCount: 0,
    requiresConsentCount: 0,
  };

  for (const permission of permissions) {
    const metadata = getPermissionMetadata(permission);
    switch (metadata.risk_group) {
      case "standard_sandbox_allowed":
        summary.standardCount += 1;
        break;
      case "escalated_sandbox":
        summary.escalatedCount += 1;
        break;
      case "trusted_only":
        summary.trustedOnlyCount += 1;
        break;
    }

    if (!metadata.within_standard_ceiling) {
      summary.outsideStandardCount += 1;
    }
    if (metadata.requires_explicit_consent) {
      summary.requiresConsentCount += 1;
    }
  }

  return summary;
}

export function getLocalizedPluginName(manifest: PluginManifest, locale: string): string {
  return manifest.locales?.[locale]?.name ?? manifest.name;
}

export function getLocalizedPluginDescription(manifest: PluginManifest, locale: string): string {
  return manifest.locales?.[locale]?.description ?? manifest.description;
}
