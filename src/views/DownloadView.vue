<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import { SLTabBar } from "@components/common";
import DownloadForm from "@components/views/download/DownloadForm.vue";
import DownloadServerForm from "@components/views/download/DownloadServerForm.vue";
import DownloadProgress from "@components/views/download/DownloadProgress.vue";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { downloadApi, downloadServerApi, type DownloadLink } from "@api/downloader";
import { systemApi } from "@api/system";
import { useCreateServerDraftStore } from "@stores/createServerDraft.ts";
import { i18n } from "@language";

const router = useRouter();
const createServerDraftStore = useCreateServerDraftStore();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: submitting, start: startLoading, stop: stopLoading } = useLoading();

// Tab state
const activeTab = ref<"file" | "server">("server");
const taskOriginTab = ref<"file" | "server" | null>(null);

const tabs = computed(() => [
  { key: "server" as const, label: i18n.t("downloadServerView.title") },
  { key: "file" as const, label: i18n.t("download-file.title") },
]);

function handleTabChange(tab: string | null) {
  if (tab) {
    activeTab.value = tab as "file" | "server";
  }
}

// File download state
const url = ref("");
const savePath = ref("");
const filename = ref("");
const threadCount = ref("32");

// Server download state
const serverTypes = ref<string[]>([]);
const versions = ref<string[]>([]);
const selectedType = ref("");
const selectedVersion = ref("");
const serverSaveDir = ref("");
const serverFilename = ref("server.jar");
const serverThreadCount = ref("32");
const info = ref<DownloadLink | null>(null);

const loadingTypes = ref(false);
const loadingVersions = ref(false);
const loadingInfo = ref(false);

// Download task state
const {
  taskInfo,
  start: startTask,
  reset: resetTask,
  errorMessage: taskError,
} = downloadApi.useDownload();

const isDownloading = computed(() => taskInfo.id !== "" && !taskInfo.isFinished);
const isTaskVisibleForCurrentTab = computed(
  () => taskInfo.id !== "" && taskOriginTab.value === activeTab.value,
);
const isDownloadingCurrentTab = computed(
  () => isTaskVisibleForCurrentTab.value && !taskInfo.isFinished,
);
const isDownloadCompleted = computed(
  () => isTaskVisibleForCurrentTab.value && taskInfo.isFinished && !taskError.value,
);
const loadingAny = computed(() => loadingTypes.value || loadingVersions.value || loadingInfo.value);
const combinedLoading = computed(() => submitting.value || isDownloading.value || loadingAny.value);

// File download computed properties
const canFileDownload = computed(() => {
  if (
    isDownloading.value ||
    url.value.trim() === "" ||
    savePath.value.trim() === "" ||
    filename.value.trim() === "" ||
    threadCount.value.trim() === ""
  ) {
    return false;
  }

  // 验证线程数
  if (!checkThreadCount()) {
    return false;
  }

  // 验证URL格式
  try {
    const validatedUrl = new URL(url.value.trim());
  } catch {
    return false;
  }

  return true;
});

// Server download computed properties
const serverTypeOptions = computed(() =>
  serverTypes.value.map((type) => ({ label: type, value: type })),
);

const versionOptions = computed(() => {
  return [...versions.value]
    .toSorted((a, b) => {
      const aParts = a.split(".").map(Number);
      const bParts = b.split(".").map(Number);
      for (let i = 0; i < Math.max(aParts.length, bParts.length); i++) {
        const aNum = aParts[i] || 0;
        const bNum = bParts[i] || 0;
        if (bNum - aNum !== 0) return bNum - aNum;
      }
      return 0;
    })
    .map((v) => ({ label: v, value: v }));
});

const canServerDownload = computed(() => {
  if (combinedLoading.value) return false;
  if (!selectedType.value || !selectedVersion.value) return false;
  if (!info.value?.url) return false;
  if (!serverSaveDir.value.trim() || !serverFilename.value.trim()) return false;
  return /^[1-9]\d*$/.test(serverThreadCount.value.trim());
});

const canGoCreate = computed(() => {
  return (
    taskOriginTab.value === "server" &&
    isTaskVisibleForCurrentTab.value &&
    taskInfo.isFinished &&
    !taskError.value
  );
});

const fileDownloadButtonLabel = computed(() => {
  if (isDownloadingCurrentTab.value) return i18n.t("download-file.downloading");
  if (isDownloadCompleted.value) return i18n.t("downloadServerView.status.finished");
  return i18n.t("download-file.download");
});

const serverDownloadButtonLabel = computed(() => {
  if (isDownloadingCurrentTab.value) return i18n.t("downloadServerView.actions.downloading");
  if (isDownloadCompleted.value) return i18n.t("downloadServerView.status.finished");
  return i18n.t("downloadServerView.actions.startDownload");
});

const savePathPreview = computed(() => {
  if (!serverSaveDir.value.trim() || !serverFilename.value.trim()) return "";
  return buildServerSavePath();
});

// File download methods
function checkUrl() {
  try {
    const urlObj = new URL(url.value);
    const pathName = urlObj.pathname;
    const segments = pathName.split("/");
    if (segments.length > 1) {
      filename.value = segments[segments.length - 1];
    }
  } catch {
    // 当URL无效时，不重置filename，因为用户可能手动输入了文件名
  }
}

async function pickFileFolder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) savePath.value = result;
  } catch (e) {
    console.error("Pick file error:", e);
  }
}

function checkThreadCount() {
  const threadCountValue = threadCount.value;
  if (threadCountValue == "") {
    return false;
  }
  if (!/^-?\d+$/.test(threadCountValue)) {
    showError(i18n.t("download-file.thread_count_invalid"));
    return false;
  }
  if (!/^[1-9]\d*$/.test(threadCountValue)) {
    showError(i18n.t("download-file.thread_count_positive"));
    return false;
  }
  if (parseInt(threadCountValue, 10) > 256) {
    showError(i18n.t("download-file.thread_count_too_big"));
    return false;
  }
  return true;
}

// Server download methods
async function loadServerTypes() {
  loadingTypes.value = true;
  clearError();
  try {
    const types = await downloadServerApi.getServerTypes();
    serverTypes.value = types;
    if (types.length > 0) selectedType.value = types[0];
  } catch (e) {
    showError(String(e));
  } finally {
    loadingTypes.value = false;
  }
}

// 修复：默认选中 原始数组最后一个元素（最新版本）
async function loadVersionsByType(serverType: string) {
  if (!serverType) return;
  loadingVersions.value = true;
  clearError();
  versions.value = [];
  selectedVersion.value = "";
  info.value = null;

  try {
    const list = await downloadServerApi.getVersionsByType(serverType);
    versions.value = list;
    // 核心修复：后端返回数组升序，最后一个 = 最新版本
    if (list.length > 0) selectedVersion.value = list[list.length - 1];
  } catch (e) {
    showError(String(e));
  } finally {
    loadingVersions.value = false;
  }
}

async function loadDownloadInfo(serverType: string, version: string) {
  if (!serverType || !version) return;
  loadingInfo.value = true;
  clearError();
  info.value = null;
  serverFilename.value = "server.jar";

  try {
    const result = await downloadServerApi.getDownloadInfo(serverType, version);
    info.value = result;
    serverFilename.value = result.fileName;
  } catch (e) {
    showError(String(e));
  } finally {
    loadingInfo.value = false;
  }
}

async function pickServerFolder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) serverSaveDir.value = result;
  } catch (e) {
    showError(String(e));
  }
}

function buildServerSavePath() {
  const dir = serverSaveDir.value.replace(/[\\/]+$/, "").replace(/\\/g, "/");
  const file = serverFilename.value.replace(/^[\\/]+/, "");
  return `${dir}/${file}`;
}

function gotoCreatePage(sourcePath: string) {
  createServerDraftStore.setDraft({
    sourcePath: sourcePath,
    sourceType: "archive",
  });
  router.push("/create");
}

// Common methods
async function cancelDownload() {
  try {
    if (taskInfo.id) {
      await downloadApi.cancelDownloadTask(taskInfo.id);
    }

    resetTask();
    taskOriginTab.value = null;
  } catch (e) {
    showError(String(e));
  } finally {
    stopLoading();
  }
}

async function handleFileDownload() {
  if (combinedLoading.value) return;

  const threadCountValue = threadCount.value;
  if (threadCountValue == "") {
    showError(i18n.t("download-file.thread_count_empty"));
    return;
  }
  if (!/^-?\d+$/.test(threadCountValue)) {
    showError(i18n.t("download-file.thread_count_invalid"));
    return;
  }
  if (!/^[1-9]\d*$/.test(threadCountValue)) {
    showError(i18n.t("download-file.thread_count_positive"));
    return;
  }
  const threadCountInt = parseInt(threadCountValue, 10);

  clearError();
  resetTask();
  taskOriginTab.value = "file";
  startLoading();

  try {
    const normalizedSavePath = savePath.value.replace(/\\/g, "/");
    await startTask({
      url: url.value,
      savePath: normalizedSavePath + "/" + filename.value,
      threadCount: threadCountInt,
    });

    if (taskError.value) showError(taskError.value);
  } catch (e) {
    showError(String(e));
  } finally {
    stopLoading();
  }
}

async function handleServerDownload() {
  if (!canServerDownload.value || !info.value) return;

  clearError();
  resetTask();
  taskOriginTab.value = "server";
  startLoading();

  const targetPath = buildServerSavePath();

  try {
    await startTask({
      url: info.value.url,
      savePath: targetPath,
      threadCount: parseInt(serverThreadCount.value, 10),
    });

    if (taskError.value) {
      showError(taskError.value);
    }
  } catch (e) {
    showError(String(e));
  } finally {
    stopLoading();
  }
}

const statusLabel = computed(() => {
  if (taskError.value) return i18n.t("download-file.failed");
  if (taskInfo.isFinished) return i18n.t("downloadServerView.status.finished");
  return i18n.t("download-file.downloading");
});

// Watchers
watch(selectedType, (val) => {
  loadVersionsByType(val);
});

watch(selectedVersion, (val) => {
  if (selectedType.value && val) {
    loadDownloadInfo(selectedType.value, val);
  }
});

watch(taskError, (newError) => {
  if (newError) showError(newError);
});

onMounted(() => {
  loadServerTypes();
});
</script>

<template>
  <div class="download-view animate-fade-in-up">
    <div v-if="errorMsg" class="error-banner">
      <span>{{ errorMsg }}</span>
      <button class="error-close" @click="clearError">x</button>
    </div>

    <!-- Tab selection -->
    <SLTabBar v-model="activeTab" :tabs="tabs" :level="1" @update:modelValue="handleTabChange" />

    <!-- File download tab -->
    <SLCard v-if="activeTab === 'file'" :title="i18n.t('download-file.title')">
      <DownloadForm
        :url="url"
        :savePath="savePath"
        :filename="filename"
        :threadCount="threadCount"
        :isDownloading="isDownloading"
        @update:url="url = $event"
        @update:savePath="savePath = $event"
        @update:filename="filename = $event"
        @update:threadCount="threadCount = $event"
        @checkUrl="checkUrl()"
        @pickFolder="pickFileFolder"
        @checkThreadCount="checkThreadCount"
      />
    </SLCard>

    <!-- Server download tab -->
    <SLCard v-else-if="activeTab === 'server'" :title="i18n.t('downloadServerView.title')">
      <DownloadServerForm
        :serverTypeOptions="serverTypeOptions"
        :versionOptions="versionOptions"
        :selectedType="selectedType"
        :selectedVersion="selectedVersion"
        :filename="serverFilename"
        :saveDir="serverSaveDir"
        :threadCount="serverThreadCount"
        :loadingTypes="loadingTypes"
        :loadingVersions="loadingVersions"
        :isDownloading="isDownloading"
        :savePathPreview="savePathPreview"
        :infoUrl="info?.url"
        @update:selectedType="selectedType = $event"
        @update:selectedVersion="selectedVersion = $event"
        @update:filename="serverFilename = $event"
        @update:saveDir="serverSaveDir = $event"
        @update:threadCount="serverThreadCount = $event"
        @pickFolder="pickServerFolder"
      />
    </SLCard>

    <!-- Actions -->
    <div class="create-actions">
      <SLButton variant="secondary" size="lg" @click="cancelDownload">
        {{ i18n.t("download-file.cancel") }}
      </SLButton>
      <SLButton
        v-if="activeTab === 'file'"
        :variant="isDownloadCompleted ? 'success' : 'primary'"
        size="lg"
        :disabled="!canFileDownload"
        @click="handleFileDownload"
        :loading="isDownloadingCurrentTab"
      >
        {{ fileDownloadButtonLabel }}
      </SLButton>
      <SLButton
        v-else-if="activeTab === 'server'"
        :variant="isDownloadCompleted ? 'success' : 'primary'"
        size="lg"
        :disabled="!canServerDownload"
        @click="handleServerDownload"
        :loading="isDownloadingCurrentTab"
      >
        {{ serverDownloadButtonLabel }}
      </SLButton>
      <SLButton
        v-if="activeTab === 'server'"
        variant="secondary"
        size="lg"
        :disabled="!canGoCreate"
        @click="gotoCreatePage(buildServerSavePath())"
      >
        {{ i18n.t("downloadServerView.actions.goCreatePage") }}
      </SLButton>
    </div>

    <!-- Download progress -->
    <Transition name="fade">
      <div v-if="isTaskVisibleForCurrentTab" class="bottom-progress-area">
        <DownloadProgress :taskInfo="taskInfo" :taskError="taskError" :statusLabel="statusLabel" />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.download-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
  max-width: 640px;
  margin: 0 auto;
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-error);
  font-size: var(--sl-font-size-base);
}

.error-close {
  color: var(--sl-error);
  font-weight: 600;
  cursor: pointer;
  background: none;
  border: none;
}

.create-actions {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
  margin-top: var(--sl-space-md);
}

.animate-fade-in-up {
  animation: fadeInUp 0.4s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.bottom-progress-area {
  margin-top: var(--sl-space-lg);
  display: flex;
  justify-content: center;
  width: 100%;
}
</style>
