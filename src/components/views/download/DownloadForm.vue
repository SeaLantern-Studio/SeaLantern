<script setup lang="ts">
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";

const props = defineProps<{
  url: string;
  savePath: string;
  filename: string;
  threadCount: string;
  isDownloading: boolean;
}>();

const emit = defineEmits<{
  (e: "update:url", value: string): void;
  (e: "update:savePath", value: string): void;
  (e: "update:filename", value: string): void;
  (e: "update:threadCount", value: string): void;
  (e: "checkUrl", event: Event): void;
  (e: "checkFilename", event: Event): void;
  (e: "pickFolder"): void;
}>();
</script>

<template>
  <div class="form-grid">
    <SLInput
      :label="i18n.t('download-file.url')"
      :model-value="url"
      @update:model-value="$emit('update:url', $event)"
      :disabled="isDownloading"
      @input="$emit('checkUrl', $event)"
    />
    <SLInput
      :label="i18n.t('download-file.save_folder')"
      :model-value="savePath"
      @update:model-value="$emit('update:savePath', $event)"
      :disabled="isDownloading"
    >
      <template #suffix>
        <button class="sl-input-action" @click="$emit('pickFolder')" :disabled="isDownloading">
          {{ i18n.t("download-file.browse") }}
        </button>
      </template>
    </SLInput>
    <SLInput
      :label="i18n.t('download-file.filename')"
      :model-value="filename"
      @update:model-value="$emit('update:filename', $event)"
      :disabled="isDownloading"
      @input="$emit('checkFilename', $event)"
    />
    <SLInput
      :label="i18n.t('download-file.thread_count')"
      :model-value="threadCount"
      @update:model-value="$emit('update:threadCount', $event)"
      :disabled="isDownloading"
    />
  </div>
</template>

<style scoped>
.form-grid {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
</style>
