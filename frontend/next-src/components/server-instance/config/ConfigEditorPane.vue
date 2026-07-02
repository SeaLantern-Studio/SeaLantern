<script setup lang="ts">
import { computed } from "vue";
import { SLBadge, SLButton, SLCard } from "@components/common";
import ConfigSourceEditor from "@components/config/ConfigSourceEditor.vue";
import type { DiscoveredServerConfigFile } from "@api/config";
import { i18n } from "@language";
import { RefreshCw, Save } from "@lucide/vue";

interface Props {
  file: DiscoveredServerConfigFile | null;
  modelValue: string;
  loading: boolean;
  saving: boolean;
  hasUnsavedChanges: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  "reload-current": [];
  "save-current": [];
}>();

function getKindText(file: DiscoveredServerConfigFile): string {
  return file.kind.toUpperCase();
}

function getRoleText(file: DiscoveredServerConfigFile): string | null {
  switch (file.known_role) {
    case "server_properties":
      return "server.properties";
    case "startup_primary":
      return "SeaLantern/config.toml";
    case "startup_legacy":
      return "SL.json";
    case "pumpkin":
      return "pumpkin.toml";
    default:
      return null;
  }
}

function getOwnershipText(file: DiscoveredServerConfigFile): string {
  switch (file.ownership) {
    case "service_managed":
      return i18n.t("config.ownership_service_managed");
    case "server_managed":
      return i18n.t("config.ownership_server_managed");
    case "third_party":
      return i18n.t("config.ownership_third_party");
  }
}

const statusText = computed(() => {
  if (props.hasUnsavedChanges) {
    return i18n.t("config.status_unsaved");
  }

  return i18n.t("config.status_loaded");
});
</script>

<template>
  <SLCard variant="outline" class="config-editor-pane">
    <template #header>
      <div class="config-editor-pane__header">
        <div class="config-editor-pane__title-block">
          <span class="config-editor-pane__eyebrow">{{
            i18n.t("config.next_v1.editor_title")
          }}</span>
          <strong class="config-editor-pane__title">{{
            file?.relative_path ?? i18n.t("config.config_files")
          }}</strong>
          <p v-if="file" class="config-editor-pane__subline">
            <span>{{ i18n.t("config.next_v1.absolute_path") }}: {{ file.absolute_path }}</span>
          </p>
          <p v-if="file" class="config-editor-pane__subline">
            <span>{{ i18n.t("config.next_v1.source_label") }}: {{ file.source_label }}</span>
          </p>
        </div>

        <div class="config-editor-pane__header-actions">
          <SLBadge
            :text="statusText"
            :variant="hasUnsavedChanges ? 'warning' : 'success'"
            size="small"
            rounded="medium"
          />
          <SLButton
            variant="secondary"
            size="sm"
            :disabled="!file || loading"
            @click="emit('reload-current')"
          >
            <RefreshCw :size="14" />
            {{ i18n.t("config.reload") }}
          </SLButton>
          <SLButton
            variant="primary"
            size="sm"
            :disabled="!file || !hasUnsavedChanges"
            :loading="saving"
            @click="emit('save-current')"
          >
            <Save :size="14" />
            {{ i18n.t("config.save") }}
          </SLButton>
        </div>
      </div>

      <div v-if="file" class="config-editor-pane__badge-row">
        <SLBadge :text="getKindText(file)" variant="info" size="small" rounded="medium" />
        <SLBadge
          v-if="getRoleText(file)"
          :text="getRoleText(file) || ''"
          variant="primary"
          size="small"
          rounded="medium"
        />
        <SLBadge :text="getOwnershipText(file)" variant="neutral" size="small" rounded="medium" />
      </div>
    </template>

    <div v-if="!file" class="config-editor-pane__empty">
      <span>{{ i18n.t("config.next_v1.editor_empty") }}</span>
    </div>

    <div v-else-if="loading" class="config-editor-pane__empty">
      <span>{{ i18n.t("config.loading") }}</span>
    </div>

    <ConfigSourceEditor
      v-else
      :modelValue="modelValue"
      @update:modelValue="emit('update:modelValue', $event)"
    />
  </SLCard>
</template>

<style scoped>
.config-editor-pane,
.config-editor-pane__header,
.config-editor-pane__title-block {
  display: grid;
  gap: 10px;
}

.config-editor-pane__header {
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: start;
}

.config-editor-pane__title-block {
  min-width: 0;
}

.config-editor-pane__eyebrow,
.config-editor-pane__subline {
  color: var(--sl-text-secondary);
}

.config-editor-pane__eyebrow {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.config-editor-pane__title,
.config-editor-pane__subline {
  word-break: break-word;
}

.config-editor-pane__subline {
  margin: 0;
  font-size: var(--sl-font-size-sm);
}

.config-editor-pane__header-actions,
.config-editor-pane__badge-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.config-editor-pane__header-actions {
  justify-content: flex-end;
}

.config-editor-pane__empty {
  min-height: 220px;
  display: grid;
  place-items: center;
  color: var(--sl-text-secondary);
  text-align: center;
}

@media (max-width: 1100px) {
  .config-editor-pane__header {
    grid-template-columns: 1fr;
  }

  .config-editor-pane__header-actions {
    justify-content: flex-start;
  }
}
</style>
