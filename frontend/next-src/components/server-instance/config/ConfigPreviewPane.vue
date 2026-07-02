<script setup lang="ts">
import { computed } from "vue";
import { SLCard } from "@components/common";
import ConfigSourceDiffView from "@src/features/config-editor/components/ConfigSourceDiffView.vue";
import type { DiscoveredServerConfigFile } from "@api/config";
import { i18n } from "@language";
import type { ServerInstanceConfigPreviewState } from "@next-src/pages/server-instance/config/useServerInstanceConfigPage";

interface Props {
  file: DiscoveredServerConfigFile | null;
  sourceValue: string;
  previewState: ServerInstanceConfigPreviewState;
}

const props = defineProps<Props>();

const hasDiff = computed(() => {
  return props.previewState.kind === "ready" && props.previewState.modified !== props.sourceValue;
});
</script>

<template>
  <SLCard
    variant="outline"
    class="config-preview-pane"
    :title="i18n.t('config.next_v1.preview_title')"
    :subtitle="i18n.t('config.next_v1.preview_note')"
  >
    <div v-if="!file" class="config-preview-pane__empty">
      <span>{{ i18n.t("config.next_v1.preview_empty") }}</span>
    </div>

    <div v-else-if="previewState.kind === 'unsupported'" class="config-preview-pane__empty">
      <span>{{ previewState.message }}</span>
    </div>

    <div v-else-if="previewState.kind === 'empty'" class="config-preview-pane__empty">
      <span>{{ previewState.message }}</span>
    </div>

    <div v-else-if="previewState.kind === 'loading'" class="config-preview-pane__empty">
      <span>{{ previewState.message }}</span>
      <small>{{ previewState.note }}</small>
    </div>

    <div v-else-if="previewState.kind === 'error'" class="config-preview-pane__error">
      <strong>{{ i18n.t("config.next_v1.preview_error") }}</strong>
      <span>{{ previewState.message || i18n.t("config.next_v1.preview_parse_failed") }}</span>
      <small v-if="previewState.note">{{ previewState.note }}</small>
    </div>

    <div v-else-if="!hasDiff" class="config-preview-pane__empty">
      <span>{{ i18n.t("config.next_v1.preview_same") }}</span>
      <small>{{ previewState.note }}</small>
    </div>

    <div v-else class="config-preview-pane__diff-wrap">
      <div class="config-preview-pane__labels">
        <span>{{ i18n.t("config.next_v1.current_source") }}</span>
        <span>{{ i18n.t("config.next_v1.preview_result") }}</span>
      </div>
      <ConfigSourceDiffView :original="sourceValue" :modified="previewState.modified" />
      <small class="config-preview-pane__note">{{ previewState.note }}</small>
    </div>
  </SLCard>
</template>

<style scoped>
.config-preview-pane,
.config-preview-pane__diff-wrap {
  display: grid;
  gap: 12px;
}

.config-preview-pane__empty,
.config-preview-pane__error {
  min-height: 220px;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 8px;
  text-align: center;
}

.config-preview-pane__empty {
  color: var(--sl-text-secondary);
}

.config-preview-pane__error {
  color: var(--sl-error);
}

.config-preview-pane__labels {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.config-preview-pane__note {
  color: var(--sl-text-secondary);
}

@media (max-width: 767px) {
  .config-preview-pane__labels {
    flex-direction: column;
  }
}
</style>
