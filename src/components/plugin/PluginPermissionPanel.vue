<script setup lang="ts">
import { computed } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import { getPermissionMetadata } from "@type/plugin";
import { Lock } from "lucide-vue-next";
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuContent,
  DropdownMenuArrow,
} from "reka-ui";

interface Props {
  pluginId: string;
  permissions: string[];
}

const props = defineProps<Props>();
const pluginStore = usePluginStore();

function getPermissionLabel(permission: string): string {
  const meta = getPermissionMetadata(permission);
  return i18n.te(meta.name) ? i18n.t(meta.name) : meta.id;
}

function getPermissionDesc(permission: string): string {
  const meta = getPermissionMetadata(permission);
  return i18n.te(meta.description) ? i18n.t(meta.description) : "";
}

const logs = computed(() => {
  return pluginStore.getPermissionLogs(props.pluginId);
});

const commandLogs = computed(() => {
  return logs.value
    .filter((log) => log.log_type === "command")
    .slice(-50)
    .toReversed();
});

const apiStats = computed(() => {
  const stats = new Map<string, number>();
  logs.value
    .filter((log) => log.log_type === "api_call")
    .forEach((log) => {
      stats.set(log.action, (stats.get(log.action) || 0) + 1);
    });
  return Array.from(stats.entries())
    .map(([name, count]) => ({ name, count }))
    .toSorted((a: { count: number }, b: { count: number }) => b.count - a.count);
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
  <DropdownMenuRoot>
    <DropdownMenuTrigger
      class="permission-btn"
      :title="i18n.t('plugins.permission.panel_btn_title')"
    >
      <Lock :size="14" :stroke-width="2" />
      <span class="permission-btn-text">{{ i18n.t("plugins.permission.panel_btn_text") }}</span>
    </DropdownMenuTrigger>

    <DropdownMenuPortal>
      <DropdownMenuContent
        class="permission-panel"
        :side-offset="4"
        positionStrategy="fixed"
        :collision-padding="8"
      >
        <DropdownMenuArrow class="panel-arrow" />

        <div class="panel-header">
          <span class="panel-title">{{ i18n.t("plugins.permission.panel_title") }}</span>
        </div>

        <div class="panel-content">
          <div class="panel-section">
            <div class="section-title">{{ i18n.t("plugins.permission.panel_declared") }}</div>
            <div class="permission-tags">
              <span
                v-for="perm in permissions"
                :key="perm"
                class="permission-tag"
                :title="getPermissionDesc(perm)"
              >
                {{ getPermissionLabel(perm) }}
                <span v-if="getPermissionDesc(perm)" class="permission-tag-tooltip">{{
                  getPermissionDesc(perm)
                }}</span>
              </span>
              <span v-if="permissions.length === 0" class="empty-hint">
                {{ i18n.t("plugins.permission.panel_no_permissions") }}
              </span>
            </div>
          </div>

          <div class="panel-section">
            <div class="section-title">{{ i18n.t("plugins.permission.panel_command_log") }}</div>
            <div class="command-list">
              <div v-for="(log, index) in commandLogs" :key="index" class="command-item">
                <span class="command-action" :title="log.detail">{{ log.action }}</span>
                <span class="command-time">{{ formatTime(log.timestamp) }}</span>
              </div>
              <div v-if="commandLogs.length === 0" class="empty-hint">
                {{ i18n.t("plugins.permission.panel_no_commands") }}
              </div>
            </div>
          </div>

          <div class="panel-section">
            <div class="section-title">{{ i18n.t("plugins.permission.panel_api_stats") }}</div>
            <div class="api-stats">
              <div v-for="stat in apiStats" :key="stat.name" class="api-stat-item">
                <span class="api-name">{{ stat.name }}</span>
                <span class="api-count">{{
                  i18n.t("plugins.permission.panel_call_count", { count: stat.count })
                }}</span>
              </div>
              <div v-if="apiStats.length === 0" class="empty-hint">
                {{ i18n.t("plugins.permission.panel_no_api_calls") }}
              </div>
            </div>
          </div>
        </div>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
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

.permission-btn[data-state="open"] {
  background: var(--sl-primary);
  color: white;
}

.permission-btn-text {
  font-weight: 500;
}

:deep(.permission-panel) {
  width: 320px;
  max-height: 400px;
  border-radius: var(--sl-radius-lg);
  border: 1px solid var(--sl-glass-border, rgba(255, 255, 255, 0.5));
  box-shadow: var(--sl-shadow-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  z-index: 9999999;
}

:deep(.panel-arrow) {
  fill: var(--sl-surface);
}

:deep(.panel-header) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--sl-border);
  background: var(--sl-bg-tertiary);
}

:deep(.panel-title) {
  font-size: 14px;
  font-weight: 600;
  color: var(--sl-text-primary);
}

:deep(.panel-content) {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
}

:deep(.panel-section) {
  margin-bottom: 16px;
}

:deep(.panel-section:last-child) {
  margin-bottom: 0;
}

:deep(.section-title) {
  font-size: 12px;
  font-weight: 600;
  color: var(--sl-text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

:deep(.permission-tags) {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

:deep(.permission-tag) {
  position: relative;
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: var(--sl-radius-lg);
  background: var(--sl-primary-alpha, rgba(59, 130, 246, 0.15));
  color: var(--sl-primary);
  font-size: var(--sl-font-size-xs);
  font-weight: 500;
  cursor: default;
}

:deep(.permission-tag-tooltip) {
  display: none;
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  background: var(--sl-bg-tertiary);
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-xs);
  font-weight: 400;
  line-height: 1.5;
  padding: 6px 10px;
  border-radius: var(--sl-radius-md);
  width: max-content;
  max-width: 220px;
  white-space: normal;
  word-break: break-all;
  z-index: 1001;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  pointer-events: none;
}

:deep(.permission-tag:hover .permission-tag-tooltip) {
  display: block;
}

:deep(.command-list) {
  max-height: 120px;
  overflow-y: auto;
}

:deep(.command-item) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  margin-bottom: 4px;
}

:deep(.command-item:last-child) {
  margin-bottom: 0;
}

:deep(.command-action) {
  flex: 1;
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-primary);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: 8px;
}

:deep(.command-time) {
  font-size: 11px;
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
}

:deep(.api-stats) {
  max-height: 100px;
  overflow-y: auto;
}

:deep(.api-stat-item) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  margin-bottom: 4px;
}

:deep(.api-stat-item:last-child) {
  margin-bottom: 0;
}

:deep(.api-name) {
  font-size: 12px;
  color: var(--sl-text-primary);
  font-family: monospace;
}

:deep(.api-count) {
  font-size: 11px;
  color: var(--sl-text-secondary);
  font-weight: 500;
}

:deep(.empty-hint) {
  font-size: 12px;
  color: var(--sl-text-tertiary);
  font-style: italic;
}

:deep(.panel-content::-webkit-scrollbar),
:deep(.command-list::-webkit-scrollbar),
:deep(.api-stats::-webkit-scrollbar) {
  width: 4px;
}

:deep(.panel-content::-webkit-scrollbar-track),
:deep(.command-list::-webkit-scrollbar-track),
:deep(.api-stats::-webkit-scrollbar-track) {
  background: transparent;
}

:deep(.panel-content::-webkit-scrollbar-thumb),
:deep(.command-list::-webkit-scrollbar-thumb),
:deep(.api-stats::-webkit-scrollbar-thumb) {
  background: var(--sl-border);
  border-radius: 2px;
}

:deep(.panel-content::-webkit-scrollbar-thumb:hover),
:deep(.command-list::-webkit-scrollbar-thumb:hover),
:deep(.api-stats::-webkit-scrollbar-thumb:hover) {
  background: var(--sl-text-tertiary);
}
</style>
