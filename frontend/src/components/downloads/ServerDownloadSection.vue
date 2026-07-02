<script setup lang="ts">
import { computed } from "vue";
import { FolderOpen, FileText, Cpu } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchPanel from "@src/components/workbench/WorkbenchPanel.vue";
import { i18n } from "@language";

interface Props {
  selectedType: string;
  selectedVersion: string;
  filename: string;
  saveDir: string;
  threadCount: string;
  loadingTypes: boolean;
  loadingVersions: boolean;
  canSubmit: boolean;
  submitting: boolean;
  serverTypeOptions: Array<{ label: string; value: string }>;
  versionOptions: Array<{ label: string; value: string }>;
  downloadUrl: string;
  savePath: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  updateSelectedType: [value: string];
  updateSelectedVersion: [value: string];
  updateFilename: [value: string];
  updateThreadCount: [value: string];
  pickFolder: [];
  submit: [];
}>();

const summaryFacts = computed(() => [
  { label: i18n.t("downloads.next.server.summary.target"), value: props.savePath || i18n.t("downloads.next.server.summary.not_set") },
  { label: i18n.t("downloads.next.server.summary.version"), value: props.selectedVersion || i18n.t("downloads.next.server.summary.not_set") },
  { label: i18n.t("downloads.next.server.summary.threads"), value: props.threadCount || "32" },
]);
</script>

<template>
  <div class="server-download-section">
    <WorkbenchPanel :title="i18n.t('downloads.next.server.form_title')" :description="i18n.t('downloads.next.server.form_description')">
      <div class="server-download-section__form-grid">
        <div class="server-download-section__field">
          <label>{{ i18n.t("downloadServerView.form.type") }}</label>
          <SLSelect :model-value="selectedType" :options="serverTypeOptions" :loading="loadingTypes" :disabled="loadingTypes || submitting" searchable @update:model-value="emit('updateSelectedType', String($event))" />
        </div>
        <div class="server-download-section__field">
          <label>{{ i18n.t("downloadServerView.form.version") }}</label>
          <SLSelect :model-value="selectedVersion" :options="versionOptions" :loading="loadingVersions" :disabled="loadingVersions || !selectedType || submitting" searchable @update:model-value="emit('updateSelectedVersion', String($event))" />
        </div>
        <div class="server-download-section__field server-download-section__field--full">
          <label>{{ i18n.t("downloadServerView.form.saveDir") }}</label>
          <div class="server-download-section__picker">
            <div class="server-download-section__picker-copy">
              <FolderOpen :size="18" class="server-download-section__picker-icon" />
              <div>
                <strong>{{ saveDir || i18n.t("downloadServerView.form.saveDirPlaceholder") }}</strong>
                <span>{{ i18n.t("downloads.next.server.save_dir_hint") }}</span>
              </div>
            </div>
            <SLButton variant="secondary" size="sm" :disabled="submitting" @click="emit('pickFolder')">{{ i18n.t("downloadServerView.actions.pickFolder") }}</SLButton>
          </div>
        </div>
        <div class="server-download-section__field">
          <label>{{ i18n.t("downloadServerView.form.fileName") }}</label>
          <SLInput :model-value="filename" :disabled="submitting" @update:model-value="emit('updateFilename', $event)">
            <template #prefix><FileText :size="16" /></template>
          </SLInput>
        </div>
        <div class="server-download-section__field">
          <label>{{ i18n.t("downloadServerView.form.threadCount") }}</label>
          <SLInput :model-value="threadCount" :disabled="submitting" @update:model-value="emit('updateThreadCount', $event)">
            <template #prefix><Cpu :size="16" /></template>
          </SLInput>
        </div>
      </div>
    </WorkbenchPanel>

    <WorkbenchPanel :title="i18n.t('downloads.next.server.summary_title')" :description="i18n.t('downloads.next.server.summary_description')">
      <WorkbenchFactGrid :items="summaryFacts" />
      <div class="server-download-section__summary-extra">
        <span>{{ i18n.t("downloads.next.server.download_url") }}</span>
        <strong>{{ downloadUrl || i18n.t("downloads.next.server.summary.not_set") }}</strong>
      </div>
      <div class="server-download-section__actions">
        <SLButton variant="primary" size="lg" :loading="submitting" :disabled="!canSubmit" @click="emit('submit')">{{ i18n.t("downloads.next.server.submit") }}</SLButton>
      </div>
    </WorkbenchPanel>
  </div>
</template>

<style scoped>
.server-download-section { min-width: 0; display: grid; gap: 14px; }
.server-download-section__form-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
.server-download-section__field { min-width: 0; display: grid; gap: 6px; }
.server-download-section__field--full { grid-column: 1 / -1; }
.server-download-section__field label { font-size: 0.82rem; color: var(--sl-text-tertiary); font-weight: 500; }
.server-download-section__picker { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 12px 14px; border-radius: 16px; border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent); background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent); }
.server-download-section__picker-copy { min-width: 0; display: flex; align-items: center; gap: 10px; }
.server-download-section__picker-copy div, .server-download-section__summary-extra { min-width: 0; display: grid; gap: 4px; }
.server-download-section__picker-copy strong, .server-download-section__picker-copy span, .server-download-section__summary-extra strong, .server-download-section__summary-extra span { margin: 0; }
.server-download-section__picker-copy strong, .server-download-section__summary-extra strong { color: var(--sl-text-primary); word-break: break-word; }
.server-download-section__picker-copy span, .server-download-section__summary-extra span { color: var(--sl-text-secondary); font-size: 0.8rem; line-height: 1.45; }
.server-download-section__picker-icon { flex: none; color: var(--sl-primary); }
.server-download-section__actions { display: flex; justify-content: flex-start; }
@media (max-width: 760px) {
  .server-download-section__form-grid { grid-template-columns: 1fr; }
  .server-download-section__picker { flex-direction: column; align-items: flex-start; }
  .server-download-section__actions { justify-content: stretch; }
}
</style>
