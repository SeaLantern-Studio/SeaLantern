<script setup lang="ts">
import { Pencil, FolderOpen, Check, X } from "lucide-vue-next";
import SLBadge from "@components/common/SLBadge.vue";
import SLButton from "@components/common/SLButton.vue";
import type { ServerInstance } from "@type/server";
import { i18n } from "@language";
import { systemApi } from "@api/system";
import { useRouter } from "vue-router";
import {
  actionLoading,
  editingServerId,
  editName,
  editLoading,
  deletingServerId,
  inputServerName,
  deleteError,
  isClosing,
  formatServerPath,
  getStatusVariant,
  getStatusText,
  handleStart,
  handleStop,
  startEditServerName,
  saveServerName,
  cancelEdit,
  showDeleteConfirmInput,
  confirmDelete,
  cancelDelete,
  handleAnimationEnd,
} from "@utils/serverUtils";
import { useServerStore } from "@stores/serverStore";

const props = defineProps<{
  server: ServerInstance;
}>();

const store = useServerStore();
const router = useRouter();

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
</script>

<template>
  <div class="server-card glass-card">
    <div class="server-card-header">
      <div class="server-info">
        <div class="server-name-container">
          <template v-if="editingServerId === server.id">
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
                <button
                  class="inline-edit-btn cancel"
                  @click="cancelEdit"
                  :disabled="editLoading"
                >
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
          <span>{{ server.core_type }}</span>
          <span>{{ i18n.t("home.port") }} {{ server.port }}</span>
          <span>{{ server.max_memory }}MB</span>
        </div>
      </div>
      <SLBadge
        :text="getStatusText(store.statuses[server.id]?.status)"
        :variant="getStatusVariant(store.statuses[server.id]?.status)"
        size="large"
      />
    </div>

    <div
      class="server-card-path text-mono text-caption"
      :title="server.path"
      @click="handlePathClick(server.path)"
    >
      <span class="server-path-text">{{ formatServerPath(server.jar_path) }}</span>
      <FolderOpen class="folder-icon" :size="16" />
    </div>

    <div class="server-card-actions">
      <SLButton
        v-if="
          store.statuses[server.id]?.status === 'Stopped' ||
          store.statuses[server.id]?.status === 'Error' ||
          !store.statuses[server.id]?.status
        "
        variant="primary"
        size="sm"
        :loading="actionLoading[server.id]"
        :disabled="actionLoading[server.id] || store.statuses[server.id]?.status === 'Stopping'"
        @click="handleStart(server.id)"
        >{{ i18n.t("home.start") }}</SLButton
      >
      <SLButton
        v-else
        variant="danger"
        size="sm"
        :loading="actionLoading[server.id]"
        :disabled="actionLoading[server.id] || store.statuses[server.id]?.status === 'Stopping'"
        @click="handleStop(server.id)"
        >{{ i18n.t("home.stop") }}</SLButton
      >
      <SLButton
        variant="ghost"
        size="sm"
        @click="handleConsole"
      >
        {{ i18n.t("common.console") }}
      </SLButton>
      <SLButton
        variant="ghost"
        size="sm"
        @click="handleConfig"
      >
        {{ i18n.t("common.config_edit") }}
      </SLButton>
      <SLButton variant="ghost" size="sm" @click="showDeleteConfirmInput(server)">
        {{ i18n.t("home.delete") }}
      </SLButton>
    </div>

    <div
      v-if="deletingServerId === server.id"
      :class="['delete-confirm-area', { closing: isClosing }]"
      @animationend="handleAnimationEnd"
    >
      <p
        class="delete-confirm-message"
        v-html="
          i18n.t('home.delete_confirm_message', {
            server: '<strong>' + server.name + '</strong>',
          })
        "
      ></p>
      <div class="delete-input-group">
        <input
          type="text"
          v-model="inputServerName"
          class="delete-input"
          :placeholder="i18n.t('home.delete_input_placeholder')"
          @keyup.enter="confirmDelete"
          @keyup.esc="cancelDelete"
        />
        <div v-if="deleteError" class="delete-error">{{ deleteError }}</div>
      </div>
      <div class="delete-actions">
        <SLButton variant="ghost" size="sm" @click="cancelDelete">{{
          i18n.t("home.delete_cancel")
        }}</SLButton>
        <SLButton variant="danger" size="sm" @click="confirmDelete">{{
          i18n.t("home.delete_confirm")
        }}</SLButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.server-card {
  padding: var(--sl-space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  border-radius: var(--sl-radius-lg);
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
  background: var(--sl-bg-secondary);
  box-shadow: var(--sl-shadow-sm);
}

.server-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--sl-shadow-lg);
  border-color: var(--sl-primary-light);
}

.server-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, var(--sl-primary), var(--sl-secondary));
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s ease;
}

.server-card:hover::before {
  transform: scaleX(1);
}

.server-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--sl-space-md);
}

.server-info {
  flex: 1;
  min-width: 0;
}

.server-name-container {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
  margin-bottom: var(--sl-space-xs);
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

.server-card-header :deep(.sl-badge) {
  flex-shrink: 0;
}

.edit-server-name {
  opacity: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s ease;
  padding: 4px;
  border-radius: var(--sl-radius-sm);
  flex-shrink: 0;
}

.server-card:hover .edit-server-name {
  opacity: 1;
}

.edit-server-name:hover {
  background: var(--sl-bg-secondary);
  transform: scale(1.05);
}

.server-meta {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  display: flex;
  flex-wrap: wrap;
  gap: var(--sl-space-xs);
  margin-top: var(--sl-space-xs);
}

.server-meta span {
  background: var(--sl-bg-tertiary);
  padding: 4px 12px;
  border-radius: var(--sl-radius-full);
  white-space: nowrap;
  border: 1px solid var(--sl-border);
  transition: all 0.2s ease;
}

.server-meta span:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
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
  transition: all 0.2s ease;
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
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
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

.server-card-path {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
  font-size: 0.75rem;
  color: var(--sl-text-secondary);
  background: var(--sl-bg-tertiary);
  padding: var(--sl-space-sm) var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  margin: var(--sl-space-xs) 0;
  border: 1px solid var(--sl-border);
  transition: all 0.2s ease;
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
  transition: opacity 0.2s ease;
  color: var(--sl-text-secondary);
}

.server-card-path:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
  color: var(--sl-text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.server-card-path:hover .folder-icon {
  opacity: 1;
  color: var(--sl-text-primary);
}

.server-card-path:active {
  transform: translateY(1px);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
}

.server-card-actions {
  display: flex;
  gap: var(--sl-space-sm);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
  flex-wrap: wrap;
  align-items: center;
}

.server-card-actions :deep(.sl-button) {
  flex: 1;
  min-width: 90px;
  border-radius: var(--sl-radius-md);
  transition: all 0.2s ease;
}

.server-card-actions :deep(.sl-button:hover) {
  transform: translateY(-1px);
}

.server-card-actions :deep(.sl-button:not(.sl-button--variant-primary):not(.sl-button--variant-danger)) {
  flex: 0 0 auto;
  min-width: unset;
}

@media (max-width: 480px) {
  .server-card-actions {
    flex-direction: column;
    align-items: stretch;
  }

  .server-card-actions :deep(.sl-button) {
    flex: 1;
    min-width: unset;
  }
}

.delete-confirm-area {
  margin-top: var(--sl-space-md);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border);
  animation: slideDown 0.3s ease forwards;
}

.delete-confirm-area.closing {
  animation: slideUp 0.2s ease forwards;
}

@keyframes slideDown {
  from {
    opacity: 0;
    max-height: 0;
    padding-top: 0;
    margin-top: 0;
  }
  to {
    opacity: 1;
    max-height: 200px;
    padding-top: var(--sl-space-md);
    margin-top: var(--sl-space-md);
  }
}

@keyframes slideUp {
  from {
    opacity: 1;
    max-height: 200px;
    padding-top: var(--sl-space-md);
    margin-top: var(--sl-space-md);
  }
  to {
    opacity: 0;
    max-height: 0;
    padding-top: 0;
    margin-top: 0;
  }
}

.delete-confirm-message {
  font-size: 0.875rem;
  margin-bottom: var(--sl-space-sm);
  line-height: 1.4;
}

.delete-input-group {
  margin-bottom: var(--sl-space-sm);
}

.delete-input {
  width: 100%;
  padding: var(--sl-space-sm) var(--sl-space-md);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  font-size: 0.75rem;
  outline: none;
  transition: all 0.2s ease;
}

.delete-input:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 2px var(--sl-primary-bg);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
}

.delete-input:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-primary-light);
  color: var(--sl-text-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.delete-input:active {
  transform: translateY(1px);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
}

.delete-error {
  margin-top: var(--sl-space-xs);
  font-size: 0.75rem;
  color: var(--sl-error);
}

.delete-actions {
  display: flex;
  gap: var(--sl-space-xs);
  justify-content: flex-end;
  margin-top: var(--sl-space-sm);
}
</style>
