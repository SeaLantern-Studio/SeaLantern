<script setup lang="ts">
import { computed } from "vue";
import { FolderOpen, Link, FileText, Cpu } from "@lucide/vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchPanel from "@src/components/workbench/WorkbenchPanel.vue";
import { i18n } from "@language";

interface Props {
  url: string;
  filename: string;
  saveDir: string;
  threadCount: string;
  savePath: string;
  canSubmit: boolean;
  submitting: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  updateUrl: [value: string];
  updateFilename: [value: string];
  updateThreadCount: [value: string];
  fillFilename: [];
  pickFolder: [];
  submit: [];
}>();

const summaryFacts = computed(() => [
  {
    label: i18n.t("downloads.next.file.summary.target"),
    value: props.savePath || i18n.t("downloads.next.file.summary.not_set"),
  },
  {
    label: i18n.t("downloads.next.file.summary.filename"),
    value: props.filename || i18n.t("downloads.next.file.summary.not_set"),
  },
  { label: i18n.t("downloads.next.file.summary.threads"), value: props.threadCount || "32" },
]);
</script>

<template>
  <div class="file-download-section">
    <WorkbenchPanel
      :title="i18n.t('downloads.next.file.form_title')"
      :description="i18n.t('downloads.next.file.form_description')"
    >
      <div class="file-download-section__form-grid">
        <div class="file-download-section__field file-download-section__field--full">
          <label>{{ i18n.t("download-file.url") }}</label>
          <SLInput
            :model-value="url"
            :disabled="submitting"
            :placeholder="i18n.t('download-file.url_placeholder')"
            @update:model-value="emit('updateUrl', $event)"
            @blur="emit('fillFilename')"
          >
            <template #prefix><Link :size="16" /></template>
          </SLInput>
        </div>
        <div class="file-download-section__field file-download-section__field--full">
          <label>{{ i18n.t("download-file.save_path") }}</label>
          <div class="file-download-section__picker">
            <div class="file-download-section__picker-copy">
              <FolderOpen :size="18" class="file-download-section__picker-icon" />
              <div>
                <strong>{{ saveDir || i18n.t("download-file.select_folder") }}</strong>
                <span>{{ i18n.t("downloads.next.file.save_dir_hint") }}</span>
              </div>
            </div>
            <SLButton
              variant="secondary"
              size="sm"
              :disabled="submitting"
              @click="emit('pickFolder')"
              >{{ i18n.t("download-file.pick_folder") }}</SLButton
            >
          </div>
        </div>
        <div class="file-download-section__field">
          <label>{{ i18n.t("download-file.filename") }}</label>
          <SLInput
            :model-value="filename"
            :disabled="submitting"
            @update:model-value="emit('updateFilename', $event)"
          >
            <template #prefix><FileText :size="16" /></template>
          </SLInput>
        </div>
        <div class="file-download-section__field">
          <label>{{ i18n.t("download-file.thread_count") }}</label>
          <SLInput
            :model-value="threadCount"
            :disabled="submitting"
            @update:model-value="emit('updateThreadCount', $event)"
          >
            <template #prefix><Cpu :size="16" /></template>
          </SLInput>
        </div>
      </div>
    </WorkbenchPanel>

    <WorkbenchPanel
      :title="i18n.t('downloads.next.file.summary_title')"
      :description="i18n.t('downloads.next.file.summary_description')"
    >
      <WorkbenchFactGrid :items="summaryFacts" />
      <div class="file-download-section__summary-extra">
        <span>{{ i18n.t("downloads.next.file.url_label") }}</span>
        <strong>{{ url || i18n.t("downloads.next.file.summary.not_set") }}</strong>
      </div>
      <div class="file-download-section__actions">
        <SLButton
          variant="primary"
          size="lg"
          :loading="submitting"
          :disabled="!canSubmit"
          @click="emit('submit')"
          >{{ i18n.t("downloads.next.file.submit") }}</SLButton
        >
      </div>
    </WorkbenchPanel>
  </div>
</template>

<style scoped>
.file-download-section {
  min-width: 0;
  display: grid;
  gap: 14px;
}
.file-download-section__form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}
.file-download-section__field {
  min-width: 0;
  display: grid;
  gap: 6px;
}
.file-download-section__field--full {
  grid-column: 1 / -1;
}
.file-download-section__field label {
  font-size: 0.82rem;
  color: var(--sl-text-tertiary);
  font-weight: 500;
}
.file-download-section__picker {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}
.file-download-section__picker-copy {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}
.file-download-section__picker-copy div,
.file-download-section__summary-extra {
  min-width: 0;
  display: grid;
  gap: 4px;
}
.file-download-section__picker-copy strong,
.file-download-section__picker-copy span,
.file-download-section__summary-extra strong,
.file-download-section__summary-extra span {
  margin: 0;
}
.file-download-section__picker-copy strong,
.file-download-section__summary-extra strong {
  color: var(--sl-text-primary);
  word-break: break-word;
}
.file-download-section__picker-copy span,
.file-download-section__summary-extra span {
  color: var(--sl-text-secondary);
  font-size: 0.8rem;
  line-height: 1.45;
}
.file-download-section__picker-icon {
  flex: none;
  color: var(--sl-primary);
}
.file-download-section__actions {
  display: flex;
  justify-content: flex-start;
}
@media (max-width: 760px) {
  .file-download-section__form-grid {
    grid-template-columns: 1fr;
  }
  .file-download-section__picker {
    flex-direction: column;
    align-items: flex-start;
  }
  .file-download-section__actions {
    justify-content: stretch;
  }
}
</style>
