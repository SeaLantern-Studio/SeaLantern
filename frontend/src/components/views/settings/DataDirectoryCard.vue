<script setup lang="ts">
import { computed } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";
import type { DataDirStatus } from "@api/settings";

const props = defineProps<{
  status: DataDirStatus | null;
  pathDraft: string;
  busy?: boolean;
  error?: string | null;
  infoMessage?: string | null;
}>();

const emit = defineEmits<{
  (e: "update:pathDraft", value: string): void;
  (e: "browse"): void;
  (e: "refresh"): void;
  (e: "change", value: string): void;
}>();

const resolutionSourceLabel = computed(() => {
  const source = props.status?.resolution_source || "default";
  return i18n.t(`settings.data_dir_source_${source}`);
});
</script>

<template>
  <SLCard :title="i18n.t('settings.data_dir_title')" :subtitle="i18n.t('settings.data_dir_desc')">
    <div class="sl-settings-group">
      <div class="sl-setting-row full-width">
        <div class="sl-setting-info">
          <span class="sl-setting-label">{{ i18n.t("settings.data_dir_current") }}</span>
          <span class="sl-setting-desc">{{ resolutionSourceLabel }}</span>
        </div>
        <SLInput
          :model-value="pathDraft"
          :placeholder="status?.recommended_data_dir || ''"
          @update:model-value="(value) => emit('update:pathDraft', value)"
        >
          <template #suffix>
            <button type="button" class="sl-input-action" @click="emit('browse')">
              {{ i18n.t("settings.browse") }}
            </button>
          </template>
        </SLInput>
      </div>

      <div v-if="status" class="data-dir-meta">
        <p>{{ i18n.t("settings.data_dir_locator") }}: {{ status.locator_path }}</p>
        <p>{{ i18n.t("settings.data_dir_recommended") }}: {{ status.recommended_data_dir }}</p>
      </div>

      <div v-if="error" class="data-dir-error">{{ error }}</div>
      <div v-else-if="infoMessage" class="data-dir-info">{{ infoMessage }}</div>

      <div class="data-dir-actions">
        <SLButton variant="secondary" :disabled="busy" @click="emit('refresh')">
          {{ i18n.t("common.refresh") }}
        </SLButton>
        <SLButton variant="primary" :loading="busy" @click="emit('change', pathDraft)">
          {{ i18n.t("settings.data_dir_change") }}
        </SLButton>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.sl-setting-row.full-width {
  flex-direction: column;
  align-items: stretch;
}

.data-dir-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.data-dir-meta p {
  margin: 0;
  word-break: break-all;
}

.data-dir-error,
.data-dir-info {
  padding: 10px 12px;
  border-radius: var(--sl-radius-md);
  font-size: var(--sl-font-size-sm);
}

.data-dir-error {
  border: 1px solid var(--sl-error);
  background: var(--sl-error-bg);
  color: var(--sl-error);
}

.data-dir-info {
  border: 1px solid color-mix(in srgb, var(--sl-primary) 28%, transparent);
  background: color-mix(in srgb, var(--sl-primary) 10%, transparent);
  color: var(--sl-text-primary);
}

.data-dir-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
}
</style>
