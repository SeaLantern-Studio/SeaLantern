<script setup lang="ts">
import { i18n } from "@language";
import type { MarketPlugin, MarketFeedbackType } from "./usePluginMarket";
import type { MarketPluginInfo } from "@api/plugin";
import { Puzzle, X } from "lucide-vue-next";

defineProps<{
  plugin: MarketPlugin | null;
  pluginDetail: MarketPluginInfo | null;
  detailLoading: boolean;
  installingPluginId: string | null;
  getIconUrl: (plugin: MarketPlugin) => string | null;
  resolveI18n: (value: Record<string, string> | string | undefined) => string;
  getInstallButtonText: (pluginId: string) => string;
  isInstalled: (pluginId: string) => boolean;
  isInstalledAndEnabled: (pluginId: string) => boolean;
  getPermissionLevel: (perm: string) => MarketFeedbackType | "normal" | "critical" | "dangerous";
  getPermissionLabel: (perm: string) => string;
  getPermissionDesc: (perm: string) => string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "install", plugin: MarketPlugin): void;
}>();
</script>

<template>
  <Teleport to="body">
    <div v-if="plugin" class="modal-overlay" @click.self="emit('close')">
      <div class="detail-modal glass-strong">
        <button class="modal-close" @click="emit('close')">
          <X :size="20" />
        </button>
        <div class="detail-header">
          <div class="detail-icon">
            <img
              v-if="getIconUrl(plugin)"
              :src="getIconUrl(plugin)!"
              :alt="resolveI18n(plugin.name)"
            />
            <Puzzle v-else :size="48" :stroke-width="1.5" />
          </div>
          <div class="detail-title">
            <h2>{{ resolveI18n(plugin.name) }}</h2>
            <span class="detail-version">{{ plugin.version ? "v" + plugin.version : "" }}</span>
            <span class="detail-author">by {{ plugin.author?.name }}</span>
          </div>
        </div>
        <div v-if="detailLoading" class="detail-loading">
          <div class="loading-spinner"></div>
        </div>
        <div v-else class="detail-body">
          <p class="detail-desc">
            {{ resolveI18n(pluginDetail?.description || plugin.description) }}
          </p>
          <div v-if="pluginDetail?.permissions?.length" class="detail-section">
            <h3>{{ i18n.t("market.permissions") }}</h3>
            <div class="permission-badges">
              <span
                v-for="perm in pluginDetail.permissions"
                :key="perm"
                :class="['perm-badge', `perm-badge--${getPermissionLevel(perm)}`]"
                :title="getPermissionDesc(perm)"
              >
                {{ getPermissionLabel(perm) }}
              </span>
            </div>
          </div>
          <div v-if="pluginDetail?.changelog" class="detail-section">
            <h3>{{ i18n.t("market.changelog") }}</h3>
            <pre class="changelog">{{ pluginDetail.changelog }}</pre>
          </div>
        </div>
        <div class="detail-footer">
          <button
            :class="[
              'install-btn-lg',
              {
                installed: isInstalled(plugin.id),
                'is-enabled': isInstalledAndEnabled(plugin.id),
              },
            ]"
            :disabled="isInstalled(plugin.id) || installingPluginId === plugin.id"
            :title="isInstalledAndEnabled(plugin.id) ? i18n.t('market.plugin_running_warning') : ''"
            @click="emit('install', plugin)"
          >
            {{ getInstallButtonText(plugin.id) }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.detail-modal {
  width: 90%;
  max-width: 560px;
  max-height: 80vh;
  overflow-y: auto;
  border-radius: var(--sl-radius-lg);
  padding: 24px;
  position: relative;
}

.modal-close {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 8px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  border-radius: var(--sl-radius-md);
}

.modal-close:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.detail-header {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
}

.detail-icon {
  flex-shrink: 0;
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--sl-text-tertiary);
}

.detail-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-lg);
}

.detail-title h2 {
  margin: 0;
  font-size: 20px;
  color: var(--sl-text-primary);
}

.detail-version {
  display: inline-block;
  padding: 2px 8px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 12px;
  color: var(--sl-text-tertiary);
  margin-top: 4px;
}

.detail-author {
  display: block;
  font-size: 13px;
  color: var(--sl-text-secondary);
  margin-top: 4px;
}

.detail-loading {
  display: flex;
  justify-content: center;
  padding: 32px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.detail-body {
  margin-bottom: 20px;
}

.detail-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0 0 16px;
}

.detail-section {
  margin-top: 16px;
}

.detail-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px;
}

.permission-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-badge {
  padding: 3px 10px;
  border-radius: var(--sl-radius-lg);
  font-size: 12px;
  font-weight: 500;
  cursor: default;
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  border: 1px solid var(--sl-border);
}

.perm-badge--dangerous {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
  border-color: rgba(245, 158, 11, 0.3);
}

.perm-badge--critical {
  background: rgba(239, 68, 68, 0.12);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.changelog {
  margin: 0;
  padding: 12px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-md);
  font-size: 12px;
  color: var(--sl-text-secondary);
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}

.detail-footer {
  display: flex;
  justify-content: flex-end;
}

.install-btn-lg {
  padding: 10px 32px;
  border-radius: 8px;
  border: none;
  background: var(--sl-primary);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.install-btn-lg:hover:not(:disabled) {
  opacity: 0.9;
}

.install-btn-lg:disabled {
  cursor: not-allowed;
}

.install-btn-lg.installed {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
}

.install-btn-lg.is-enabled {
  background: var(--sl-bg-tertiary);
  color: var(--sl-warning);
  font-size: 13px;
}
</style>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.detail-modal {
  width: 90%;
  max-width: 560px;
  max-height: 80vh;
  overflow-y: auto;
  border-radius: var(--sl-radius-lg);
  padding: 24px;
  position: relative;
}

.modal-close {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 8px;
  border: none;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  border-radius: var(--sl-radius-md);
}

.modal-close:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.detail-header {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
}

.detail-icon {
  flex-shrink: 0;
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--sl-text-tertiary);
}

.detail-icon img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: var(--sl-radius-lg);
}

.detail-title h2 {
  margin: 0;
  font-size: 20px;
  color: var(--sl-text-primary);
}

.detail-version {
  display: inline-block;
  padding: 2px 8px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-xs);
  font-size: 12px;
  color: var(--sl-text-tertiary);
  margin-top: 4px;
}

.detail-author {
  display: block;
  font-size: 13px;
  color: var(--sl-text-secondary);
  margin-top: 4px;
}

.detail-loading {
  display: flex;
  justify-content: center;
  padding: 32px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.detail-body {
  margin-bottom: 20px;
}

.detail-desc {
  font-size: 14px;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0 0 16px;
}

.detail-section {
  margin-top: 16px;
}

.detail-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 8px;
}

.permission-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.perm-badge {
  padding: 3px 10px;
  border-radius: var(--sl-radius-lg);
  font-size: 12px;
  font-weight: 500;
  cursor: default;
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
  border: 1px solid var(--sl-border);
}

.perm-badge--dangerous {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
  border-color: rgba(245, 158, 11, 0.3);
}

.perm-badge--critical {
  background: rgba(239, 68, 68, 0.12);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.changelog {
  margin: 0;
  padding: 12px;
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-md);
  font-size: 12px;
  color: var(--sl-text-secondary);
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}

.detail-footer {
  display: flex;
  justify-content: flex-end;
}

.install-btn-lg {
  padding: 10px 32px;
  border-radius: 8px;
  border: none;
  background: var(--sl-primary);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.install-btn-lg:hover:not(:disabled) {
  opacity: 0.9;
}

.install-btn-lg:disabled {
  cursor: not-allowed;
}

.install-btn-lg.installed {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-secondary);
}

.install-btn-lg.is-enabled {
  background: var(--sl-bg-tertiary);
  color: var(--sl-warning);
  font-size: 13px;
}
</style>
