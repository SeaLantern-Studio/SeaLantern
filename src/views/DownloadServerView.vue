<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import DownloadProgress from "@components/views/download/DownloadProgress.vue";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { downloadApi, downloadServerApi, type DownloadLink } from "@api/downloader";
import { systemApi } from "@api/system";
import { useCreateServerDraftStore } from "@stores/createServerDraft.ts";
import { i18n } from "@language";

const createServerDraftStore = useCreateServerDraftStore();
const router = useRouter();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: submitting, start: startLoading, stop: stopLoading } = useLoading();

const {
  taskInfo,
  start: startTask,
  reset: resetTask,
  errorMessage: taskError,
} = downloadApi.useDownload();

const serverTypes = ref<string[]>([]);
const versions = ref<string[]>([]);

const selectedType = ref("");
const selectedVersion = ref("");

const serverTypeOptions = computed(() =>
  serverTypes.value.map((type) => ({ label: type, value: type })),
);

const versionOptions = computed(() => versions.value.map((v) => ({ label: v, value: v })));

const info = ref<DownloadLink | null>(null);

const saveDir = ref("");
const filename = ref("server.jar");
const threadCount = ref("32");

const loadingTypes = ref(false);
const loadingVersions = ref(false);
const loadingInfo = ref(false);

const isDownloading = computed(() => taskInfo.id !== "" && !taskInfo.isFinished);
const loadingAny = computed(() => loadingTypes.value || loadingVersions.value || loadingInfo.value);
const combinedLoading = computed(() => submitting.value || isDownloading.value || loadingAny.value);

const statusLabel = computed(() => {
  if (taskError.value) return i18n.t("downloadServerView.status.failed");
  if (taskInfo.isFinished) return i18n.t("downloadServerView.status.finished");
  return i18n.t("downloadServerView.status.downloading");
});

const canDownload = computed(() => {
  if (combinedLoading.value) return false;
  if (!selectedType.value || !selectedVersion.value) return false;
  if (!info.value?.url) return false;
  if (!saveDir.value.trim() || !filename.value.trim()) return false;
  return /^[1-9]\d*$/.test(threadCount.value.trim());
});

const canGoCreate = computed(() => {
  return taskInfo.isFinished && !taskError.value;
});

const savePathPreview = computed(() => {
  if (!saveDir.value.trim() || !filename.value.trim()) return "";
  return buildSavePath();
});

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
    if (list.length > 0) selectedVersion.value = list[0];
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
  filename.value = "server.jar";

  try {
    const result = await downloadServerApi.getDownloadInfo(serverType, version);
    info.value = result;
    filename.value = result.fileName;
  } catch (e) {
    showError(String(e));
  } finally {
    loadingInfo.value = false;
  }
}

async function pickFolder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) saveDir.value = result;
  } catch (e) {
    showError(String(e));
  }
}

function buildSavePath() {
  const dir = saveDir.value.replace(/[\\/]+$/, "");
  const file = filename.value.replace(/^[\\/]+/, "");
  return `${dir}/${file}`;
}

function gotoCreatePage(sourcePath: string) {
  createServerDraftStore.setDraft({
    sourcePath: sourcePath,
    sourceType: "archive",
  });
  router.push("/create");
}

async function handleDownload() {
  if (!canDownload.value || !info.value) return;

  clearError();
  resetTask();
  startLoading();

  const targetPath = buildSavePath();

  try {
    await startTask({
      url: info.value.url,
      savePath: targetPath,
      threadCount: parseInt(threadCount.value, 10),
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

    <SLCard :title="i18n.t('downloadServerView.title')">
      <div class="form-grid">
        <div class="field">
          <div class="label-row">
            <label>{{ i18n.t("downloadServerView.form.type") }}</label>
          </div>
          <SLSelect
            v-model="selectedType"
            :options="serverTypeOptions"
            :placeholder="i18n.t('downloadServerView.form.typePlaceholder')"
            :disabled="loadingTypes || isDownloading"
            :loading="loadingTypes"
          />
        </div>

        <div class="field">
          <div class="label-row">
            <label>{{ i18n.t("downloadServerView.form.version") }}</label>
          </div>
          <SLSelect
            v-model="selectedVersion"
            :options="versionOptions"
            :placeholder="i18n.t('downloadServerView.form.versionPlaceholder')"
            :disabled="loadingVersions || !selectedType || isDownloading"
            :loading="loadingVersions"
          />
        </div>

        <div class="field field-full">
          <label>{{ i18n.t("downloadServerView.form.fileName") }}</label>
          <input
            v-model="filename"
            type="text"
            :placeholder="i18n.t('downloadServerView.form.fileNamePlaceholder')"
            :disabled="isDownloading"
          />
        </div>

        <div class="field field-full">
          <label>{{ i18n.t("downloadServerView.form.saveDir") }}</label>
          <div class="path-row">
            <input
              v-model="saveDir"
              type="text"
              :placeholder="i18n.t('downloadServerView.form.saveDirPlaceholder')"
              :disabled="isDownloading"
            />
            <SLButton variant="secondary" size="md" @click="pickFolder" :disabled="isDownloading">
              {{ i18n.t("downloadServerView.actions.pickFolder") }}
            </SLButton>
          </div>
        </div>

        <div class="field">
          <label>{{ i18n.t("downloadServerView.form.threadCount") }}</label>
          <input
            v-model="threadCount"
            type="text"
            :placeholder="i18n.t('downloadServerView.form.threadCountPlaceholder')"
            :disabled="isDownloading"
          />
        </div>

        <div class="field">
          <p v-if="savePathPreview" class="preview">
            {{ i18n.t("downloadServerView.preview.saveTo") }}{{ savePathPreview }}
          </p>
          <p v-if="info?.url" class="preview">
            {{ i18n.t("downloadServerView.preview.url") }}{{ info.url }}
          </p>
        </div>
      </div>
    </SLCard>

    <div class="create-actions">
      <SLButton variant="secondary" size="lg" @click="router.push('/')">
        {{ i18n.t("downloadServerView.actions.cancel") }}
      </SLButton>
      <SLButton variant="primary" size="lg" :disabled="!canDownload" @click="handleDownload">
        {{
          isDownloading
            ? i18n.t("downloadServerView.actions.downloading")
            : i18n.t("downloadServerView.actions.startDownload")
        }}
      </SLButton>
      <SLButton
        variant="primary"
        size="lg"
        :disabled="!canGoCreate"
        @click="gotoCreatePage(buildSavePath())"
      >
        {{ i18n.t("downloadServerView.actions.goCreatePage") }}
      </SLButton>
    </div>

    <Transition name="fade">
      <div v-if="taskInfo.id" class="bottom-progress-area">
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

.form-grid {
  display: grid;
  gap: var(--sl-space-md);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-full {
  grid-column: 1 / -1;
}

.label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-sm);
}

.field label {
  font-size: 0.85rem;
  color: var(--sl-text-tertiary);
}

.loading-text {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

.field input,
.field select {
  width: 100%;
  height: 40px;
  padding: 0 10px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-surface);
  color: var(--sl-text-primary);
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease;
}

.field input:focus,
.field select:focus {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.14);
}

.field input[readonly] {
  opacity: 0.9;
}

.path-row {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: var(--sl-space-sm);
}

.preview {
  margin: 0;
  font-size: 12px;
  color: var(--sl-text-tertiary);
  word-break: break-all;
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

@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr;
  }

  .path-row {
    grid-template-columns: 1fr;
  }
}
</style>
