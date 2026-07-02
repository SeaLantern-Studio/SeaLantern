<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import PluginBatchResultModal from "@components/plugin/installer/PluginBatchResultModal.vue";
import PluginDeleteConfirmModal from "@components/views/plugins/PluginDeleteConfirmModal.vue";
import PluginDependencyPromptModal from "@components/plugin/installer/PluginDependencyPromptModal.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";
import type { BatchInstallResult, MissingDependency } from "@type/plugin";

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
  <PluginDeleteConfirmModal
    :visible="showSingleDeleteDialog"
    :title="i18n.t('plugins.confirm_delete')"
    :message="i18n.t('plugins.confirm_delete_message', { name: singleDeletePluginName })"
    @close="emit('close-single-delete')"
    @confirm="emit('confirm-single-delete', $event)"
  />

  <PluginDeleteConfirmModal
    :visible="showBatchDeleteDialog"
    :title="i18n.t('plugins.confirm_batch_delete')"
    :message="i18n.t('plugins.confirm_batch_delete_message', { count: selectedCount })"
    @close="emit('close-batch-delete')"
    @confirm="emit('confirm-batch-delete', $event)"
  />

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

  <PluginDependencyPromptModal
    :visible="showDependencyModal"
    :installed-plugin-name="installedPluginName"
    :missing-dependencies="missingDependencies"
    :get-dep-display-name="getDepDisplayName"
    @close="emit('close-dependency')"
    @go-market="emit('go-market')"
  />

  <PluginBatchResultModal
    :visible="showBatchResultModal"
    :batch-install-result="batchInstallResult"
    @close="emit('close-batch-result')"
  />
</template>

<style scoped>
.dialog-message {
  margin: 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-line;
}
</style>
