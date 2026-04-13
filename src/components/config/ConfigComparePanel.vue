<script setup lang="ts">
import SLSelect from "@components/common/SLSelect.vue";
import ConfigPropertyEditorControl from "@components/config/ConfigPropertyEditorControl.vue";

interface Option {
  label: string;
  value: string | number;
}

interface CompareControlState {
  key: string;
  value: string;
  valueType: string;
  defaultValue: string;
  numericError?: string;
}

interface ComparePanelRow {
  key: string;
  description: string;
  different: boolean;
  onlyInSource: boolean;
  onlyInTarget: boolean;
  source: CompareControlState;
  target: CompareControlState;
}

interface Props {
  compareTargetServerId: string;
  compareServerOptions: Option[];
  hasCompareTargets: boolean;
  compareLoading: boolean;
  inlineLabel: string;
  sourceServerName: string;
  targetServerName: string;
  differenceBadgeText: string;
  differentLabel: string;
  noDifferencesText: string;
  rows: ComparePanelRow[];
  gamemodeOptions: Option[];
  difficultyOptions: Option[];
}

defineProps<Props>();

const emit = defineEmits<{
  updateCompareTargetServer: [value: string | number];
  updateSourceValue: [payload: { key: string; value: string | boolean | number }];
  updateTargetValue: [payload: { key: string; value: string | boolean | number }];
}>();
</script>

<template>
  <div class="compare-entries">
    <div class="compare-header glass-card">
      <div class="compare-column-head compare-column-meta">
        <div class="compare-header-control">
          <span class="text-caption compare-target-label">{{ inlineLabel }}</span>
          <SLSelect
            :modelValue="compareTargetServerId"
            :options="compareServerOptions"
            :disabled="!hasCompareTargets || compareLoading"
            class="compare-target-select"
            @update:modelValue="emit('updateCompareTargetServer', $event)"
          />
        </div>
      </div>
      <div class="compare-column-head">
        <div class="compare-server-heading">
          <span class="text-caption compare-server-title">{{ sourceServerName }}</span>
        </div>
      </div>
      <div class="compare-column-head">
        <div class="compare-server-heading">
          <span class="text-caption compare-server-title">{{ targetServerName }}</span>
          <span class="compare-count-badge">{{ differenceBadgeText }}</span>
        </div>
      </div>
    </div>

    <div
      v-for="row in rows"
      :key="row.key"
      class="compare-entry glass-card"
      :class="{
        different: row.different,
        'only-source': row.onlyInSource,
        'only-target': row.onlyInTarget,
      }"
    >
      <div class="compare-meta">
        <div class="entry-key-row">
          <span class="entry-key text-mono">{{ row.key }}</span>
          <span v-if="row.different" class="compare-diff-badge">
            {{ differentLabel }}
          </span>
        </div>
        <p v-if="row.description" class="entry-desc text-caption">
          {{ row.description }}
        </p>
      </div>
      <div class="compare-value-block compare-source-block">
        <div class="entry-control compare-entry-control">
          <ConfigPropertyEditorControl
            :propertyKey="row.source.key"
            :modelValue="row.source.value"
            :valueType="row.source.valueType"
            :defaultValue="row.source.defaultValue"
            :numericError="row.source.numericError"
            :gamemodeOptions="gamemodeOptions"
            :difficultyOptions="difficultyOptions"
            @update:modelValue="emit('updateSourceValue', { key: row.key, value: $event })"
          />
        </div>
      </div>
      <div class="compare-value-block compare-target-block">
        <div class="entry-control compare-entry-control">
          <ConfigPropertyEditorControl
            :propertyKey="row.target.key"
            :modelValue="row.target.value"
            :valueType="row.target.valueType"
            :defaultValue="row.target.defaultValue"
            :numericError="row.target.numericError"
            :gamemodeOptions="gamemodeOptions"
            :difficultyOptions="difficultyOptions"
            @update:modelValue="emit('updateTargetValue', { key: row.key, value: $event })"
          />
        </div>
      </div>
    </div>

    <div v-if="rows.length === 0" class="empty-state glass-card">
      <p class="text-caption">{{ noDifferencesText }}</p>
    </div>
  </div>
</template>
