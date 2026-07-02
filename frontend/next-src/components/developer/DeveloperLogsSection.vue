<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import ConsoleOutput from "@components/console/ConsoleOutput.vue";
import WorkbenchPanel from "@next-src/components/workbench/WorkbenchPanel.vue";
import { i18n } from "@language";

interface LogFilterOption {
  label: string;
  value: string;
}

interface ConsoleOutputExpose {
  doScroll: () => void;
  appendLines: (lines: string[]) => void;
  clear: (resetScrollState?: boolean) => void;
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
  refresh: [];
  copy: [];
  export: [];
  clear: [];
  updateSelectedLogLevel: [value: string];
  updateSelectedLogModule: [value: string];
}>();

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

function syncRenderedLines(lines: string[]): void {
  const output = outputRef.value;
  if (!output) {
    return;
  }
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
    if (output) {
      syncRenderedLines(props.logLines);
    }
  },
  { flush: "post" },
);
</script>

<template>
  <WorkbenchPanel :title="i18n.t('developer.next.logs.title')" :description="i18n.t('developer.next.logs.description')">
    <template #actions>
      <SLButton variant="secondary" size="sm" @click="emit('refresh')">{{ i18n.t("common.refresh") }}</SLButton>
      <SLButton variant="secondary" size="sm" @click="emit('copy')">{{ i18n.t("developer.copy_logs") }}</SLButton>
      <SLButton variant="secondary" size="sm" :loading="exporting" :disabled="isBrowserMode || !hasLogEntries" @click="emit('export')">{{ i18n.t("developer.export_logs") }}</SLButton>
      <SLButton variant="danger" size="sm" :loading="clearing" :disabled="!hasLogEntries" @click="emit('clear')">{{ i18n.t("developer.clear_logs") }}</SLButton>
    </template>

    <div class="developer-logs-section__filters">
      <SLSelect :model-value="selectedLogLevel" :options="logLevelOptions" :label="i18n.t('developer.log_level_filter')" @update:model-value="emit('updateSelectedLogLevel', String($event))" />
      <SLSelect :model-value="selectedLogModule" :options="logModuleOptions" :label="i18n.t('developer.log_module_filter')" searchable @update:model-value="emit('updateSelectedLogModule', String($event))" />
      <p class="developer-logs-section__summary">{{ i18n.t("developer.log_filter_summary", { shown: filteredLogCount, total: totalLogCount }) }}</p>
    </div>

    <p v-if="error" class="developer-logs-section__error">{{ error }}</p>
    <div v-else class="developer-logs-section__output">
      <ConsoleOutput ref="outputRef" :consoleFontSize="consoleFontSize" :consoleFontFamily="consoleFontFamily" :consoleLetterSpacing="consoleLetterSpacing" :maxLogLines="maxLogLines" :userScrolledUp="userScrolledUp" @scroll="(value) => (userScrolledUp = value)" @scrollToBottom="userScrolledUp = false; outputRef?.doScroll();" />
      <div v-if="loading && logLines.length === 0" class="developer-logs-section__overlay">{{ i18n.t("common.loading") }}</div>
      <div v-else-if="!hasLogEntries" class="developer-logs-section__overlay">{{ i18n.t("developer.logs_empty") }}</div>
      <div v-else-if="logLines.length === 0" class="developer-logs-section__overlay">{{ i18n.t("developer.logs_filtered_empty") }}</div>
    </div>
  </WorkbenchPanel>
</template>

<style scoped>
.developer-logs-section__filters { display: grid; grid-template-columns: minmax(180px, 220px) minmax(220px, 320px) 1fr; gap: 10px; align-items: end; }
.developer-logs-section__summary, .developer-logs-section__error { margin: 0; font-size: 0.84rem; line-height: 1.45; }
.developer-logs-section__summary { color: var(--sl-text-secondary); }
.developer-logs-section__error { color: var(--sl-error); }
.developer-logs-section__output { position: relative; min-height: 300px; display: flex; padding: 8px; border-radius: 18px; border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent); background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent); overflow: hidden; }
.developer-logs-section__output :deep(.console-output) { flex: 1; min-height: 0; padding: 4px 6px; }
.developer-logs-section__overlay { position: absolute; inset: 8px; display: flex; align-items: center; justify-content: center; padding: 12px; color: var(--sl-text-tertiary); background: color-mix(in srgb, var(--sl-bg-secondary) 82%, transparent); text-align: center; pointer-events: none; }
@media (max-width: 860px) { .developer-logs-section__filters { grid-template-columns: 1fr; align-items: stretch; } }
</style>
