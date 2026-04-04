<script setup lang="ts">
import { FolderOpen, FileText, Cpu } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";

interface Props {
  serverTypeOptions: Array<{ label: string; value: string }>;
  versionOptions: Array<{ label: string; value: string }>;
  selectedType: string;
  selectedVersion: string;
  filename: string;
  saveDir: string;
  threadCount: string;
  loadingTypes: boolean;
  loadingVersions: boolean;
  isDownloading: boolean;
  savePathPreview: string;
  infoUrl: string | undefined;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:selectedType", value: string): void;
  (e: "update:selectedVersion", value: string): void;
  (e: "update:filename", value: string): void;
  (e: "update:saveDir", value: string): void;
  (e: "update:threadCount", value: string): void;
  (e: "pickFolder"): void;
}>();

function handlePickFolder() {
  if (props.isDownloading) return;
  emit("pickFolder");
}
</script>

<template>
  <div class="download-form">
    <div class="form-grid">
      <div class="field">
        <div class="label-row">
          <label>{{ i18n.t("downloadServerView.form.type") }}</label>
        </div>
        <SLSelect
          :model-value="selectedType"
          :options="serverTypeOptions"
          :placeholder="i18n.t('downloadServerView.form.typePlaceholder')"
          :disabled="loadingTypes || isDownloading"
          :loading="loadingTypes"
          searchable
          maxHeight="240px"
          @update:modelValue="emit('update:selectedType', $event)"
        />
      </div>

      <div class="field">
        <div class="label-row">
          <label>{{ i18n.t("downloadServerView.form.version") }}</label>
        </div>
        <SLSelect
          :model-value="selectedVersion"
          :options="versionOptions"
          :placeholder="i18n.t('downloadServerView.form.versionPlaceholder')"
          :disabled="loadingVersions || !selectedType || isDownloading"
          :loading="loadingVersions"
          searchable
          maxHeight="240px"
          @update:modelValue="emit('update:selectedVersion', $event)"
        />
      </div>

      <div class="field field-full">
        <label>{{ i18n.t("downloadServerView.form.fileName") }}</label>
        <SLInput
          :model-value="filename"
          type="text"
          :placeholder="i18n.t('downloadServerView.form.fileNamePlaceholder')"
          :disabled="isDownloading"
          @update:modelValue="emit('update:filename', $event)"
        >
          <template #prefix>
            <FileText :size="16" class="input-icon" />
          </template>
        </SLInput>
      </div>

      <div class="field field-full">
        <label>{{ i18n.t("downloadServerView.form.saveDir") }}</label>
        <div
          class="path-picker"
          :class="{ disabled: isDownloading }"
          role="button"
          tabindex="0"
          @click="handlePickFolder"
          @keydown.enter.prevent="handlePickFolder"
        >
          <FolderOpen :size="18" class="path-icon" />
          <div class="path-content">
            <div class="path-title">{{ i18n.t("downloadServerView.form.saveDir") }}</div>
            <div class="path-value" :class="{ empty: !saveDir }">
              {{
                saveDir.replace(/\\/g, "/") || i18n.t("downloadServerView.form.saveDirPlaceholder")
              }}
            </div>
          </div>
          <SLButton
            variant="secondary"
            size="sm"
            :disabled="isDownloading"
            @click.stop="handlePickFolder"
          >
            {{ i18n.t("downloadServerView.actions.pickFolder") }}
          </SLButton>
        </div>
      </div>

      <div class="field">
        <label>{{ i18n.t("downloadServerView.form.threadCount") }}</label>
        <SLInput
          :model-value="threadCount"
          type="text"
          :placeholder="i18n.t('downloadServerView.form.threadCountPlaceholder')"
          :disabled="isDownloading"
          @update:modelValue="emit('update:threadCount', $event)"
        >
          <template #prefix>
            <Cpu :size="16" class="input-icon" />
          </template>
        </SLInput>
      </div>

      <div class="field">
        <p v-if="savePathPreview" class="preview">
          {{ i18n.t("downloadServerView.preview.saveTo") }}{{ savePathPreview }}
        </p>
        <p v-if="infoUrl" class="preview">
          {{ i18n.t("downloadServerView.preview.url") }}{{ infoUrl }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.download-form {
  display: grid;
  gap: var(--sl-space-md);
}

.form-grid {
  display: grid;
  gap: var(--sl-space-md);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-full {
  grid-column: 1 / -1;
}

.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
}

.field label {
  font-size: 0.82rem;
  color: var(--sl-text-tertiary);
  font-weight: 500;
}

.loading-text {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

.field input,
.field select {
  width: 100%;
  height: 40px;
  padding: 0 10px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.field input:focus,
.field select:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.14);
}

.field input[readonly] {
  opacity: 0.9;
}

.input-icon {
  color: var(--sl-text-tertiary);
  pointer-events: none;
}

.path-picker {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  padding: 10px 12px;
  background: var(--sl-surface);
  transition: all 0.18s ease;
  cursor: pointer;
}

.path-picker:hover {
  border-color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.path-picker.disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.path-icon {
  color: var(--sl-primary);
  flex-shrink: 0;
}

.path-content {
  min-width: 0;
  flex: 1;
}

.path-title {
  font-size: 0.72rem;
  color: var(--sl-text-tertiary);
  margin-bottom: 2px;
}

.path-value {
  font-size: 0.86rem;
  color: var(--sl-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.path-value.empty {
  color: var(--sl-text-tertiary);
}

.preview {
  margin: 0;
  font-size: 12px;
  color: var(--sl-text-tertiary);
  word-break: break-all;
}

@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr;
  }
}
</style>
