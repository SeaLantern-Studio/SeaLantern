<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import DownloadForm from "@components/views/download/DownloadForm.vue";
import DownloadServerForm from "@components/views/download/DownloadServerForm.vue";
import { useToast } from "cmzya-modern-ui";
import { useLoading } from "@composables/useAsync";
import { downloadServerApi, type DownloadLink } from "@api/downloader";
import { systemApi } from "@api/system";
import { useCreateServerDraftStore } from "@stores/createServerDraft.ts";
import { useDownloadStore } from "@stores/downloadStore";
import { i18n } from "@language";
import { handleError } from "@utils/errorHandler";

const router = useRouter();
const createServerDraftStore = useCreateServerDraftStore();
const downloadStore = useDownloadStore();
const toast = useToast();
const { loading: submitting, start: startLoading, stop: stopLoading } = useLoading();

const MAX_DOWNLOAD_THREADS = 64;

type ThreadCountValidationResult =
  | { valid: true; value: number }
  | { valid: false; error: "empty" | "invalid" | "non_positive" | "too_large" };

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
const threadCountInvalid = ref(false);
const serverThreadCountInvalid = ref(false);

// 下载任务状态（来自全局 store，任意页面可读，顶部任务球依赖此）
const isDownloading = computed(() => downloadStore.isDownloading);
const isFinished = computed(() => downloadStore.isFinished);
const taskError = computed(() => downloadStore.taskError);
const taskOriginTab = computed(() => downloadStore.taskOriginTab);
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

  if (!validateThreadCount(threadCount.value).valid) return false;

  return parseDownloadUrl(url.value) !== null;
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
  return validateThreadCount(serverThreadCount.value).valid;
});

const canGoCreate = computed(() => {
  return taskOriginTab.value === "server" && isFinished.value && !taskError.value;
});

const fileDownloadButtonLabel = computed(() => {
  if (isDownloading.value && taskOriginTab.value === "file")
    return i18n.t("download-file.downloading");
  if (isFinished.value && taskOriginTab.value === "file" && !taskError.value)
    return i18n.t("downloadServerView.status.finished");
  return i18n.t("download-file.download");
});

const serverDownloadButtonLabel = computed(() => {
  if (isDownloading.value && taskOriginTab.value === "server")
    return i18n.t("downloadServerView.actions.downloading");
  if (isFinished.value && taskOriginTab.value === "server" && !taskError.value)
    return i18n.t("downloadServerView.status.finished");
  return i18n.t("downloadServerView.actions.startDownload");
});

const savePathPreview = computed(() => {
  if (!serverSaveDir.value.trim() || !serverFilename.value.trim()) return "";
  return buildServerSavePath();
});

// File download methods
function parseDownloadUrl(value: string): URL | null {
  try {
    const parsedUrl = new URL(value.trim());
    if (!["http:", "https:"].includes(parsedUrl.protocol) || !parsedUrl.hostname) return null;
    return parsedUrl;
  } catch {
    return null;
  }
}

function parseFilenameFromUrl(value: string): string | null {
  const parsedUrl = parseDownloadUrl(value);
  if (!parsedUrl) return null;

  const encodedFilename = parsedUrl.pathname.match(/\/([^/]+)\/*$/)?.[1];
  if (!encodedFilename) return null;

  try {
    const decodedFilename = decodeURIComponent(encodedFilename);
    // 解码结果若包含路径分隔符，则保留编码形式，避免生成不安全的文件名。
    return /[\\/]/.test(decodedFilename) ? encodedFilename : decodedFilename;
  } catch {
    return encodedFilename;
  }
}

function handleUrlChange(value: string) {
  url.value = value;
  const parsedFilename = parseFilenameFromUrl(value);
  if (parsedFilename) filename.value = parsedFilename;
}

async function pickFileFolder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) savePath.value = result;
  } catch (e) {
    console.error("Pick file error:", e);
  }
}

function validateThreadCount(value: string): ThreadCountValidationResult {
  const normalized = value.trim();
  if (!normalized) return { valid: false, error: "empty" };
  if (!/^-?\d+$/.test(normalized)) return { valid: false, error: "invalid" };
  if (!/^[1-9]\d*$/.test(normalized)) return { valid: false, error: "non_positive" };

  const parsed = Number(normalized);
  if (parsed > MAX_DOWNLOAD_THREADS) return { valid: false, error: "too_large" };

  return { valid: true, value: parsed };
}

function checkThreadCount(value = threadCount.value) {
  const result = validateThreadCount(value);
  if (result.valid) return true;

  const messageKey = {
    empty: "download-file.thread_count_empty",
    invalid: "download-file.thread_count_invalid",
    non_positive: "download-file.thread_count_positive",
    too_large: "download-file.thread_count_too_big",
  }[result.error];
  toast.error(i18n.t(messageKey));
  return false;
}

function handleThreadCountChange(value: string) {
  threadCount.value = value;
  if (validateThreadCount(value).valid) threadCountInvalid.value = false;
}

function handleServerThreadCountChange(value: string) {
  serverThreadCount.value = value;
  if (validateThreadCount(value).valid) serverThreadCountInvalid.value = false;
}

function validateFileThreadCount() {
  threadCountInvalid.value = !checkThreadCount(threadCount.value);
}

function validateServerThreadCount() {
  serverThreadCountInvalid.value = !checkThreadCount(serverThreadCount.value);
}

// Server download methods
async function loadServerTypes() {
  loadingTypes.value = true;
  try {
    const types = await downloadServerApi.getServerTypes();
    serverTypes.value = types;
    if (types.length > 0) selectedType.value = types[0];
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    loadingTypes.value = false;
  }
}

// 修复：默认选中 原始数组最后一个元素（最新版本）
async function loadVersionsByType(serverType: string) {
  if (!serverType) return;
  loadingVersions.value = true;
  versions.value = [];
  selectedVersion.value = "";
  info.value = null;

  try {
    const list = await downloadServerApi.getVersionsByType(serverType);
    versions.value = list;
    // 核心修复：后端返回数组升序，最后一个 = 最新版本
    if (list.length > 0) selectedVersion.value = list[list.length - 1];
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    loadingVersions.value = false;
  }
}

async function loadDownloadInfo(serverType: string, version: string) {
  if (!serverType || !version) return;
  loadingInfo.value = true;
  info.value = null;
  serverFilename.value = "server.jar";

  try {
    const result = await downloadServerApi.getDownloadInfo(serverType, version);
    info.value = result;
    serverFilename.value = result.fileName;
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    loadingInfo.value = false;
  }
}

async function pickServerFolder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) serverSaveDir.value = result;
  } catch (e) {
    toast.error(handleError(e));
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
    await downloadStore.cancelTask();
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    stopLoading();
  }
}

async function handleFileDownload() {
  if (combinedLoading.value) return;

  if (!checkThreadCount()) return;
  const threadCountValue = threadCount.value.trim();
  const threadCountInt = parseInt(threadCountValue, 10);

  startLoading();

  const normalizedSavePath = savePath.value.replace(/\\/g, "/");
  const fullSavePath = normalizedSavePath + "/" + filename.value;

  try {
    await downloadStore.startTask(
      { url: url.value, save_path: fullSavePath, thread_count: threadCountInt },
      { filename: filename.value, savePath: fullSavePath, origin: "file" },
    );

    if (taskError.value) toast.error(taskError.value);
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    stopLoading();
  }
}

async function handleServerDownload() {
  if (!info.value || !checkThreadCount(serverThreadCount.value)) return;
  if (!canServerDownload.value) return;

  startLoading();

  const targetPath = buildServerSavePath();

  try {
    await downloadStore.startTask(
      {
        url: info.value.url,
        save_path: targetPath,
        thread_count: parseInt(serverThreadCount.value, 10),
      },
      { filename: serverFilename.value, savePath: targetPath, origin: "server" },
    );

    if (taskError.value) {
      toast.error(taskError.value);
    }
  } catch (e) {
    toast.error(handleError(e));
  } finally {
    stopLoading();
  }
}

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
  if (newError) toast.error(newError);
});

onMounted(() => {
  loadServerTypes();
});
</script>

<template>
  <div class="download-view animate-stagger-in">
    <div class="download-cards">
      <cmz-card :title="i18n.t('downloadServerView.title')">
        <DownloadServerForm
          :server-type-options="serverTypeOptions"
          :version-options="versionOptions"
          :selected-type="selectedType"
          :selected-version="selectedVersion"
          :filename="serverFilename"
          :save-dir="serverSaveDir"
          :thread-count="serverThreadCount"
          :thread-count-invalid="serverThreadCountInvalid"
          :loading-types="loadingTypes"
          :loading-versions="loadingVersions"
          :is-downloading="isDownloading"
          :save-path-preview="savePathPreview"
          :info-url="info?.url"
          @update:selected-type="selectedType = $event"
          @update:selected-version="selectedVersion = $event"
          @update:filename="serverFilename = $event"
          @update:save-dir="serverSaveDir = $event"
          @update:thread-count="handleServerThreadCountChange"
          @pickFolder="pickServerFolder"
          @checkThreadCount="validateServerThreadCount"
        />
        <div class="card-actions">
          <cmz-button
            :variant="isFinished && taskOriginTab === 'server' && !taskError ? 'solid' : undefined"
            :color="isFinished && taskOriginTab === 'server' && !taskError ? '#22c55e' : undefined"
            :disabled="!canServerDownload"
            @click="handleServerDownload"
            :loading="isDownloading && taskOriginTab === 'server'"
          >
            {{ serverDownloadButtonLabel }}
          </cmz-button>
          <cmz-button
            variant="outline"
            :disabled="!canGoCreate"
            @click="gotoCreatePage(buildServerSavePath())"
          >
            {{ i18n.t("downloadServerView.actions.goCreatePage") }}
          </cmz-button>
        </div>
      </cmz-card>

      <cmz-card :title="i18n.t('download-file.title')">
        <DownloadForm
          :url="url"
          :save-path="savePath"
          :filename="filename"
          :thread-count="threadCount"
          :thread-count-invalid="threadCountInvalid"
          :is-downloading="isDownloading"
          @update:url="handleUrlChange"
          @update:save-path="savePath = $event"
          @update:filename="filename = $event"
          @update:thread-count="handleThreadCountChange"
          @pickFolder="pickFileFolder"
          @checkThreadCount="validateFileThreadCount"
        />
        <div class="card-actions">
          <cmz-button
            :variant="isFinished && taskOriginTab === 'file' && !taskError ? 'solid' : undefined"
            :color="isFinished && taskOriginTab === 'file' && !taskError ? '#22c55e' : undefined"
            :disabled="!canFileDownload"
            @click="handleFileDownload"
            :loading="isDownloading && taskOriginTab === 'file'"
          >
            {{ fileDownloadButtonLabel }}
          </cmz-button>
          <cmz-button variant="outline" @click="cancelDownload">
            {{ i18n.t("download-file.cancel") }}
          </cmz-button>
        </div>
      </cmz-card>
    </div>
  </div>
</template>

<style scoped>
.download-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

.download-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-lg);
}

.download-cards .card-actions {
  display: flex;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-md);
}

@media (max-width: 780px) {
  .download-cards {
    grid-template-columns: 1fr;
  }
}
</style>
