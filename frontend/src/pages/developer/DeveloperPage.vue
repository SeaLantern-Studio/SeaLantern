<script setup lang="ts">
import DeveloperAnnouncementModal from "@src/components/developer/DeveloperAnnouncementModal.vue";
import DeveloperDangerSection from "@src/components/developer/DeveloperDangerSection.vue";
import DeveloperLogsSection from "@src/components/developer/DeveloperLogsSection.vue";
import DeveloperOverviewSection from "@src/components/developer/DeveloperOverviewSection.vue";
import DeveloperToolsSection from "@src/components/developer/DeveloperToolsSection.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchSectionHeader from "@src/components/workbench/WorkbenchSectionHeader.vue";
import WorkbenchSplitView from "@src/components/workbench/WorkbenchSplitView.vue";
import { i18n } from "@language";
import { useDeveloperPage } from "./useDeveloperPage";

const {
  activeSectionId,
  currentSection,
  sectionItems,
  summaryFacts,
  version,
  systemInfo,
  loadingSystem,
  systemError,
  memoryDisplayPrecision,
  logLines,
  filteredLogCount,
  totalLogCount,
  hasLogEntries,
  loadingLogs,
  exportingLogs,
  clearingLogs,
  isBrowserMode,
  logError,
  selectedLogLevel,
  selectedLogModule,
  logLevelOptions,
  logModuleOptions,
  downloadingUpdate,
  triggeringCrash,
  updateUrl,
  activeAnnouncement,
  canTriggerCrash,
  refreshLogs,
  refreshSystemInfo,
  copyLogs,
  exportLogsToFile,
  clearAllLogs,
  copySystemSummary,
  downloadUpdateFromUrl,
  triggerCrashTest,
  selectSection,
  setSelectedLogLevel,
  setSelectedLogModule,
  setUpdateUrl,
  triggerToastTest,
  triggerAnnouncementTest,
  clearTestAnnouncement,
  consoleDisplay,
} = useDeveloperPage();
</script>

<template>
  <div class="developer-page">
    <WorkbenchFactGrid :items="summaryFacts" />

    <WorkbenchSplitView
      :items="sectionItems"
      :active-id="activeSectionId"
      :aria-label="i18n.t('developer.next.nav_aria_label')"
      :ariaLabel="i18n.t('developer.next.nav_aria_label')"
      @select="selectSection"
    >
      <template #content-header>
        <WorkbenchSectionHeader
          :title="currentSection.label"
          :description="currentSection.description"
        />
      </template>

      <DeveloperOverviewSection
        v-if="activeSectionId === 'overview'"
        :version="version"
        :system-info="systemInfo"
        :loading="loadingSystem"
        :error="systemError"
        :memory-display-precision="memoryDisplayPrecision"
        @refresh="refreshSystemInfo"
        @copy-system="copySystemSummary"
      />

      <DeveloperLogsSection
        v-else-if="activeSectionId === 'logs'"
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
        :console-font-size="consoleDisplay.fontSize"
        :console-font-family="consoleDisplay.fontFamily"
        :console-letter-spacing="consoleDisplay.letterSpacing"
        :max-log-lines="consoleDisplay.maxLogLines"
        @refresh="refreshLogs"
        @copy="copyLogs"
        @export="exportLogsToFile"
        @clear="clearAllLogs"
        @update-selected-log-level="setSelectedLogLevel"
        @update-selected-log-module="setSelectedLogModule"
      />

      <DeveloperToolsSection
        v-else-if="activeSectionId === 'tools'"
        :update-url="updateUrl"
        :downloading-update="downloadingUpdate"
        :is-browser-mode="isBrowserMode"
        @update-update-url="setUpdateUrl"
        @download-update="downloadUpdateFromUrl"
        @show-toast="triggerToastTest"
        @show-announcement="triggerAnnouncementTest"
      />

      <DeveloperDangerSection
        v-else
        :can-trigger-crash="canTriggerCrash"
        :triggering-crash="triggeringCrash"
        @trigger-crash="triggerCrashTest"
      />
    </WorkbenchSplitView>

    <DeveloperAnnouncementModal :announcement="activeAnnouncement" @close="clearTestAnnouncement" />
  </div>
</template>

<style scoped>
.developer-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}
</style>
