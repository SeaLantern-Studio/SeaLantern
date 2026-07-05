<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import DownloadTaskList from "@src/components/downloads/DownloadTaskList.vue";
import FileDownloadSection from "@src/components/downloads/FileDownloadSection.vue";
import ServerDownloadSection from "@src/components/downloads/ServerDownloadSection.vue";
import WorkbenchFactGrid from "@src/components/workbench/WorkbenchFactGrid.vue";
import WorkbenchSectionHeader from "@src/components/workbench/WorkbenchSectionHeader.vue";
import WorkbenchSplitView from "@src/components/workbench/WorkbenchSplitView.vue";
import WorkbenchStatusBanner from "@src/components/workbench/WorkbenchStatusBanner.vue";
import { i18n } from "@language";
import { useDownloadsPage } from "./useDownloadsPage";

const {
  bootstrapping,
  loadingServerTypes,
  loadingVersions,
  submittingServer,
  submittingFile,
  pageError,
  activeSectionId,
  currentSection,
  sectionItems,
  savePaths,
  fileForm,
  serverForm,
  serverTypeOptions,
  versionOptions,
  downloadInfo,
  currentTask,
  recentTasks,
  hasAnyTask,
  hasActiveTask,
  isTaskRunning,
  summaryFacts,
  fileSavePath,
  serverSavePath,
  canSubmitServer,
  canSubmitFile,
  selectSection,
  fillFilenameFromUrl,
  updateSelectedType,
  updateSelectedVersion,
  pickServerDirectory,
  pickFileDirectory,
  submitServerDownload,
  submitFileDownload,
  cancelActiveTask,
  goToCreateInstance,
} = useDownloadsPage();
</script>

<template>
  <div class="downloads-page">
    <div v-if="hasActiveTask && isTaskRunning" class="downloads-page__top-actions">
      <SLButton variant="secondary" size="sm" @click="cancelActiveTask">
        {{ i18n.t("downloads.next.cancel_active") }}
      </SLButton>
    </div>

    <WorkbenchFactGrid :items="summaryFacts" />

    <WorkbenchStatusBanner v-if="pageError" tone="error">
      <strong>{{ i18n.t("downloads.next.error_title") }}</strong>
      <span>{{ pageError }}</span>
    </WorkbenchStatusBanner>

    <WorkbenchSplitView
      :items="sectionItems"
      :active-id="activeSectionId"
      :aria-label="i18n.t('downloads.next.nav_aria_label')"
      :ariaLabel="i18n.t('downloads.next.nav_aria_label')"
      @select="selectSection"
    >
      <template #content-header>
        <WorkbenchSectionHeader
          :title="currentSection.label"
          :description="currentSection.description"
        />
      </template>

      <section v-if="bootstrapping" class="downloads-page__loading">
        <div class="downloads-page__loading-card"></div>
        <div class="downloads-page__loading-card"></div>
      </section>

      <template v-else-if="activeSectionId === 'tasks'">
        <DownloadTaskList
          v-if="hasActiveTask && currentTask"
          :title="i18n.t('downloads.next.tasks.current_title')"
          :tasks="[currentTask]"
          :empty-title="i18n.t('downloads.next.tasks.empty_current_title')"
          :empty-description="i18n.t('downloads.next.tasks.empty_current_description')"
          @create-instance="goToCreateInstance"
        />
        <DownloadTaskList
          :title="i18n.t('downloads.next.tasks.recent_title')"
          :tasks="recentTasks"
          :empty-title="i18n.t('downloads.next.tasks.empty_recent_title')"
          :empty-description="
            hasAnyTask
              ? i18n.t('downloads.next.tasks.empty_recent_description')
              : i18n.t('downloads.next.tasks.empty_all_description')
          "
          @create-instance="goToCreateInstance"
        />
      </template>

      <ServerDownloadSection
        v-else-if="activeSectionId === 'server'"
        :selected-type="serverForm.selectedType"
        :selected-version="serverForm.selectedVersion"
        :filename="serverForm.filename"
        :save-dir="savePaths.serverDir"
        :thread-count="serverForm.threadCount"
        :loading-types="loadingServerTypes"
        :loading-versions="loadingVersions"
        :server-type-options="serverTypeOptions"
        :version-options="versionOptions"
        :download-url="downloadInfo?.url ?? ''"
        :save-path="serverSavePath"
        :can-submit="canSubmitServer"
        :submitting="submittingServer"
        @update-selected-type="updateSelectedType"
        @update-selected-version="updateSelectedVersion"
        @update-filename="serverForm.filename = $event"
        @update-thread-count="serverForm.threadCount = $event"
        @pick-folder="pickServerDirectory"
        @submit="submitServerDownload"
      />

      <FileDownloadSection
        v-else
        :url="fileForm.url"
        :filename="fileForm.filename"
        :save-dir="savePaths.fileDir"
        :thread-count="fileForm.threadCount"
        :save-path="fileSavePath"
        :can-submit="canSubmitFile"
        :submitting="submittingFile"
        @update-url="fileForm.url = $event"
        @update-filename="fileForm.filename = $event"
        @update-thread-count="fileForm.threadCount = $event"
        @fill-filename="fillFilenameFromUrl"
        @pick-folder="pickFileDirectory"
        @submit="submitFileDownload"
      />
    </WorkbenchSplitView>
  </div>
</template>

<style scoped>
.downloads-page {
  min-width: 0;
  display: grid;
  gap: 16px;
}
.downloads-page__top-actions {
  display: flex;
  justify-content: flex-end;
}
.downloads-page__loading {
  display: grid;
  gap: 14px;
}
.downloads-page__loading-card {
  min-height: 180px;
  border-radius: 22px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--sl-bg-secondary) 86%, transparent) 0%,
    color-mix(in srgb, var(--sl-surface) 92%, transparent) 50%,
    color-mix(in srgb, var(--sl-bg-secondary) 86%, transparent) 100%
  );
  background-size: 200% 100%;
  animation: downloads-page-skeleton 1.2s ease-in-out infinite;
}
@keyframes downloads-page-skeleton {
  0% {
    background-position: 100% 0;
  }
  100% {
    background-position: -100% 0;
  }
}
</style>
