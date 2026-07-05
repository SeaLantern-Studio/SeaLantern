<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import ConfigEditorPane from "@src/components/server-instance/config/ConfigEditorPane.vue";
import ConfigFileList from "@src/components/server-instance/config/ConfigFileList.vue";
import ConfigPreviewPane from "@src/components/server-instance/config/ConfigPreviewPane.vue";
import { useServerInstanceConfigPage } from "./useServerInstanceConfigPage";

const page = useServerInstanceConfigPage();

const discardCancelText = computed(() => i18n.t("config.next_v1.discard_cancel"));
const handleDraftSourceUpdate = (value: string) => {
  page.updateDraftSource(value);
};
const files = computed(() => page.files.value);
const selectedLocator = computed(() => page.selectedLocator.value);
const loadingFiles = computed(() => page.loadingFiles.value);
const refreshingFiles = computed(() => page.refreshingFiles.value);
const currentFile = computed(() => page.currentFile.value);
const draftSource = computed(() => page.draftSource.value);
const loadingCurrentFile = computed(() => page.loadingCurrentFile.value);
const saving = computed(() => page.saving.value);
const hasUnsavedChanges = computed(() => page.hasUnsavedChanges.value);
const previewSource = computed(() => page.previewSource.value);
const previewState = computed(() => page.previewState.value);
const discardDialogState = computed(() => page.discardDialogState.value);
const errorMessage = computed(() => page.errorMessage.value);
const successMessage = computed(() => page.successMessage.value);
</script>

<template>
  <div class="server-instance-config-page">
    <section v-if="errorMessage" class="server-instance-config-page__error" role="alert">
      <strong>{{ i18n.t("servers.next.error_title") }}</strong>
      <span>{{ errorMessage }}</span>
    </section>

    <section v-if="successMessage" class="server-instance-config-page__success">
      <span>{{ successMessage }}</span>
    </section>

    <div class="server-instance-config-page__grid">
      <ConfigFileList
        :files="files"
        :selected-locator="selectedLocator"
        :loading="loadingFiles"
        :refreshing="refreshingFiles"
        @select-file="page.requestSwitchFile"
        @refresh="page.refreshFiles"
      />

      <ConfigEditorPane
        :file="currentFile"
        :model-value="draftSource"
        :loading="loadingCurrentFile"
        :saving="saving"
        :has-unsaved-changes="hasUnsavedChanges"
        @update:model-value="handleDraftSourceUpdate"
        @reload-current="page.reloadCurrentFile"
        @save-current="page.saveCurrentFile"
      />

      <ConfigPreviewPane
        :file="currentFile"
        :source-value="previewSource"
        :preview-state="previewState"
      />
    </div>

    <SLConfirmDialog
      :visible="discardDialogState.visible"
      :title="discardDialogState.title"
      :message="discardDialogState.message"
      :confirmText="discardDialogState.confirmText"
      :cancelText="discardCancelText"
      confirmVariant="danger"
      @confirm="page.confirmDiscardAndContinue"
      @close="page.cancelDiscardAndContinue"
    />
  </div>
</template>

<style scoped>
.server-instance-config-page {
  display: grid;
  gap: 16px;
}

.server-instance-config-page__grid {
  display: grid;
  grid-template-columns: minmax(260px, 300px) minmax(0, 1.35fr) minmax(320px, 1fr);
  gap: 16px;
  align-items: start;
}

.server-instance-config-page__error,
.server-instance-config-page__success {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
}

.server-instance-config-page__error {
  border: 1px solid rgba(239, 68, 68, 0.24);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.server-instance-config-page__success {
  border: 1px solid rgba(34, 197, 94, 0.24);
  background: rgba(34, 197, 94, 0.1);
  color: rgb(var(--sl-success));
}

@media (max-width: 1380px) {
  .server-instance-config-page__grid {
    grid-template-columns: minmax(240px, 300px) minmax(0, 1fr);
  }

  .server-instance-config-page__grid > :last-child {
    grid-column: 1 / -1;
  }
}

@media (max-width: 960px) {
  .server-instance-config-page__grid {
    grid-template-columns: 1fr;
  }

  .server-instance-config-page__grid > :last-child {
    grid-column: auto;
  }
}
</style>
