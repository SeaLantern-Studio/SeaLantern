<script setup lang="ts">
import { FolderOpen, Link, FileText, Cpu } from "lucide-vue-next";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

interface Props {
  url: string;
  savePath: string;
  filename: string;
  threadCount: string;
  isDownloading: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:url", value: string): void;
  (e: "update:savePath", value: string): void;
  (e: "update:filename", value: string): void;
  (e: "update:threadCount", value: string): void;
  (e: "checkUrl"): void;
  (e: "checkFilename"): void;
  (e: "pickFolder"): void;
  (e: "checkThreadCount"): void;
}>();

function handlePickFolder() {
  if (props.isDownloading) return;
  emit("pickFolder");
}
</script>

<template>
  <div class="download-form">
    <div class="field">
      <label>{{ i18n.t("download-file.url") }}</label>
      <div class="input-wrap">
        <Link :size="16" class="input-icon" />
        <input
          :value="url"
          type="text"
          :placeholder="i18n.t('download-file.url_placeholder')"
          :disabled="isDownloading"
          @input="emit('update:url', ($event.target as HTMLInputElement).value)"
          @blur="emit('checkUrl')"
        />
      </div>
    </div>

    <div class="field">
      <label>{{ i18n.t("download-file.save_path") }}</label>
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
          <div class="path-title">{{ i18n.t("download-file.save_path") }}</div>
          <div class="path-value" :class="{ empty: !savePath }">
            {{ savePath || i18n.t("download-file.select_folder") }}
          </div>
        </div>
        <SLButton
          variant="secondary"
          size="sm"
          :disabled="isDownloading"
          @click.stop="handlePickFolder"
        >
          {{ i18n.t("download-file.pick_folder") }}
        </SLButton>
      </div>
    </div>

    <div class="field">
      <label>{{ i18n.t("download-file.filename") }}</label>
      <div class="input-wrap">
        <FileText :size="16" class="input-icon" />
        <input
          :value="filename"
          type="text"
          :placeholder="i18n.t('download-file.filename_placeholder')"
          :disabled="isDownloading"
          @input="emit('update:filename', ($event.target as HTMLInputElement).value)"
          @blur="emit('checkFilename')"
        />
      </div>
    </div>

    <div class="field">
      <label>{{ i18n.t("download-file.thread_count") }}</label>
      <div class="input-wrap">
        <Cpu :size="16" class="input-icon" />
        <input
          :value="threadCount"
          type="text"
          placeholder="32"
          :disabled="isDownloading"
          @input="emit('update:threadCount', ($event.target as HTMLInputElement).value)"
          @blur="emit('checkThreadCount')"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.download-form {
  display: grid;
  gap: var(--sl-space-md);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label {
  font-size: 0.82rem;
  color: var(--sl-text-tertiary);
  font-weight: 500;
}

.input-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.input-icon {
  position: absolute;
  left: 10px;
  color: var(--sl-text-tertiary);
  pointer-events: none;
}

.input-wrap input {
  width: 100%;
  height: 40px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  padding: 0 12px 0 34px;
  outline: none;
}

.input-wrap input:focus {
  border-color: var(--sl-primary);
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
</style>
