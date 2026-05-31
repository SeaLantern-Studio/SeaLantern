<script setup lang="ts">
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import type { BatchInstallResult, MissingDependency } from "@type/plugin";
import { Trash, Trash2 } from "lucide-vue-next";

defineProps<{
  showSingleDeleteDialog: boolean;
  singleDeletePluginName: string;
  showBatchDeleteDialog: boolean;
  selectedCount: number;
  alertDialog: {
    show: boolean;
    title: string;
    message: string;
  };
  showDependencyModal: boolean;
  installedPluginName: string;
  missingDependencies: MissingDependency[];
  showBatchResultModal: boolean;
  batchInstallResult: BatchInstallResult | null;
  getDepDisplayName: (depId: string) => string;
}>();

const emit = defineEmits<{
  (e: "close-single-delete"): void;
  (e: "confirm-single-delete", deleteData: boolean): void;
  (e: "close-batch-delete"): void;
  (e: "confirm-batch-delete", deleteData: boolean): void;
  (e: "close-alert"): void;
  (e: "close-dependency"): void;
  (e: "go-market"): void;
  (e: "close-batch-result"): void;
}>();
</script>

<template>
  <SLModal
    :visible="showSingleDeleteDialog"
    :title="i18n.t('plugins.confirm_delete')"
    @close="emit('close-single-delete')"
  >
    <div class="batch-delete-dialog">
      <p class="dialog-message">
        {{ i18n.t("plugins.confirm_delete_message", { name: singleDeletePluginName }) }}
      </p>
      <div class="batch-delete-options">
        <SLButton
          variant="secondary"
          class="batch-delete-option"
          @click="emit('confirm-single-delete', true)"
        >
          <Trash2 class="option-icon delete-with-data" :size="20" />
          <span class="option-label">{{ i18n.t("plugins.delete_with_data") }}</span>
        </SLButton>
        <SLButton
          variant="secondary"
          class="batch-delete-option"
          @click="emit('confirm-single-delete', false)"
        >
          <Trash class="option-icon delete-without-data" :size="20" />
          <span class="option-label">{{ i18n.t("plugins.delete_without_data") }}</span>
        </SLButton>
      </div>
    </div>
    <template #footer>
      <SLButton variant="secondary" size="sm" @click="emit('close-single-delete')">{{
        i18n.t("plugins.cancel")
      }}</SLButton>
    </template>
  </SLModal>

  <SLModal
    :visible="showBatchDeleteDialog"
    :title="i18n.t('plugins.confirm_batch_delete')"
    @close="emit('close-batch-delete')"
  >
    <div class="batch-delete-dialog">
      <p class="dialog-message">
        {{ i18n.t("plugins.confirm_batch_delete_message", { count: selectedCount }) }}
      </p>
      <div class="batch-delete-options">
        <SLButton
          variant="secondary"
          class="batch-delete-option"
          @click="emit('confirm-batch-delete', true)"
        >
          <Trash2 class="option-icon delete-with-data" :size="20" />
          <span class="option-label">{{ i18n.t("plugins.delete_with_data") }}</span>
        </SLButton>
        <SLButton
          variant="secondary"
          class="batch-delete-option"
          @click="emit('confirm-batch-delete', false)"
        >
          <Trash class="option-icon delete-without-data" :size="20" />
          <span class="option-label">{{ i18n.t("plugins.delete_without_data") }}</span>
        </SLButton>
      </div>
    </div>
    <template #footer>
      <SLButton variant="secondary" size="sm" @click="emit('close-batch-delete')">{{
        i18n.t("plugins.cancel")
      }}</SLButton>
    </template>
  </SLModal>

  <SLModal
    :visible="alertDialog.show"
    :title="alertDialog.title"
    :auto-close="3000"
    @close="emit('close-alert')"
  >
    <p class="dialog-message">{{ alertDialog.message }}</p>
    <template #footer>
      <SLButton variant="primary" size="sm" @click="emit('close-alert')">{{
        i18n.t("plugins.ok")
      }}</SLButton>
    </template>
  </SLModal>

  <SLModal
    :visible="showDependencyModal"
    :title="i18n.t('plugins.missing_deps_title')"
    @close="emit('close-dependency')"
  >
    <div class="dependency-dialog">
      <p class="dependency-intro">
        {{ i18n.t("plugins.missing_deps_intro", { name: installedPluginName }) }}
      </p>
      <ul class="dependency-list">
        <li v-for="dep in missingDependencies" :key="dep.id" class="dependency-item">
          <span class="dependency-name">{{ getDepDisplayName(dep.id) }}</span>
          <span v-if="dep.version_requirement" class="dependency-version">{{
            dep.version_requirement
          }}</span>
          <span :class="['dependency-badge', dep.required ? 'required' : 'optional']">
            {{ dep.required ? i18n.t("plugins.dep_required") : i18n.t("plugins.dep_optional") }}
          </span>
        </li>
      </ul>
      <p class="dependency-hint">
        {{ i18n.t("plugins.missing_deps_hint") }}
      </p>
    </div>
    <template #footer>
      <SLButton variant="secondary" size="sm" @click="emit('close-dependency')">{{
        i18n.t("plugins.later")
      }}</SLButton>
      <SLButton variant="primary" size="sm" @click="emit('go-market')">{{
        i18n.t("plugins.go_market")
      }}</SLButton>
    </template>
  </SLModal>

  <SLModal
    :visible="showBatchResultModal"
    :title="i18n.t('plugins.batch_result_title')"
    @close="emit('close-batch-result')"
  >
    <div class="batch-result-dialog" v-if="batchInstallResult">
      <div v-if="batchInstallResult.success.length > 0" class="batch-success-section">
        <p class="batch-section-title">
          {{ i18n.t("plugins.batch_success", { count: batchInstallResult.success.length }) }}
        </p>
        <ul class="batch-list">
          <li
            v-for="item in batchInstallResult.success"
            :key="item.plugin.manifest.id"
            class="batch-item success"
          >
            <span class="batch-item-name">{{ item.plugin.manifest.name }}</span>
            <span class="batch-item-version">v{{ item.plugin.manifest.version }}</span>
          </li>
        </ul>
      </div>
      <div v-if="batchInstallResult.failed.length > 0" class="batch-failed-section">
        <p class="batch-section-title">
          {{ i18n.t("plugins.batch_failed", { count: batchInstallResult.failed.length }) }}
        </p>
        <ul class="batch-list">
          <li v-for="item in batchInstallResult.failed" :key="item.path" class="batch-item failed">
            <span class="batch-item-path">{{ item.path.split(/[/\\]/).pop() }}</span>
            <span class="batch-item-error">{{ item.error }}</span>
          </li>
        </ul>
      </div>
    </div>
    <template #footer>
      <SLButton variant="primary" size="sm" @click="emit('close-batch-result')">{{
        i18n.t("plugins.ok")
      }}</SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.batch-delete-dialog {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.batch-delete-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.batch-delete-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border: none;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.batch-delete-option:hover {
  background: var(--sl-border);
}

.batch-delete-option:active {
  transform: scale(0.98);
}

.batch-delete-option .option-icon {
  flex-shrink: 0;
}

.batch-delete-option .option-icon.delete-with-data {
  color: var(--sl-error);
}

.batch-delete-option .option-icon.delete-without-data {
  color: var(--sl-warning);
}

.batch-delete-option .option-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.dialog-message {
  margin: 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-line;
}

.dependency-dialog {
  padding: 4px 0;
}

.dependency-intro {
  margin: 0 0 16px 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
}

.dependency-list {
  list-style: none;
  margin: 0 0 16px 0;
  padding: 0;
}

.dependency-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  margin-bottom: 8px;
  background: var(--sl-bg-tertiary, rgba(255, 255, 255, 0.05));
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border, rgba(255, 255, 255, 0.08));
}

.dependency-item:last-child {
  margin-bottom: 0;
}

.dependency-name {
  font-weight: 500;
  color: var(--sl-text-primary, #e2e8f0);
  font-size: 14px;
}

.dependency-version {
  font-size: 12px;
  color: var(--sl-text-tertiary, #64748b);
  font-family: monospace;
}

.dependency-badge {
  margin-left: auto;
  padding: 2px 8px;
  border-radius: var(--sl-radius-xs);
  font-size: 11px;
  font-weight: 500;
}

.dependency-badge.required {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.dependency-badge.optional {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.dependency-hint {
  margin: 0;
  color: var(--sl-text-tertiary, #64748b);
  font-size: 13px;
  line-height: 1.5;
}

.batch-result-dialog {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.batch-section-title {
  font-size: 14px;
  color: var(--sl-text-primary);
  margin: 0 0 8px 0;
}

.batch-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.batch-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: var(--sl-radius-sm);
  font-size: 13px;
}

.batch-item.success {
  background: var(--sl-success-bg);
  border: 1px solid var(--sl-success);
}

.batch-item.failed {
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
  flex-direction: column;
  align-items: flex-start;
}

.batch-item-name {
  color: var(--sl-text-primary);
  font-weight: 500;
}

.batch-item-version {
  color: var(--sl-text-tertiary);
  font-size: 12px;
}

.batch-item-path {
  color: var(--sl-text-primary);
  font-weight: 500;
}

.batch-item-error {
  color: var(--sl-error);
  font-size: 12px;
}
</style>
