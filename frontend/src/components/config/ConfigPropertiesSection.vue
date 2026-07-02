<script setup lang="ts">
import { computed, defineAsyncComponent } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import ConfigCategories from "@components/config/ConfigCategories.vue";
import ConfigComparePanel from "@components/config/ConfigComparePanel.vue";
import ConfigPropertyEditorControl from "@components/config/ConfigPropertyEditorControl.vue";
import type {
  ConfigEntry as ConfigEntryType,
  DiscoveredServerConfigFile,
  ServerConfigJsonMode,
  ServerConfigOwnership,
  ServerConfigSearchHit,
  ServerConfigSearchMode,
  ServerConfigSearchScope,
  ServerConfigSourceKind,
} from "@api/config";
import { i18n } from "@language";
import type { ComparePanelRow } from "@views/config/useConfigCompare";
import { FileDiff, Plus, RefreshCw, Save, X } from "@lucide/vue";

const ConfigSourceEditor = defineAsyncComponent(
  () => import("@components/config/ConfigSourceEditor.vue"),
);

interface Option {
  label: string;
  value: string | number;
}

interface UpdateValuePayload {
  key: string;
  value: string | boolean | number;
}

interface Props {
  editorMode: "visual" | "source";
  isPropertiesFile: boolean;
  loading: boolean;
  compareLoading: boolean;
  compareMode: boolean;
  compareSupported: boolean;
  hasCompareTargets: boolean;
  compareTargetServerId: string;
  compareServerOptions: Option[];
  compareDifferenceBadgeText: string;
  comparePanelRows: ComparePanelRow[];
  sourceServerName: string;
  targetServerName: string;
  hasDiscoveredConfigFiles: boolean;
  configFiles: DiscoveredServerConfigFile[];
  selectedConfigLocator: string;
  manualImportDirs: string[];
  manualImportFiles: string[];
  configJsonMode: ServerConfigJsonMode;
  configSearchQuery: string;
  configSearchMode: ServerConfigSearchMode;
  configSearchScope: ServerConfigSearchScope;
  configSearchResults: ServerConfigSearchHit[];
  configSearchLoading: boolean;
  configSearchError: string | null;
  categories: string[];
  activeCategory: string;
  searchQuery: string;
  filteredEntries: ConfigEntryType[];
  translatedDescriptionByKey: Record<string, string>;
  editValues: Record<string, string>;
  numericFieldErrors: Record<string, string>;
  gamemodeOptions: Option[];
  difficultyOptions: Option[];
  sourceDraftText: string;
  compareTargetSourceDraftText: string;
  sourceParseError: string | null;
  hasUnsavedChanges: boolean;
  saveStatusText: string;
  saving: boolean;
  reloadCurrentTooltipText: string;
  reloadCompareTooltipText: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  updateCategory: [category: string];
  updateSelectedConfigFile: [value: string | number];
  importConfigDirectory: [];
  importConfigFile: [];
  removeConfigImportDirectory: [path: string];
  removeConfigImportFile: [path: string];
  updateConfigSearchQuery: [query: string];
  updateConfigSearchMode: [value: string | number];
  updateConfigSearchScope: [value: string | number];
  updateConfigJsonMode: [value: string | number];
  updateCompareMode: [value: boolean];
  updateSearch: [query: string];
  updateSourceDraft: [value: string];
  updateCompareTargetSourceDraft: [value: string];
  updateValue: [payload: UpdateValuePayload];
  updateCompareTargetValue: [payload: UpdateValuePayload];
  addSourceValue: [payload: UpdateValuePayload];
  addTargetValue: [payload: UpdateValuePayload];
  updateCompareTargetServer: [value: string | number];
  reloadCurrent: [];
  reloadCompare: [];
  saveProperties: [];
}>();

const searchHitByLocator = computed(
  () => new Map(props.configSearchResults.map((hit) => [hit.locator, hit] as const)),
);

const configFileOptions = computed(() =>
  props.configFiles.map((file) => {
    const hit = searchHitByLocator.value.get(file.locator);
    const subLabel = hit?.content_match
      ? `${file.relative_path} · L${hit.content_match.line_number}: ${hit.content_match.line_text}`
      : `${file.relative_path} · ${file.source_label}`;

    return {
      label: file.file_name,
      value: file.locator,
      subLabel,
    };
  }),
);

const configSearchModeOptions = computed<Option[]>(() => [
  { label: i18n.t("config.search_mode_keyword"), value: "keyword" },
  { label: i18n.t("config.search_mode_regex"), value: "regex" },
  { label: i18n.t("config.search_mode_similarity"), value: "similarity" },
]);

const configSearchScopeOptions = computed<Option[]>(() => [
  { label: i18n.t("config.search_scope_path"), value: "path" },
  { label: i18n.t("config.search_scope_content"), value: "content" },
  { label: i18n.t("config.search_scope_all"), value: "all" },
]);

const configJsonModeOptions = computed<Option[]>(() => [
  { label: i18n.t("config.json_mode_filtered"), value: "filtered" },
  { label: i18n.t("config.json_mode_all"), value: "all" },
  { label: i18n.t("config.json_mode_disabled"), value: "disabled" },
]);

function getSourceKindLabel(kind: ServerConfigSourceKind) {
  switch (kind) {
    case "server_root":
      return i18n.t("config.source_group_server_root");
    case "manual_root":
      return i18n.t("config.source_group_manual_root");
    case "manual_file":
      return i18n.t("config.source_group_manual_file");
  }
}

function getOwnershipLabel(ownership: ServerConfigOwnership) {
  switch (ownership) {
    case "service_managed":
      return i18n.t("config.ownership_service_managed");
    case "server_managed":
      return i18n.t("config.ownership_server_managed");
    case "third_party":
      return i18n.t("config.ownership_third_party");
  }
}

const sourceSummaryItems = computed(() => {
  const order: ServerConfigSourceKind[] = ["server_root", "manual_root", "manual_file"];
  return order
    .map((kind) => ({
      key: kind,
      label: getSourceKindLabel(kind),
      count: props.configFiles.filter((file) => file.source_kind === kind).length,
    }))
    .filter((item) => item.count > 0);
});

const ownershipSummaryItems = computed(() => {
  const order: ServerConfigOwnership[] = ["service_managed", "server_managed", "third_party"];
  return order
    .map((ownership) => ({
      key: ownership,
      label: getOwnershipLabel(ownership),
      count: props.configFiles.filter((file) => file.ownership === ownership).length,
    }))
    .filter((item) => item.count > 0);
});
</script>

<template>
  <div class="config-file-toolbar glass-card">
    <div class="config-file-toolbar-main">
      <div class="config-file-picker-wrap">
        <span class="text-caption config-file-picker-label">{{
          i18n.t("config.config_files")
        }}</span>
        <SLSelect
          :modelValue="selectedConfigLocator"
          :options="configFileOptions"
          :searchable="configFileOptions.length > 8"
          dropdownWidth="360px"
          @update:modelValue="emit('updateSelectedConfigFile', $event)"
        />
      </div>
      <SLButton
        v-if="compareSupported"
        size="sm"
        :variant="compareMode ? 'primary' : 'secondary'"
        class="config-compare-toggle"
        @click="emit('updateCompareMode', !compareMode)"
      >
        <FileDiff :size="16" />
        {{ i18n.t("config.compare.toggle") }}
      </SLButton>
    </div>

    <div class="config-discovery-row">
      <div class="config-discovery-actions">
        <span class="text-caption">{{ i18n.t("config.imported_folders") }}</span>
        <SLButton size="sm" variant="secondary" @click="emit('importConfigDirectory')">
          <Plus :size="14" />
          {{ i18n.t("config.import_folder") }}
        </SLButton>
      </div>
      <div
        class="config-import-list"
        :class="{ 'config-import-list--empty': manualImportDirs.length === 0 }"
      >
        <div v-for="path in manualImportDirs" :key="path" class="config-import-chip">
          <span class="config-import-chip-text text-mono">{{ path }}</span>
          <button
            class="config-import-chip-remove"
            type="button"
            :aria-label="i18n.t('config.remove_import')"
            @click="emit('removeConfigImportDirectory', path)"
          >
            <X :size="14" />
          </button>
        </div>
        <span v-if="manualImportDirs.length === 0" class="text-caption">-</span>
      </div>
    </div>

    <div class="config-discovery-row">
      <div class="config-discovery-actions">
        <span class="text-caption">{{ i18n.t("config.imported_files") }}</span>
        <SLButton size="sm" variant="secondary" @click="emit('importConfigFile')">
          <Plus :size="14" />
          {{ i18n.t("config.import_file") }}
        </SLButton>
      </div>
      <div
        class="config-import-list"
        :class="{ 'config-import-list--empty': manualImportFiles.length === 0 }"
      >
        <div v-for="path in manualImportFiles" :key="path" class="config-import-chip">
          <span class="config-import-chip-text text-mono">{{ path }}</span>
          <button
            class="config-import-chip-remove"
            type="button"
            :aria-label="i18n.t('config.remove_import')"
            @click="emit('removeConfigImportFile', path)"
          >
            <X :size="14" />
          </button>
        </div>
        <span v-if="manualImportFiles.length === 0" class="text-caption">-</span>
      </div>
    </div>

    <div class="config-file-search-row">
      <SLInput
        :modelValue="configSearchQuery"
        :placeholder="i18n.t('common.search')"
        @update:modelValue="emit('updateConfigSearchQuery', $event)"
      />
      <SLSelect
        :modelValue="configSearchMode"
        :options="configSearchModeOptions"
        :loading="configSearchLoading"
        dropdownWidth="160px"
        @update:modelValue="emit('updateConfigSearchMode', $event)"
      />
      <SLSelect
        :modelValue="configSearchScope"
        :options="configSearchScopeOptions"
        dropdownWidth="160px"
        @update:modelValue="emit('updateConfigSearchScope', $event)"
      />
      <SLSelect
        :modelValue="configJsonMode"
        :options="configJsonModeOptions"
        dropdownWidth="160px"
        @update:modelValue="emit('updateConfigJsonMode', $event)"
      />
    </div>

    <p v-if="configSearchError" class="config-file-search-error text-caption">
      {{ configSearchError }}
    </p>

    <div v-if="configFiles.length > 0" class="config-summary-row">
      <div class="config-summary-group">
        <span class="text-caption">{{ i18n.t("config.source_groups") }}</span>
        <div class="config-summary-chips">
          <span v-for="item in sourceSummaryItems" :key="item.key" class="config-summary-chip">
            {{ item.label }} {{ item.count }}
          </span>
        </div>
      </div>
      <div class="config-summary-group">
        <span class="text-caption">{{ i18n.t("config.owner_groups") }}</span>
        <div class="config-summary-chips">
          <span v-for="item in ownershipSummaryItems" :key="item.key" class="config-summary-chip">
            {{ item.label }} {{ item.count }}
          </span>
        </div>
      </div>
    </div>
  </div>

  <div v-if="!hasDiscoveredConfigFiles" class="empty-state glass-card">
    <p class="text-caption">{{ i18n.t("config.empty_config_folder") }}</p>
  </div>

  <div v-else-if="configFiles.length === 0" class="empty-state glass-card">
    <p class="text-caption">{{ i18n.t("common.no_match") }}</p>
  </div>

  <template v-else>
    <div v-show="editorMode === 'visual'">
      <div v-if="!isPropertiesFile" class="empty-state glass-card">
        <p class="text-caption">{{ i18n.t("config.source_mode") }}</p>
      </div>
      <ConfigCategories
        v-else
        :categories="categories"
        :activeCategory="activeCategory"
        :searchQuery="searchQuery"
        @updateCategory="emit('updateCategory', $event)"
        @updateSearch="emit('updateSearch', $event)"
      />

      <div v-if="loading || compareLoading" class="loading-state">
        <SLSpinner size="lg" />
        <span>{{ i18n.t("config.loading") }}</span>
      </div>

      <div
        v-else-if="isPropertiesFile && compareMode && !hasCompareTargets"
        class="empty-state glass-card"
      >
        <p class="text-caption">{{ i18n.t("config.compare.no_target_servers") }}</p>
      </div>

      <ConfigComparePanel
        v-else-if="isPropertiesFile && compareMode && compareTargetServerId"
        :compareTargetServerId="compareTargetServerId"
        :compareServerOptions="compareServerOptions"
        :hasCompareTargets="hasCompareTargets"
        :compareLoading="compareLoading"
        :inlineLabel="i18n.t('config.compare.inline_label')"
        :sourceServerName="sourceServerName"
        :targetServerName="targetServerName"
        :differenceBadgeText="compareDifferenceBadgeText"
        :differentLabel="i18n.t('config.compare.different')"
        :noDifferencesText="i18n.t('config.compare.no_differences')"
        :rows="comparePanelRows"
        :gamemodeOptions="gamemodeOptions"
        :difficultyOptions="difficultyOptions"
        @updateCompareTargetServer="emit('updateCompareTargetServer', $event)"
        @updateSourceValue="emit('updateValue', $event)"
        @updateTargetValue="emit('updateCompareTargetValue', $event)"
        @addSourceValue="emit('addSourceValue', $event)"
        @addTargetValue="emit('addTargetValue', $event)"
      />

      <div v-else-if="isPropertiesFile" class="config-entries">
        <div v-for="entry in filteredEntries" :key="entry.key" class="config-entry glass-card">
          <div class="entry-header">
            <div class="entry-key-row">
              <span class="entry-key text-mono">{{ entry.key }}</span>
            </div>
            <p v-if="translatedDescriptionByKey[entry.key]" class="entry-desc text-caption">
              {{ translatedDescriptionByKey[entry.key] }}
            </p>
          </div>
          <div class="entry-control">
            <ConfigPropertyEditorControl
              :propertyKey="entry.key"
              :modelValue="editValues[entry.key]"
              :valueType="entry.value_type"
              :defaultValue="entry.default_value"
              :numericError="numericFieldErrors[entry.key]"
              :gamemodeOptions="gamemodeOptions"
              :difficultyOptions="difficultyOptions"
              @update:modelValue="emit('updateValue', { key: entry.key, value: $event })"
            />
          </div>
        </div>
        <div v-if="filteredEntries.length === 0 && !loading" class="empty-state">
          <p class="text-caption">{{ i18n.t("config.no_config") }}</p>
        </div>
      </div>
    </div>

    <div v-show="editorMode === 'source'">
      <div class="source-editor-wrap" :class="{ 'source-editor-wrap--compare': compareMode }">
        <template v-if="isPropertiesFile && compareMode && compareTargetServerId">
          <div class="source-compare-grid">
            <div class="source-compare-column">
              <ConfigSourceEditor
                :modelValue="sourceDraftText"
                :title="sourceServerName"
                iconNavOnly
                @update:modelValue="emit('updateSourceDraft', $event)"
              />
            </div>
            <div class="source-compare-column">
              <ConfigSourceEditor
                :modelValue="compareTargetSourceDraftText"
                :title="targetServerName"
                iconNavOnly
                @update:modelValue="emit('updateCompareTargetSourceDraft', $event)"
              />
            </div>
          </div>
        </template>
        <ConfigSourceEditor
          v-else
          :modelValue="sourceDraftText"
          @update:modelValue="emit('updateSourceDraft', $event)"
        />
        <p v-if="sourceParseError" class="source-parse-error">
          {{ sourceParseError }}
        </p>
      </div>
    </div>

    <div
      class="config-floating-actions glass-strong"
      :class="{ 'config-floating-actions--unsaved': hasUnsavedChanges }"
    >
      <div class="floating-status-wrap">
        <div class="floating-status text-caption">{{ saveStatusText }}</div>
      </div>
      <div class="floating-actions-group">
        <SLTooltip :content="reloadCurrentTooltipText">
          <SLButton
            variant="secondary"
            size="sm"
            iconOnly
            class="config-floating-icon-btn"
            @click="emit('reloadCurrent')"
          >
            <RefreshCw :size="16" />
          </SLButton>
        </SLTooltip>
        <SLTooltip v-if="compareMode" :content="reloadCompareTooltipText">
          <SLButton
            variant="secondary"
            size="sm"
            iconOnly
            class="config-floating-icon-btn"
            :loading="compareLoading"
            :disabled="!compareTargetServerId"
            @click="emit('reloadCompare')"
          >
            <RefreshCw :size="16" />
          </SLButton>
        </SLTooltip>
        <SLButton
          variant="primary"
          size="sm"
          iconOnly
          class="config-floating-icon-btn"
          :class="
            hasUnsavedChanges
              ? 'config-floating-icon-btn--unsaved'
              : 'config-floating-icon-btn--idle'
          "
          :disabled="!hasUnsavedChanges"
          :loading="saving"
          @click="emit('saveProperties')"
        >
          <span
            class="save-icon-wrap"
            :class="{ 'save-icon-wrap--unsaved': hasUnsavedChanges && !saving }"
          >
            <Save :size="16" />
          </span>
        </SLButton>
      </div>
    </div>
  </template>
</template>

<style scoped>
.config-discovery-row {
  display: grid;
  grid-template-columns: 180px minmax(0, 1fr);
  gap: 12px;
  margin-top: 12px;
  align-items: start;
}

.config-discovery-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.config-import-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  min-height: 36px;
  align-items: center;
}

.config-import-list--empty {
  color: var(--sl-text-tertiary);
}

.config-import-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-surface);
}

.config-import-chip-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.config-import-chip-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--sl-text-tertiary);
  cursor: pointer;
}

.config-file-search-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 160px 160px 160px;
  gap: 12px;
  margin-top: 12px;
}

.config-file-search-error {
  margin-top: 8px;
  color: var(--sl-danger, #ef4444);
}

.config-summary-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 12px;
}

.config-summary-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.config-summary-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.config-summary-chip {
  display: inline-flex;
  align-items: center;
  padding: 6px 8px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background: var(--sl-surface);
  font-size: var(--sl-font-size-sm);
}

@media (max-width: 960px) {
  .config-discovery-row {
    grid-template-columns: 1fr;
  }

  .config-file-search-row {
    grid-template-columns: 1fr 1fr;
  }

  .config-summary-row {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .config-file-search-row {
    grid-template-columns: 1fr;
  }
}
</style>
