<script setup lang="ts">
import { computed, watch } from "vue";
import { useRouter } from "vue-router";
import { useSettingsStore } from "@stores/settingsStore";
import { i18n } from "@language";
import SLCard from "@components/common/SLCard.vue";
import DeveloperActionsPanel from "@components/views/developer/DeveloperActionsPanel.vue";
import DeveloperLogPanel from "@components/views/developer/DeveloperLogPanel.vue";
import DeveloperSystemPanel from "@components/views/developer/DeveloperSystemPanel.vue";
import { useDeveloperTools } from "@composables/useDeveloperTools";

const settingsStore = useSettingsStore();
const router = useRouter();
const developerEnabled = computed(() => settingsStore.settings.developer_mode);

const {
  logLines,
  filteredLogCount,
  totalLogCount,
  hasLogEntries,
  selectedLogLevel,
  selectedLogModule,
  logLevelOptions,
  logModuleOptions,
  systemInfo,
  version,
  loadingLogs,
  loadingSystem,
  exportingLogs,
  clearingLogs,
  downloadingUpdate,
  triggeringCrash,
  updateUrl,
  logError,
  systemError,
  isBrowserMode,
  canTriggerCrash,
  refreshLogs,
  refreshSystemInfo,
  copyLogs,
  exportLogsToFile,
  clearAllLogs,
  copySystemSummary,
  downloadUpdateFromUrl,
  triggerCrashTest,
} = useDeveloperTools({
  enabled: () => developerEnabled.value,
});

const consoleFontSize = computed(() => settingsStore.settings.console_font_size || 12);
const consoleFontFamily = computed(() => settingsStore.settings.console_font_family || "");
const consoleLetterSpacing = computed(() => settingsStore.settings.console_letter_spacing || 0);
const maxLogLines = computed(() => Math.max(100, settingsStore.settings.max_log_lines || 1000));

watch(
  developerEnabled,
  (enabled) => {
    if (!enabled) {
      router.replace("/settings");
    }
  },
  { flush: "sync" },
);
</script>

<template>
  <div class="developer-view animate-fade-in-up">
    <SLCard
      v-if="!developerEnabled"
      :title="i18n.t('developer.disabled_title')"
      :subtitle="i18n.t('developer.disabled_desc')"
    />

    <template v-else>
      <SLCard :title="i18n.t('developer.title')" :subtitle="i18n.t('developer.subtitle')">
        <p class="developer-hero-text">{{ i18n.t("developer.hero") }}</p>
      </SLCard>

      <DeveloperSystemPanel
        :system-info="systemInfo"
        :version="version"
        :loading="loadingSystem"
        :error="systemError"
        @refresh="refreshSystemInfo"
      />

      <DeveloperLogPanel
        :log-lines="logLines"
        :filtered-log-count="filteredLogCount"
        :total-log-count="totalLogCount"
        :has-log-entries="hasLogEntries"
        :loading="loadingLogs"
        :exporting="exportingLogs"
        :clearing="clearingLogs"
        :is-browser-mode="isBrowserMode"
        :error="logError"
        :selected-log-level="selectedLogLevel"
        :selected-log-module="selectedLogModule"
        :log-level-options="logLevelOptions"
        :log-module-options="logModuleOptions"
        :console-font-size="consoleFontSize"
        :console-font-family="consoleFontFamily"
        :console-letter-spacing="consoleLetterSpacing"
        :max-log-lines="maxLogLines"
        @update:selected-log-level="selectedLogLevel = $event"
        @update:selected-log-module="selectedLogModule = $event"
        @refresh="refreshLogs"
        @copy="copyLogs"
        @export="exportLogsToFile"
        @clear="clearAllLogs"
      />

      <DeveloperActionsPanel
        :update-url="updateUrl"
        :downloading-update="downloadingUpdate"
        :triggering-crash="triggeringCrash"
        :can-trigger-crash="canTriggerCrash"
        :has-system-info="!!systemInfo"
        :is-browser-mode="isBrowserMode"
        @update:update-url="updateUrl = $event"
        @copy-system="copySystemSummary"
        @download-update="downloadUpdateFromUrl"
        @trigger-crash="triggerCrashTest"
      />
    </template>
  </div>
</template>

<style scoped>
.developer-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  max-width: 1080px;
  margin: 0 auto;
  padding-bottom: var(--sl-space-2xl);
}

.developer-hero-text {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.7;
}
</style>
