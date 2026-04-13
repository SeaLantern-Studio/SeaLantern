<script setup lang="ts">
import SLSpinner from "@components/common/SLSpinner.vue";
import SLButton from "@components/common/SLButton.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import ConfigCategories from "@components/config/ConfigCategories.vue";
import ConfigSourceEditor from "@components/config/ConfigSourceEditor.vue";
import ConfigPropertyEditorControl from "@components/config/ConfigPropertyEditorControl.vue";
import ConfigComparePanel from "@components/config/ConfigComparePanel.vue";
import type { ComparePanelRow } from "@views/config/useConfigCompare";
import type { ConfigEntry as ConfigEntryType } from "@api/config";
import { i18n } from "@language";
import { RefreshCw, Save } from "lucide-vue-next";

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
  loading: boolean;
  compareLoading: boolean;
  compareMode: boolean;
  hasCompareTargets: boolean;
  compareTargetServerId: string;
  compareServerOptions: Option[];
  compareDifferenceBadgeText: string;
  comparePanelRows: ComparePanelRow[];
  sourceServerName: string;
  targetServerName: string;
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

defineProps<Props>();

const emit = defineEmits<{
  updateCategory: [category: string];
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
</script>

<template>
  <div v-show="editorMode === 'visual'">
    <ConfigCategories
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

    <div v-else-if="compareMode && !hasCompareTargets" class="empty-state glass-card">
      <p class="text-caption">{{ i18n.t("config.compare.no_target_servers") }}</p>
    </div>

    <ConfigComparePanel
      v-else-if="compareMode && compareTargetServerId"
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

    <div v-else class="config-entries">
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
      <template v-if="compareMode && compareTargetServerId">
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
          hasUnsavedChanges ? 'config-floating-icon-btn--unsaved' : 'config-floating-icon-btn--idle'
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
