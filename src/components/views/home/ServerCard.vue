<script setup lang="ts">
import { computed } from "vue";
import { Pencil, FolderOpen, Check, X } from "lucide-vue-next";
import type { ServerInstance } from "@type/server";
import { i18n } from "@language";
import { systemApi } from "@api/system";
import { useRouter } from "vue-router";
import {
  actionLoading,
  editingServerId,
  editName,
  editLoading,
  formatServerPath,
  formatMemoryMB,
  getStatusText,
  handleStart,
  handleStop,
  startEditServerName,
  saveServerName,
  cancelEdit,
  showDeleteConfirmInput,
  showChangePathModal,
} from "@utils/serverUtils";
import { useServerStore } from "@stores/serverStore";

const props = defineProps<{
  server: ServerInstance;
}>();

const store = useServerStore();
const router = useRouter();

// 把 store.statuses[server.id]?.status 集中到一个 computed
// 避免模板中 6+ 次响应式读取
const status = computed<string | undefined>(() => store.statuses[props.server.id]?.status);
const actionLoadingForServer = computed(() => actionLoading.value[props.server.id] === true);
const isEditing = computed(() => editingServerId.value === props.server.id);

async function handlePathClick(path: string) {
  try {
    await systemApi.openFolder(path);
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}

function handleConsole() {
  store.setCurrentServer(props.server.id);
  router.push("/console/" + props.server.id);
}

function handleConfig() {
  store.setCurrentServer(props.server.id);
  router.push("/config/" + props.server.id);
}

function handleStartupConfig() {
  store.setCurrentServer(props.server.id);
  router.push({ path: "/config/" + props.server.id, query: { tab: "startup" } });
}

function getStatusClass(s: string | undefined): string {
  return s === "Running"
    ? "running"
    : s === "Starting"
      ? "starting"
      : s === "Stopping"
        ? "stopping"
        : "stopped";
}
</script>

<template>
  <cmz-card class="server-card" :data-server-id="server.id">
    <div class="status-badge-container">
      <div class="status-indicator" :class="getStatusClass(status)">
        <span class="status-dot"></span>
        <span class="status-label">{{ getStatusText(status) }}</span>
      </div>
    </div>

    <div class="server-card-header">
      <div class="server-name-container">
        <template v-if="isEditing">
          <div class="inline-edit">
            <input
              type="text"
              v-model="editName"
              class="server-name-input"
              @keyup.enter="saveServerName(server.id)"
              @keyup.esc="cancelEdit"
              @blur="saveServerName(server.id)"
            />
            <div class="inline-edit-actions">
              <button
                class="inline-edit-btn save"
                @click="saveServerName(server.id)"
                :disabled="!editName.trim() || editLoading"
                :class="{ loading: editLoading }"
              >
                <Check :size="16" />
              </button>
              <button class="inline-edit-btn cancel" @click="cancelEdit" :disabled="editLoading">
                <X :size="16" />
              </button>
            </div>
          </div>
        </template>
        <template v-else>
          <h4 class="server-name">{{ server.name }}</h4>
          <button
            class="edit-server-name"
            @click="startEditServerName(server)"
            :title="i18n.t('common.edit_server_name')"
          >
            <Pencil :size="16" />
          </button>
        </template>
      </div>
      <div class="server-meta">
        <span class="meta-tag core-type">{{ server.core_type }}</span>
        <span class="meta-tag">{{ i18n.t("home.port") }} {{ server.port }}</span>
        <span class="meta-tag clickable" @click="handleStartupConfig">{{
          formatMemoryMB(server.max_memory)
        }}</span>
      </div>
    </div>

    <div class="server-card-content">
      <div
        class="server-card-path text-mono text-caption"
        :title="server.path"
        @click="handlePathClick(server.path)"
      >
        <span class="server-path-text">{{ formatServerPath(server.path) }}</span>
        <FolderOpen class="folder-icon" :size="16" />
      </div>
    </div>

    <div class="server-card-actions">
      <div class="action-group primary-actions">
        <cmz-button
          v-if="status === 'Stopped' || status === 'Error' || !status"
          size="sm"
          :loading="actionLoadingForServer"
          :disabled="actionLoadingForServer || status === 'Stopping'"
          @click="handleStart(server.id)"
          >{{ i18n.t("home.start") }}</cmz-button
        >
        <cmz-button
          v-else
          variant="solid"
          color="#ef4444"
          size="sm"
          :loading="actionLoadingForServer"
          :disabled="actionLoadingForServer || status === 'Stopping'"
          @click="handleStop(server.id)"
          >{{ i18n.t("home.stop") }}</cmz-button
        >
      </div>
      <div class="action-group secondary-actions">
        <cmz-button variant="ghost" size="sm" @click="handleConsole">
          {{ i18n.t("common.console") }}
        </cmz-button>
        <cmz-button variant="ghost" size="sm" @click="handleConfig">
          {{ i18n.t("common.config_edit") }}
        </cmz-button>
        <cmz-button variant="ghost" size="sm" @click="showChangePathModal(server)">
          {{ i18n.t("home.change_path") }}
        </cmz-button>
        <cmz-button variant="ghost" size="sm" @click="showDeleteConfirmInput(server)">
          {{ i18n.t("home.delete") }}
        </cmz-button>
      </div>
    </div>
  </cmz-card>
</template>

<style scoped>
.server-card {
  display: flex;
  flex-direction: column;
  position: relative;
  height: 100%;
  min-height: 200px;
  cursor: pointer;
  /* 所有交互过渡统一节奏,避免各部件各动各的 */
  transition:
    transform var(--sl-transition-normal),
    box-shadow var(--sl-transition-normal),
    border-color var(--sl-transition-normal),
    background-color var(--sl-transition-normal);
}

/* 整体卡片 hover:克制上浮 + 阴影加强 + 边框高亮,不搞花里胡哨的渐变条 */
.server-card:hover {
  transform: translateY(-1px);
  box-shadow: var(--sl-shadow-elevated);
  border-color: var(--sl-primary-light);
}

.status-badge-container {
  position: absolute;
  top: var(--sl-space-sm);
  right: var(--sl-space-sm);
  z-index: 10;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 4px 12px;
  border-radius: var(--sl-radius-full);
  font-size: 0.75rem;
  font-weight: 500;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-indicator.running {
  background: rgba(34, 197, 94, 0.1);
  color: var(--sl-success);
}

.status-indicator.running .status-dot {
  background: var(--sl-success);
}

.status-indicator.stopped {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
}

.status-indicator.stopped .status-dot {
  background: var(--sl-text-tertiary);
}

.status-indicator.starting,
.status-indicator.stopping {
  background: rgba(245, 158, 11, 0.1);
  color: var(--sl-warning);
}

.status-indicator.starting .status-dot,
.status-indicator.stopping .status-dot {
  background: var(--sl-warning);
  animation: statusPulse 2s ease-in-out infinite;
  /* 频繁动画元素提升到独立图层,减少重绘开销 */
  will-change: transform, opacity;
}

/* 状态呼吸灯:用透明度变化代替缩放,避免脉冲浮动感 */
@keyframes statusPulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.45;
  }
}

.server-card-header {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  padding-right: 100px;
}

.server-name-container {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
}

.server-name {
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.edit-server-name {
  opacity: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: var(--sl-transition-normal);
  padding: 4px;
  border-radius: var(--sl-radius-sm);
  flex-shrink: 0;
  color: var(--sl-text-secondary);
}

.server-card:hover .edit-server-name {
  opacity: 1;
}

.edit-server-name:hover {
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
}

.inline-edit {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.server-name-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--sl-primary);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 1rem;
  font-weight: 600;
  outline: none;
  transition: var(--sl-transition-normal);
}

.server-name-input:focus {
  box-shadow: 0 0 0 2px var(--sl-primary-bg);
}

.inline-edit-actions {
  display: flex;
  gap: 4px;
  align-items: center;
}

.inline-edit-btn {
  width: 24px;
  height: 24px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--sl-transition-normal);
}

.inline-edit-btn.save {
  background: var(--sl-primary);
  color: white;
}

.inline-edit-btn.save:hover:not(:disabled) {
  background: var(--sl-primary-dark);
}

.inline-edit-btn.cancel {
  background: var(--sl-bg-secondary);
  color: var(--sl-text-secondary);
  border-color: var(--sl-border);
}

.inline-edit-btn.cancel:hover:not(:disabled) {
  background: var(--sl-bg-tertiary);
}

.inline-edit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.inline-edit-btn.loading {
  opacity: 0.8;
}

.server-meta {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  display: flex;
  flex-wrap: wrap;
  gap: var(--sl-space-xs);
}

.meta-tag {
  background: var(--sl-bg-tertiary);
  padding: 4px 10px;
  border-radius: var(--sl-radius-full);
  white-space: nowrap;
  border: 1px solid var(--sl-border);
  transition: var(--sl-transition-normal);
}

/* 带点击事件的 meta-tag(如内存配置入口)给个指针提示 */
.meta-tag[onclick],
.meta-tag[style*="cursor"],
.meta-tag.clickable {
  cursor: pointer;
}

.meta-tag:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
}

.meta-tag.core-type {
  background: var(--sl-primary-bg);
  border-color: var(--sl-primary-light);
  color: var(--sl-primary);
  font-weight: 500;
}

/* core-type hover 保持克制,不突然跳成全主色实心块 */
.meta-tag.core-type:hover {
  background: color-mix(in srgb, var(--sl-primary-bg) 65%, var(--sl-primary) 35%);
  border-color: var(--sl-primary);
}

.server-card-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: var(--sl-space-sm) 0;
}

.server-card-path {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
  font-size: 0.75rem;
  color: var(--sl-text-secondary);
  background: var(--sl-bg-tertiary);
  padding: 8px var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  transition: var(--sl-transition-normal);
  cursor: pointer;
  user-select: none;
}

.server-path-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.folder-icon {
  flex-shrink: 0;
  opacity: 0.6;
  transition: var(--sl-transition-normal);
  color: var(--sl-text-secondary);
}

.server-card-path:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
  color: var(--sl-text-primary);
}

.server-card-path:hover .folder-icon {
  opacity: 1;
  color: var(--sl-text-primary);
}

.server-card-actions {
  display: flex;
  gap: var(--sl-space-sm);
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
  align-items: center;
  justify-content: space-between;
  margin-top: auto;
}

.action-group {
  display: flex;
  gap: var(--sl-space-xs);
  align-items: center;
}

.primary-actions :deep(.cmz-button),
.secondary-actions :deep(.cmz-button) {
  border-radius: var(--sl-radius-md);
  /* 按钮不再自己上浮,避免和卡片上浮叠加造成分层感 */
  transition:
    color var(--sl-transition-normal),
    background-color var(--sl-transition-normal),
    border-color var(--sl-transition-normal),
    box-shadow var(--sl-transition-normal),
    opacity var(--sl-transition-normal);
}

.primary-actions :deep(.cmz-button) {
  min-width: 72px;
}

@media (max-width: 640px) {
  .server-card-actions {
    flex-wrap: wrap;
  }

  .action-group {
    flex: 1;
  }

  .primary-actions {
    flex: 0 0 auto;
  }

  .secondary-actions {
    justify-content: flex-end;
  }
}

@media (max-width: 480px) {
  .server-card-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .action-group {
    width: 100%;
    justify-content: center;
  }

  .action-group :deep(.cmz-button) {
    flex: 1;
  }
}
</style>
