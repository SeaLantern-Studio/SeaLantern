<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import { formatPluginInstallIssue } from "@components/views/plugins/pluginInstallErrorMessage";
import { i18n } from "@language";
import type { BatchInstallError, BatchInstallResult, PluginInstallResult } from "@type/plugin";

defineProps<{
  visible: boolean;
  batchInstallResult: BatchInstallResult | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

function getErrorMessage(item: BatchInstallError): string {
  return formatPluginInstallIssue(item.issue) || item.error;
}

function getSuccessNoticeMessages(item: PluginInstallResult): string[] {
  return (item.install_notices || [])
    .map((notice) => formatPluginInstallIssue(notice))
    .filter((message): message is string => Boolean(message));
}
</script>

<template>
  <SLModal :visible="visible" :title="i18n.t('plugins.batch_result_title')" @close="emit('close')">
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
            <ul v-if="getSuccessNoticeMessages(item).length > 0" class="batch-notice-list">
              <li v-for="message in getSuccessNoticeMessages(item)" :key="message" class="batch-item-notice">
                {{ message }}
              </li>
            </ul>
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
            <span class="batch-item-error">{{ getErrorMessage(item) }}</span>
          </li>
        </ul>
      </div>
    </div>
    <template #footer>
      <SLButton variant="primary" size="sm" @click="emit('close')">{{
        i18n.t("plugins.ok")
      }}</SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
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
  flex-direction: column;
  align-items: flex-start;
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

.batch-notice-list {
  margin: 4px 0 0 0;
  padding-left: 16px;
}

.batch-item-notice {
  color: var(--sl-text-secondary);
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
