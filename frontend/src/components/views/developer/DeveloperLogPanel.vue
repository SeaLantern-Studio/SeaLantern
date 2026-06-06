<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import SLSelect from "@components/common/SLSelect.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import { i18n } from "@language";

interface LogFilterOption {
  label: string;
  value: string;
}

const props = defineProps<{
  logLines: string[];
  filteredLogCount: number;
  totalLogCount: number;
  hasLogEntries: boolean;
  loading: boolean;
  exporting: boolean;
  clearing: boolean;
  isBrowserMode: boolean;
  error: string | null;
  selectedLogLevel: string;
  selectedLogModule: string;
  logLevelOptions: LogFilterOption[];
  logModuleOptions: LogFilterOption[];
  consoleFontSize: number;
  consoleFontFamily: string;
  consoleLetterSpacing: number;
  maxLogLines: number;
}>();

const emit = defineEmits<{
  (e: "refresh"): void;
  (e: "copy"): void;
  (e: "export"): void;
  (e: "clear"): void;
  (e: "update:selectedLogLevel", value: string): void;
  (e: "update:selectedLogModule", value: string): void;
}>();

interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: (resetScrollState?: boolean) => void;
}

const outputRef = ref<ConsoleOutputExpose | null>(null);
const userScrolledUp = ref(false);
const renderedCount = ref(0);
const renderedLines = ref<string[]>([]);
const lastRenderedFirstLine = ref("");
const lastRenderedLastLine = ref("");

function hasMatchingPrefix(lines: string[], prefixLength: number): boolean {
  if (prefixLength === 0) {
    return false;
  }

  for (let index = 0; index < prefixLength; index += 1) {
    if (lines[index] !== renderedLines.value[index]) {
      return false;
    }
  }

  return true;
}

const panelHeight = computed(() => {
  const visibleRows = Math.min(Math.max(props.logLines.length + 1, 8), 22);
  const rowHeight = Math.max(props.consoleFontSize, 12) * 1.45;
  const height = Math.round(visibleRows * rowHeight + 28);
  return `${Math.min(Math.max(height, 220), 420)}px`;
});

function syncRenderedLines(lines: string[]): void {
  const output = outputRef.value;
  if (!output) return;

  if (lines.length === 0) {
    if (renderedCount.value > 0) {
      output.clear(false);
      renderedCount.value = 0;
      renderedLines.value = [];
      lastRenderedFirstLine.value = "";
      lastRenderedLastLine.value = "";
    }
    return;
  }

  const canAppendOnly =
    renderedCount.value > 0 &&
    lines.length >= renderedCount.value &&
    lines[0] === lastRenderedFirstLine.value &&
    lines[renderedCount.value - 1] === lastRenderedLastLine.value &&
    hasMatchingPrefix(lines, renderedCount.value);

  if (canAppendOnly) {
    const appendedLines = lines.slice(renderedCount.value);
    if (appendedLines.length > 0) {
      output.appendLines(appendedLines);
      renderedCount.value = lines.length;
      renderedLines.value = lines.slice();
      lastRenderedLastLine.value = lines[lines.length - 1] || "";
    }
    return;
  }

  output.clear(false);
  output.appendLines(lines);
  renderedCount.value = lines.length;
  renderedLines.value = lines.slice();
  lastRenderedFirstLine.value = lines[0] || "";
  lastRenderedLastLine.value = lines[lines.length - 1] || "";

  if (!userScrolledUp.value) {
    nextTick(() => {
      outputRef.value?.doScroll();
    });
  }
}

watch(
  () => props.logLines,
  (lines) => {
    syncRenderedLines(lines);
  },
  { immediate: true },
);

watch(
  outputRef,
  (output) => {
    if (!output) return;
    syncRenderedLines(props.logLines);
  },
  { flush: "post" },
);
</script>

<template>
  <SLCard :title="i18n.t('developer.logs_title')" :subtitle="i18n.t('developer.logs_desc')">
    <template #actions>
      <div class="developer-log-actions">
        <SLButton variant="secondary" size="sm" @click="emit('refresh')">
          {{ i18n.t("common.refresh") }}
        </SLButton>
        <SLButton variant="secondary" size="sm" @click="emit('copy')">
          {{ i18n.t("developer.copy_logs") }}
        </SLButton>
        <SLButton
          variant="secondary"
          size="sm"
          :loading="exporting"
          :disabled="isBrowserMode || !hasLogEntries"
          @click="emit('export')"
        >
          {{ i18n.t("developer.export_logs") }}
        </SLButton>
        <SLButton
          variant="danger"
          size="sm"
          :loading="clearing"
          :disabled="!hasLogEntries"
          @click="emit('clear')"
        >
          {{ i18n.t("developer.clear_logs") }}
        </SLButton>
      </div>
    </template>

    <div class="developer-log-filters">
      <SLSelect
        :model-value="selectedLogLevel"
        :options="logLevelOptions"
        :label="i18n.t('developer.log_level_filter')"
        :placeholder="i18n.t('developer.all_log_levels')"
        dropdown-width="220px"
        @update:model-value="emit('update:selectedLogLevel', String($event))"
      />
      <SLSelect
        :model-value="selectedLogModule"
        :options="logModuleOptions"
        :label="i18n.t('developer.log_module_filter')"
        :placeholder="i18n.t('developer.all_log_modules')"
        searchable
        dropdown-width="260px"
        @update:model-value="emit('update:selectedLogModule', String($event))"
      />
      <p class="developer-log-filter-summary">
        {{
          i18n.t("developer.log_filter_summary", { shown: filteredLogCount, total: totalLogCount })
        }}
      </p>
    </div>

    <p v-if="error" class="developer-panel-error">{{ error }}</p>
    <div v-else class="developer-log-output" :style="{ height: panelHeight }">
      <ConsoleOutput
        ref="outputRef"
        :consoleFontSize="consoleFontSize"
        :consoleFontFamily="consoleFontFamily"
        :consoleLetterSpacing="consoleLetterSpacing"
        :maxLogLines="maxLogLines"
        :userScrolledUp="userScrolledUp"
        @scroll="(value) => (userScrolledUp = value)"
        @scrollToBottom="
          userScrolledUp = false;
          outputRef?.doScroll();
        "
      />
      <div v-if="loading && logLines.length === 0" class="developer-log-overlay">
        {{ i18n.t("common.loading") }}
      </div>
      <div v-else-if="!hasLogEntries" class="developer-log-overlay">
        {{ i18n.t("developer.logs_empty") }}
      </div>
      <div v-else-if="logLines.length === 0" class="developer-log-overlay">
        {{ i18n.t("developer.logs_filtered_empty") }}
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.developer-log-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sl-space-sm);
}

.developer-log-output {
  position: relative;
  display: flex;
  min-height: 220px;
  padding: 8px;
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  overflow: hidden;
}

.developer-log-filters {
  display: grid;
  grid-template-columns: minmax(180px, 220px) minmax(220px, 320px) 1fr;
  gap: var(--sl-space-sm);
  align-items: end;
  margin-bottom: var(--sl-space-md);
}

.developer-log-filter-summary {
  margin: 0;
  color: var(--sl-text-tertiary);
  font-size: 0.92rem;
  line-height: 1.4;
}

.developer-log-output :deep(.console-output) {
  flex: 1;
  min-height: 0;
  padding: 4px 6px;
}

.developer-log-overlay {
  position: absolute;
  inset: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px;
  color: var(--sl-text-tertiary);
  background: color-mix(in srgb, var(--sl-bg-secondary) 82%, transparent);
  text-align: center;
  pointer-events: none;
}

.developer-panel-error {
  margin: 0;
  color: var(--sl-error);
}

@media (max-width: 860px) {
  .developer-log-filters {
    grid-template-columns: 1fr;
    align-items: stretch;
  }
}
</style>
