<script setup lang="ts">
import { computed, ref } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import { getPermissionMetadata } from "@type/plugin";
import { Lock } from "lucide-vue-next";
import { SLModal } from "@components/common";

interface Props {
  pluginId: string;
  permissions: string[];
}

interface PermissionLogSummaryItem {
  name: string;
  count: number;
  lastTimestamp: number;
  sampleDetail: string;
}

interface PermissionDisplayItem {
  id: string;
  label: string;
  description: string;
}

const props = defineProps<Props>();
const pluginStore = usePluginStore();
const visible = ref(false);

const DECLARED_PERMISSION_GROUPS: Record<string, string[]> = {
  ui: [
    "ui.inject_html",
    "ui.update_html",
    "ui.remove_html",
    "ui.insert",
    "ui.remove",
    "ui.hide",
    "ui.show",
    "ui.disable",
    "ui.enable",
    "ui.set_style",
    "ui.set_attribute",
    "ui.inject_css",
    "ui.remove_css",
    "ui.toast",
    "ui.component.read",
    "ui.component.write",
    "ui.component.proxy",
    "ui.component.create",
    "ui.component.list",
    "ui.component.get",
    "ui.component.set",
    "ui.component.call",
    "ui.component.on",
    "ui.component.create_instance",
  ],
  element: [
    "element.query",
    "element.exists",
    "element.visible",
    "element.enabled",
    "element.get_text",
    "element.get_value",
    "element.get_attribute",
    "element.get_attributes",
    "element.click",
    "element.set_value",
    "element.check",
    "element.select",
    "element.focus",
    "element.blur",
    "element.on_change",
    "element.off_change",
    "element.form_fill",
  ],
  api: ["plugins_api.call", "plugins_api.register", "plugins_api.unregister", "plugins_api.list"],
  network: ["http.fetch", "http.request", "http.download", "http.upload"],
  system: ["system.info", "system.os", "system.cpu", "system.memory"],
  storage: ["storage.get", "storage.keys", "storage.set", "storage.remove"],
  log: ["log.debug", "log.info", "log.warn", "log.error"],
  execute_program: [
    "process.exec",
    "process.get",
    "process.list",
    "process.read_output",
    "process.kill",
  ],
  plugin_folder_access: ["plugins.read", "plugins.list", "plugins.write"],
};

const FS_SCOPE_PERMISSIONS = ["fs.data", "fs.server", "fs.global"] as const;
const FS_ACTION_MAP: Record<string, string> = {
  read: "read",
  read_binary: "read",
  write: "write",
  mkdir: "write",
  remove: "delete",
  list: "list",
  info: "meta",
  exists: "meta",
  get_path: "meta",
  copy: "transfer",
  move: "transfer",
  rename: "transfer",
};

function uniqueSorted(values: string[]): string[] {
  return Array.from(new Set(values)).toSorted((a: string, b: string) => a.localeCompare(b));
}

function getPermissionLabel(permission: string): string {
  const meta = getPermissionMetadata(permission);
  return i18n.te(meta.name) ? i18n.t(meta.name) : meta.id;
}

function getPermissionDesc(permission: string): string {
  const meta = getPermissionMetadata(permission);
  return i18n.te(meta.description) ? i18n.t(meta.description) : "";
}

function createPermissionItem(permission: string): PermissionDisplayItem {
  return {
    id: permission,
    label: getPermissionLabel(permission),
    description: getPermissionDesc(permission),
  };
}

function deriveFsPermissionsFromLog(action: string, detail: string): string[] {
  if (!action.startsWith("sl.fs.")) {
    return [];
  }

  const fsAction = action.slice("sl.fs.".length);
  const mappedAction = FS_ACTION_MAP[fsAction];
  const scope = detail.split(":", 1)[0]?.trim();

  if (scope && FS_SCOPE_PERMISSIONS.includes(scope as (typeof FS_SCOPE_PERMISSIONS)[number])) {
    const scopePermission = `fs.${scope}`;
    return mappedAction
      ? [scopePermission, `${scopePermission}.${mappedAction}`]
      : [scopePermission];
  }

  return mappedAction ? [`fs.${mappedAction}`] : [`fs.${fsAction}`];
}

function derivePermissionFromLog(action: string): string | null {
  if (!action) {
    return null;
  }

  if (action.startsWith("sl.fs.")) {
    return deriveFsPermissionsFromLog(action, "")[0] ?? null;
  }

  if (action.startsWith("sl.ui.")) {
    const uiAction = action.slice("sl.ui.".length);
    const componentActionMap: Record<string, string> = {
      component_list: "ui.component.list",
      component_get: "ui.component.get",
      component_set: "ui.component.set",
      component_call: "ui.component.call",
      component_on: "ui.component.on",
      component_create: "ui.component.create",
      component_proxy: "ui.component.proxy",
      component_read: "ui.component.read",
      component_write: "ui.component.write",
    };

    if (componentActionMap[uiAction]) {
      return componentActionMap[uiAction];
    }

    const basicUiActionMap: Record<string, string> = {
      inject_html: "ui.inject_html",
      update_html: "ui.update_html",
      remove_html: "ui.remove_html",
    };

    return basicUiActionMap[uiAction] ?? `ui.${uiAction}`;
  }

  if (action.startsWith("sl.element.")) {
    const elementAction = action.slice("sl.element.".length);
    const elementActionMap: Record<string, string> = {
      element_exists: "element.exists",
      element_is_visible: "element.visible",
      element_is_enabled: "element.enabled",
      element_get_text: "element.get_text",
      element_get_value: "element.get_value",
      element_get_attribute: "element.get_attribute",
      element_get_attributes: "element.get_attributes",
      element_click: "element.click",
      element_set_value: "element.set_value",
      element_check: "element.check",
      element_select: "element.select",
      element_focus: "element.focus",
      element_blur: "element.blur",
      element_on_change: "element.on_change",
      element_off_change: "element.off_change",
      element_form_fill: "element.form_fill",
      query: "element.query",
      get_attributes: "element.get_attributes",
      get_attribute: "element.get_attribute",
      get_text: "element.get_text",
      get_value: "element.get_value",
      click: "element.click",
      set_value: "element.set_value",
      check: "element.check",
      select: "element.select",
      focus: "element.focus",
      blur: "element.blur",
      on_change: "element.on_change",
      off_change: "element.off_change",
      form_fill: "element.form_fill",
    };
    return elementActionMap[elementAction] ?? `element.${elementAction}`;
  }

  if (action.startsWith("sl.process.")) {
    return `process.${action.slice("sl.process.".length)}`;
  }

  if (action.startsWith("sl.plugins.")) {
    return `plugins.${action.slice("sl.plugins.".length)}`;
  }

  if (action.startsWith("sl.http.")) {
    return `http.${action.slice("sl.http.".length)}`;
  }

  if (action.startsWith("sl.system.")) {
    return `system.${action.slice("sl.system.".length)}`;
  }

  if (action.startsWith("sl.storage.")) {
    return `storage.${action.slice("sl.storage.".length)}`;
  }

  if (action.startsWith("sl.log.")) {
    return `log.${action.slice("sl.log.".length)}`;
  }

  return action;
}

const logs = computed(() => pluginStore.getPermissionLogs(props.pluginId));

const recentLogs = computed(() => logs.value.slice(-50).reverse());

const totalCallCount = computed(() => logs.value.length);

const lastCallTime = computed(() => {
  const lastLog = logs.value.length > 0 ? logs.value[logs.value.length - 1] : null;
  return lastLog ? formatTime(lastLog.timestamp) : "-";
});

const declaredPermissions = computed<PermissionDisplayItem[]>(() => {
  return uniqueSorted(props.permissions).map(createPermissionItem);
});

const declaredSubPermissions = computed<PermissionDisplayItem[]>(() => {
  const expanded = uniqueSorted(
    props.permissions.flatMap((permission) => {
      if (permission === "fs") {
        return FS_SCOPE_PERMISSIONS.map((scopePermission) => scopePermission);
      }

      const direct = permission.includes(".") ? [permission] : [];
      return [...direct, ...(DECLARED_PERMISSION_GROUPS[permission] || [])];
    }),
  );

  return expanded.map(createPermissionItem);
});

const usedSubPermissions = computed<PermissionDisplayItem[]>(() => {
  const derived = uniqueSorted(
    logs.value.flatMap((log) => {
      if (log.action.startsWith("sl.fs.")) {
        return deriveFsPermissionsFromLog(log.action, log.detail);
      }

      const permission = derivePermissionFromLog(log.action);
      return permission ? [permission] : [];
    }),
  );

  return derived.map(createPermissionItem);
});

const typeStats = computed<PermissionLogSummaryItem[]>(() => {
  const stats = new Map<string, PermissionLogSummaryItem>();
  for (const log of logs.value) {
    const existing = stats.get(log.log_type);
    if (existing) {
      existing.count += 1;
      if (log.timestamp >= existing.lastTimestamp) {
        existing.lastTimestamp = log.timestamp;
        existing.sampleDetail = log.detail;
      }
      continue;
    }
    stats.set(log.log_type, {
      name: log.log_type,
      count: 1,
      lastTimestamp: log.timestamp,
      sampleDetail: log.detail,
    });
  }
  return Array.from(stats.values()).toSorted(
    (a: PermissionLogSummaryItem, b: PermissionLogSummaryItem) => b.count - a.count,
  );
});

const actionStats = computed<PermissionLogSummaryItem[]>(() => {
  const stats = new Map<string, PermissionLogSummaryItem>();
  for (const log of logs.value) {
    const existing = stats.get(log.action);
    if (existing) {
      existing.count += 1;
      if (log.timestamp >= existing.lastTimestamp) {
        existing.lastTimestamp = log.timestamp;
        existing.sampleDetail = log.detail;
      }
      continue;
    }
    stats.set(log.action, {
      name: log.action,
      count: 1,
      lastTimestamp: log.timestamp,
      sampleDetail: log.detail,
    });
  }
  return Array.from(stats.values()).toSorted(
    (a: PermissionLogSummaryItem, b: PermissionLogSummaryItem) => b.count - a.count,
  );
});

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString(i18n.getLocale(), {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}
</script>

<template>
  <div class="permission-panel-root">
    <button
      class="permission-btn"
      :title="i18n.t('plugins.permission.panel_btn_title')"
      type="button"
      @click="visible = true"
    >
      <Lock :size="14" :stroke-width="2" />
      <span class="permission-btn-text">{{ i18n.t("plugins.permission.panel_btn_text") }}</span>
    </button>

    <SLModal
      :visible="visible"
      :title="i18n.t('plugins.permission.panel_title')"
      width="min(840px, calc(100vw - 32px))"
      @close="visible = false"
    >
      <div class="permission-panel-body">
        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_declared") }}</div>
          <div class="permission-tags">
            <span
              v-for="perm in declaredPermissions"
              :key="perm.id"
              class="permission-tag"
              :title="perm.description"
            >
              {{ perm.label }}
            </span>
            <span v-if="declaredPermissions.length === 0" class="empty-hint">
              {{ i18n.t("plugins.permission.panel_no_permissions") }}
            </span>
          </div>
        </div>

        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_declared_details") }}</div>
          <div class="permission-sub-list">
            <div v-for="perm in declaredSubPermissions" :key="perm.id" class="permission-sub-item">
              <div class="permission-sub-id">{{ perm.id }}</div>
              <div class="permission-sub-label">{{ perm.label }}</div>
              <div v-if="perm.description" class="permission-sub-desc">{{ perm.description }}</div>
            </div>
            <div v-if="declaredSubPermissions.length === 0" class="empty-hint">
              {{ i18n.t("plugins.permission.panel_no_permission_details") }}
            </div>
          </div>
        </div>

        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_used_details") }}</div>
          <div class="permission-sub-list">
            <div v-for="perm in usedSubPermissions" :key="perm.id" class="permission-sub-item">
              <div class="permission-sub-id">{{ perm.id }}</div>
              <div class="permission-sub-label">{{ perm.label }}</div>
              <div v-if="perm.description" class="permission-sub-desc">{{ perm.description }}</div>
            </div>
            <div v-if="usedSubPermissions.length === 0" class="empty-hint">
              {{ i18n.t("plugins.permission.panel_no_used_details") }}
            </div>
          </div>
        </div>

        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_overview") }}</div>
          <div class="overview-grid">
            <div class="overview-card">
              <span class="overview-label">{{
                i18n.t("plugins.permission.panel_total_calls")
              }}</span>
              <span class="overview-value">{{ totalCallCount }}</span>
            </div>
            <div class="overview-card">
              <span class="overview-label">{{ i18n.t("plugins.permission.panel_last_call") }}</span>
              <span class="overview-value overview-value-time">{{ lastCallTime }}</span>
            </div>
          </div>
        </div>

        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_type_stats") }}</div>
          <div class="stats-list">
            <div v-for="stat in typeStats" :key="stat.name" class="stats-item">
              <div class="stats-main-row">
                <span class="stats-name">{{ stat.name }}</span>
                <span class="stats-count">{{
                  i18n.t("plugins.permission.panel_call_count", { count: stat.count })
                }}</span>
              </div>
              <div class="stats-sub-row">
                <span class="stats-time">{{ formatTime(stat.lastTimestamp) }}</span>
                <span v-if="stat.sampleDetail" class="stats-detail" :title="stat.sampleDetail">{{
                  stat.sampleDetail
                }}</span>
              </div>
            </div>
            <div v-if="typeStats.length === 0" class="empty-hint">
              {{ i18n.t("plugins.permission.panel_no_logs") }}
            </div>
          </div>
        </div>

        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_action_stats") }}</div>
          <div class="stats-list">
            <div v-for="stat in actionStats" :key="stat.name" class="stats-item">
              <div class="stats-main-row">
                <span class="stats-name">{{ stat.name }}</span>
                <span class="stats-count">{{
                  i18n.t("plugins.permission.panel_call_count", { count: stat.count })
                }}</span>
              </div>
              <div class="stats-sub-row">
                <span class="stats-time">{{ formatTime(stat.lastTimestamp) }}</span>
                <span v-if="stat.sampleDetail" class="stats-detail" :title="stat.sampleDetail">{{
                  stat.sampleDetail
                }}</span>
              </div>
            </div>
            <div v-if="actionStats.length === 0" class="empty-hint">
              {{ i18n.t("plugins.permission.panel_no_logs") }}
            </div>
          </div>
        </div>

        <div class="panel-section">
          <div class="section-title">{{ i18n.t("plugins.permission.panel_recent_logs") }}</div>
          <div class="recent-log-list">
            <div
              v-for="(log, index) in recentLogs"
              :key="`${log.timestamp}-${index}`"
              class="recent-log-item"
            >
              <div class="recent-log-head">
                <span class="recent-log-type">{{ log.log_type }}</span>
                <span class="recent-log-time">{{ formatTime(log.timestamp) }}</span>
              </div>
              <div class="recent-log-action">{{ log.action }}</div>
              <div v-if="log.detail" class="recent-log-detail">{{ log.detail }}</div>
            </div>
            <div v-if="recentLogs.length === 0" class="empty-hint">
              {{ i18n.t("plugins.permission.panel_no_logs") }}
            </div>
          </div>
        </div>
      </div>
    </SLModal>
  </div>
</template>

<style scoped>
.permission-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: none;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-xs);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.permission-btn:hover {
  background: var(--sl-bg-hover);
  color: var(--sl-text-primary);
}

.permission-btn-text {
  font-weight: 500;
}

.permission-panel-body {
  max-height: min(78vh, 760px);
  overflow-y: auto;
  padding-right: 4px;
}

.panel-section {
  margin-bottom: 18px;
}

.panel-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--sl-text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.permission-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.permission-tag {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: var(--sl-radius-lg);
  background: var(--sl-primary-alpha, rgba(59, 130, 246, 0.15));
  color: var(--sl-primary);
  font-size: var(--sl-font-size-xs);
  font-weight: 500;
}

.permission-sub-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 8px;
}

.permission-sub-item {
  padding: 10px 12px;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.permission-sub-id {
  font-size: 12px;
  font-weight: 600;
  color: var(--sl-text-primary);
  font-family: monospace;
  word-break: break-all;
}

.permission-sub-label {
  margin-top: 4px;
  font-size: 12px;
  color: var(--sl-text-secondary);
}

.permission-sub-desc {
  margin-top: 4px;
  font-size: 11px;
  color: var(--sl-text-tertiary);
  line-height: 1.5;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.overview-card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
}

.overview-label {
  font-size: 11px;
  color: var(--sl-text-secondary);
}

.overview-value {
  font-size: 16px;
  font-weight: 700;
  color: var(--sl-text-primary);
}

.overview-value-time {
  font-size: 13px;
}

.stats-list,
.recent-log-list {
  max-height: 200px;
  overflow-y: auto;
}

.stats-item,
.recent-log-item {
  padding: 8px 10px;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  margin-bottom: 6px;
}

.stats-item:last-child,
.recent-log-item:last-child {
  margin-bottom: 0;
}

.stats-main-row,
.recent-log-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.stats-sub-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-top: 4px;
}

.stats-name,
.recent-log-action {
  font-size: 12px;
  color: var(--sl-text-primary);
  font-family: monospace;
  word-break: break-all;
}

.stats-count,
.stats-time,
.recent-log-time,
.recent-log-type {
  font-size: 11px;
  color: var(--sl-text-secondary);
  flex-shrink: 0;
}

.stats-detail,
.recent-log-detail {
  font-size: 11px;
  color: var(--sl-text-tertiary);
  word-break: break-all;
}

.recent-log-action {
  margin-top: 4px;
}

.recent-log-detail {
  margin-top: 4px;
}

.empty-hint {
  font-size: 12px;
  color: var(--sl-text-tertiary);
  font-style: italic;
}

.permission-panel-body::-webkit-scrollbar,
.stats-list::-webkit-scrollbar,
.recent-log-list::-webkit-scrollbar {
  width: 4px;
}

.permission-panel-body::-webkit-scrollbar-track,
.stats-list::-webkit-scrollbar-track,
.recent-log-list::-webkit-scrollbar-track {
  background: transparent;
}

.permission-panel-body::-webkit-scrollbar-thumb,
.stats-list::-webkit-scrollbar-thumb,
.recent-log-list::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: 2px;
}

.permission-panel-body::-webkit-scrollbar-thumb:hover,
.stats-list::-webkit-scrollbar-thumb:hover,
.recent-log-list::-webkit-scrollbar-thumb:hover {
  background: var(--sl-text-tertiary);
}

@media (max-width: 640px) {
  .overview-grid {
    grid-template-columns: 1fr;
  }

  .permission-sub-list {
    grid-template-columns: 1fr;
  }
}
</style>
