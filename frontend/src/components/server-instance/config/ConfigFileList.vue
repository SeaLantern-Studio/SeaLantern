<script setup lang="ts">
import { computed } from "vue";
import { SLBadge, SLButton, SLCard } from "@components/common";
import type { DiscoveredServerConfigFile } from "@api/config";
import { i18n } from "@language";
import { FolderSync } from "@lucide/vue";

interface Props {
  files: DiscoveredServerConfigFile[];
  selectedLocator: string;
  loading: boolean;
  refreshing: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "select-file": [locator: string];
  refresh: [];
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

function getSourceKindText(file: DiscoveredServerConfigFile): string {
  switch (file.source_kind) {
    case "server_root":
      return i18n.t("config.source_group_server_root");
    case "manual_root":
      return i18n.t("config.source_group_manual_root");
    case "manual_file":
      return i18n.t("config.source_group_manual_file");
  }
}

const emptyText = computed(() => {
  if (props.loading) {
    return i18n.t("config.loading");
  }

  return i18n.t("config.no_config");
});
</script>

<template>
  <SLCard variant="outline" class="config-file-list" :title="i18n.t('config.config_files')">
    <template #actions>
      <SLButton
        variant="secondary"
        size="sm"
        :loading="refreshing"
        :disabled="loading"
        @click="emit('refresh')"
      >
        <FolderSync :size="14" />
        {{ i18n.t("config.refresh_list") }}
      </SLButton>
    </template>

    <div v-if="files.length === 0" class="config-file-list__empty">
      <span>{{ emptyText }}</span>
    </div>

    <div v-else class="config-file-list__items">
      <button
        v-for="file in files"
        :key="file.locator"
        type="button"
        class="config-file-list__item"
        :class="{ 'config-file-list__item--active': file.locator === selectedLocator }"
        @click="emit('select-file', file.locator)"
      >
        <div class="config-file-list__item-main">
          <strong class="config-file-list__file-name">{{ file.file_name }}</strong>
          <div class="config-file-list__badges">
            <SLBadge :text="getKindText(file)" variant="info" size="small" rounded="medium" />
            <SLBadge
              v-if="getRoleText(file)"
              :text="getRoleText(file) || ''"
              variant="primary"
              size="small"
              rounded="medium"
            />
            <SLBadge
              :text="getOwnershipText(file)"
              variant="neutral"
              size="small"
              rounded="medium"
            />
          </div>
        </div>

        <div class="config-file-list__path">{{ file.relative_path }}</div>

        <div class="config-file-list__meta">
          <span>{{ getSourceKindText(file) }}</span>
          <span class="config-file-list__meta-sep">·</span>
          <span class="config-file-list__source-label" :title="file.source_label">{{
            file.source_label
          }}</span>
        </div>
      </button>
    </div>
  </SLCard>
</template>

<style scoped>
.config-file-list,
.config-file-list__items {
  display: grid;
  gap: 12px;
}

.config-file-list {
  min-height: 0;
}

.config-file-list__empty {
  min-height: 160px;
  display: grid;
  place-items: center;
  color: var(--sl-text-secondary);
  text-align: center;
}

.config-file-list__items {
  max-height: min(72vh, 920px);
  overflow: auto;
  padding-right: 4px;
}

.config-file-list__item {
  display: grid;
  gap: 8px;
  width: 100%;
  padding: 12px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-surface);
  color: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
}

.config-file-list__item:hover {
  border-color: var(--sl-primary-light);
  transform: translateY(-1px);
}

.config-file-list__item--active {
  border-color: var(--sl-primary);
  background: color-mix(in srgb, var(--sl-primary-bg) 22%, var(--sl-surface));
}

.config-file-list__item-main,
.config-file-list__badges {
  display: flex;
  gap: 8px;
}

.config-file-list__item-main {
  align-items: flex-start;
  justify-content: space-between;
}

.config-file-list__badges {
  flex-wrap: wrap;
  justify-content: flex-end;
}

.config-file-list__file-name,
.config-file-list__path {
  word-break: break-word;
}

.config-file-list__file-name {
  color: var(--sl-text-primary);
}

.config-file-list__path,
.config-file-list__meta {
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-sm);
}

.config-file-list__meta {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.config-file-list__source-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.config-file-list__meta-sep {
  color: var(--sl-text-tertiary);
}

@media (max-width: 960px) {
  .config-file-list__item-main {
    flex-direction: column;
  }

  .config-file-list__badges {
    justify-content: flex-start;
  }
}
</style>
