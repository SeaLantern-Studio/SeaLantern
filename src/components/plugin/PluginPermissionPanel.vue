<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import { getPermissionMetadata } from "@type/plugin";
import { Lock, X } from "lucide-vue-next";

interface Props {
  pluginId: string;
  permissions: string[];
}

const props = defineProps<Props>();
const pluginStore = usePluginStore();

const isOpen = ref(false);
const panelPosition = ref({ top: 0, left: 0 });

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
    .toSorted((a, b) => b.count - a.count);
});

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString(i18n.getLocale(), {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function updatePanelPosition() {
  if (!buttonRef.value) return;
  const rect = buttonRef.value.getBoundingClientRect();
  const panelWidth = 320;
  const panelHeight = 400;
  const padding = 8;

  let top = rect.bottom + 4;
  let left = rect.left;

  // 确保面板不超出视口底部
  if (top + panelHeight > window.innerHeight - padding) {
    top = rect.top - panelHeight - 4;
  }

  // 确保面板不超出视口右侧
  if (left + panelWidth > window.innerWidth - padding) {
    left = window.innerWidth - panelWidth - padding;
  }

  // 确保面板不超出视口左侧
  if (left < padding) {
    left = padding;
  }

  panelPosition.value = { top, left };
}

async function togglePanel() {
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    await nextTick();
    updatePanelPosition();
  }
}

function closePanel() {
  isOpen.value = false;
}

const panelRef = ref<HTMLElement | null>(null);
const buttonRef = ref<HTMLElement | null>(null);

function handleClickOutside(event: MouseEvent) {
  if (!isOpen.value) return;
  const target = event.target as Node;
  if (
    panelRef.value &&
    !panelRef.value.contains(target) &&
    buttonRef.value &&
    !buttonRef.value.contains(target)
  ) {
    closePanel();
  }
}

function handleScroll() {
  if (isOpen.value) {
    updatePanelPosition();
  }
}

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  window.addEventListener("scroll", handleScroll, true);
  window.addEventListener("resize", handleScroll);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  window.removeEventListener("scroll", handleScroll, true);
  window.removeEventListener("resize", handleScroll);
});
</script>

<template>
  <div class="permission-panel-wrapper" :class="{ 'permission-panel-wrapper--open': isOpen }">
    <button
      ref="buttonRef"
      class="permission-btn"
      :class="{ 'permission-btn--active': isOpen }"
      @click.stop="togglePanel"
      :title="i18n.t('plugins.permission.panel_btn_title')"
    >
      <Lock :size="14" :stroke-width="2" />
      <span class="permission-btn-text">{{ i18n.t("plugins.permission.panel_btn_text") }}</span>
    </button>

    <Teleport to="body">
      <div
        v-if="isOpen"
        ref="panelRef"
        class="permission-panel glass"
        :style="{ top: `${panelPosition.top}px`, left: `${panelPosition.left}px` }"
      >
        <div class="panel-header">
          <span class="panel-title">{{ i18n.t("plugins.permission.panel_title") }}</span>
          <button class="panel-close" @click="closePanel">
            <X :size="14" :stroke-width="2" />
          </button>
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
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.permission-panel-wrapper {
  position: relative;
  display: inline-flex;
  z-index: 1;
}

.permission-panel-wrapper--open {
  z-index: 9999;
}

.permission-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: none;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.permission-btn:hover {
  background: var(--sl-bg-hover);
  color: var(--sl-text-primary);
}

.permission-btn--active {
  background: var(--sl-primary);
  color: white;
}

.permission-btn-text {
  font-weight: 500;
}

.permission-panel {
  position: fixed;
  width: 320px;
  max-height: 400px;
  border-radius: var(--sl-radius-lg);
  background: var(--sl-surface);
  backdrop-filter: blur(12px);
  border: 1px solid var(--sl-border);
  box-shadow: var(--sl-shadow-lg);
  z-index: 9999;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--sl-border);
  background: var(--sl-bg-tertiary);
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.panel-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: var(--sl-radius-xs);
  background: transparent;
  color: var(--sl-text-tertiary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.panel-close:hover {
  background: var(--sl-bg-hover);
  color: var(--sl-text-primary);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
}

.panel-section {
  margin-bottom: 16px;
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
  position: relative;
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  border-radius: var(--sl-radius-lg);
  background: var(--sl-primary-alpha, rgba(59, 130, 246, 0.15));
  color: var(--sl-primary);
  font-size: 12px;
  font-weight: 500;
  cursor: default;
}

.permission-tag-tooltip {
  display: none;
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  background: var(--sl-bg-tertiary);
  border: 1px solid var(--sl-border);
  color: var(--sl-text-secondary);
  font-size: 11px;
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

.permission-tag:hover .permission-tag-tooltip {
  display: block;
}

.command-list {
  max-height: 120px;
  overflow-y: auto;
}

.command-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  margin-bottom: 4px;
}

.command-item:last-child {
  margin-bottom: 0;
}

.command-action {
  flex: 1;
  font-size: 12px;
  color: var(--sl-text-primary);
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: 8px;
}

.command-time {
  font-size: 11px;
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
}

.api-stats {
  max-height: 100px;
  overflow-y: auto;
}

.api-stat-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  border-radius: var(--sl-radius-xs);
  background: var(--sl-bg-tertiary);
  margin-bottom: 4px;
}

.api-stat-item:last-child {
  margin-bottom: 0;
}

.api-name {
  font-size: 12px;
  color: var(--sl-text-primary);
  font-family: monospace;
}

.api-count {
  font-size: 11px;
  color: var(--sl-text-secondary);
  font-weight: 500;
}

.empty-hint {
  font-size: 12px;
  color: var(--sl-text-tertiary);
  font-style: italic;
}

.panel-fade-enter-active,
.panel-fade-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.panel-fade-enter-from,
.panel-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.panel-content::-webkit-scrollbar,
.command-list::-webkit-scrollbar,
.api-stats::-webkit-scrollbar {
  width: 4px;
}

.panel-content::-webkit-scrollbar-track,
.command-list::-webkit-scrollbar-track,
.api-stats::-webkit-scrollbar-track {
  background: transparent;
}

.panel-content::-webkit-scrollbar-thumb,
.command-list::-webkit-scrollbar-thumb,
.api-stats::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: 2px;
}

.panel-content::-webkit-scrollbar-thumb:hover,
.command-list::-webkit-scrollbar-thumb:hover,
.api-stats::-webkit-scrollbar-thumb:hover {
  background: var(--sl-text-tertiary);
}
</style>
