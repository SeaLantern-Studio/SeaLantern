<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { systemApi } from "@src/api";
import { downloadApi } from "@api/downloader.ts";
import { SLProgress } from "@src/components";

const router = useRouter();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: submitting, start: startLoading, stop: stopLoading } = useLoading();

const {
  taskInfo,
  start: startTask,
  reset: resetTask,
  errorMessage: taskError,
} = downloadApi.useDownload();

const url = ref("");
const savePath = ref("");
const filename = ref("");
const threadCount = ref("32");
const isFilenameManuallyEdited = ref(false);

type DownloadValidationKey =
  | "invalid_input"
  | "url_invalid"
  | "save_folder_required"
  | "filename_required"
  | "filename_invalid"
  | "thread_count_required"
  | "thread_count_invalid_chars"
  | "thread_count_positive_integer";

type ThreadCountValidationResult =
  | { ok: true; value: number }
  | { ok: false; errorKey: DownloadValidationKey };

function translateDownloadValidationError(key: DownloadValidationKey): string {
  return i18n.t(`download-file.${key}`);
}

function showDownloadValidationError(key: DownloadValidationKey): void {
  showError(translateDownloadValidationError(key));
}

function getFilenameValidationKey(name: string): DownloadValidationKey | null {
  if (!name) {
    return "filename_required";
  }
  if (/[\\/]/.test(name)) {
    return "filename_invalid";
  }
  return null;
}

function getFormValidationKey(): DownloadValidationKey | null {
  if (!isUrlValid.value) {
    return "url_invalid";
  }
  if (!isSavePathValid.value) {
    return "save_folder_required";
  }
  return filenameErrorKey.value;
}

function validateThreadCount(rawValue: string): ThreadCountValidationResult {
  const value = rawValue.trim();
  if (!value) {
    return { ok: false, errorKey: "thread_count_required" };
  }
  if (!/^\d+$/.test(value)) {
    return { ok: false, errorKey: "thread_count_invalid_chars" };
  }
  if (!/^[1-9]\d*$/.test(value)) {
    return { ok: false, errorKey: "thread_count_positive_integer" };
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return { ok: false, errorKey: "thread_count_positive_integer" };
  }
  return { ok: true, value: parsed };
}

const isUrlValid = computed(() => {
  try {
    const parsed = new URL(url.value.trim());
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
});
const filenameErrorKey = computed(() => getFilenameValidationKey(filename.value.trim()));
const isFilenameValid = computed(() => !filenameErrorKey.value);
const filenameError = computed(() => {
  return filenameErrorKey.value ? translateDownloadValidationError(filenameErrorKey.value) : "";
});
const isSavePathValid = computed(() => savePath.value.trim().length > 0);
const isFormValid = computed(
  () => isUrlValid.value && isFilenameValid.value && isSavePathValid.value,
);

const isDownloading = computed(() => taskInfo.id !== "" && !taskInfo.isFinished);
const combinedLoading = computed(() => submitting.value || isDownloading.value);

function checkUrl(event: Event) {
  const input = event.target as HTMLInputElement | null;
  const rawUrl = input?.value?.trim() ?? "";
  try {
    const urlObj = new URL(rawUrl);
    const pathName = urlObj.pathname;
    const segments = pathName.split("/").filter(Boolean);
    if (segments.length > 0) {
      const candidateFileName = segments[segments.length - 1];
      const currentFilename = filename.value.trim();
      if (candidateFileName && (!isFilenameManuallyEdited.value || !currentFilename)) {
        filename.value = candidateFileName;
      }
    }
  } catch {
    // Keep current filename; URL validity is computed separately.
  }
}

function handleFilenameInput(value: string | number) {
  isFilenameManuallyEdited.value = true;
  filename.value = String(value);
}

async function pickFloder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) savePath.value = result;
  } catch (e) {
    console.error("Pick file error:", e);
  }
}

const formatSize = (bytes: number) => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

const statusLabel = computed(() => {
  if (taskError.value) return i18n.t("download-file.failed");
  if (taskInfo.isFinished) return i18n.t("download-file.completed");
  return i18n.t("download-file.downloading");
});

async function handleDownload() {
  if (combinedLoading.value) return;
  const formErrorKey = getFormValidationKey();
  if (formErrorKey) {
    showDownloadValidationError(formErrorKey);
    return;
  }

  const threadCountValidation = validateThreadCount(threadCount.value);
  if (!threadCountValidation.ok) {
    showDownloadValidationError(threadCountValidation.errorKey);
    return;
  }

  clearError();
  resetTask();
  startLoading();

  try {
    await startTask({
      url: url.value,
      savePath: savePath.value + "/" + filename.value,
      threadCount: threadCountValidation.value,
    });

    if (taskError.value) showError(taskError.value);
  } catch (e) {
    showError(String(e));
  } finally {
    stopLoading();
  }
}

watch(taskError, (newError) => {
  if (newError) showError(newError);
});
</script>

<template>
  <div class="download-view animate-fade-in-up">
    <div v-if="errorMsg" class="error-banner">
      <span>{{ errorMsg }}</span>
      <button class="error-close" @click="clearError()">x</button>
    </div>

    <SLCard :title="i18n.t('download-file.title')">
      <div class="form-grid">
        <SLInput
          :label="i18n.t('download-file.url')"
          v-model="url"
          :disabled="isDownloading"
          @input="checkUrl"
        />
        <SLInput
          :label="i18n.t('download-file.save_folder')"
          v-model="savePath"
          :disabled="isDownloading"
        >
          <template #suffix>
            <button class="pick-btn" @click="pickFloder" :disabled="isDownloading">
              {{ i18n.t("download-file.browse") }}
            </button>
          </template>
        </SLInput>
        <SLInput
          :label="i18n.t('download-file.filename')"
          :model-value="filename"
          :disabled="isDownloading"
          @update:model-value="handleFilenameInput"
        />
        <p v-if="filenameError && !isDownloading" class="field-error">{{ filenameError }}</p>
        <SLInput
          :label="i18n.t('download-file.thread_count')"
          v-model="threadCount"
          :disabled="isDownloading"
        />
      </div>
    </SLCard>

    <div class="create-actions">
      <SLButton variant="secondary" size="lg" @click="router.push('/')">
        {{ i18n.t("download-file.cancel") }}
      </SLButton>
      <SLButton
        variant="primary"
        size="lg"
        :loading="combinedLoading"
        @click="handleDownload"
        :disabled="isDownloading || !isFormValid"
      >
        {{ isDownloading ? i18n.t("download-file.downloading") : i18n.t("download-file.download") }}
      </SLButton>
    </div>

    <Transition name="fade">
      <div v-if="taskInfo.id" class="bottom-progress-area">
        <div class="progress-wrapper">
          <SLProgress
            :value="taskInfo.progress"
            :variant="taskError ? 'error' : taskInfo.isFinished ? 'success' : 'primary'"
            :label="statusLabel"
          />
          <div class="progress-footer">
            <span class="size-text"
              >{{ formatSize(taskInfo.downloaded) }} / {{ formatSize(taskInfo.totalSize) }}</span
            >
            <span class="percent-text">{{ taskInfo.progress.toFixed(1) }}%</span>
          </div>
        </div>
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
  font-size: 0.875rem;
}
.error-close {
  color: var(--sl-error);
  font-weight: 600;
  cursor: pointer;
  background: none;
  border: none;
}
.form-grid {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
.field-error {
  margin: -6px 0 0;
  font-size: 12px;
  color: var(--sl-error);
}
.pick-btn {
  padding: 4px 12px;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  cursor: pointer;
  white-space: nowrap;
  border: none;
  transition: all var(--sl-transition-fast);
}
.pick-btn:hover {
  background: var(--sl-primary);
  color: white;
}
.pick-btn:disabled {
  filter: grayscale(1);
  opacity: 0.5;
  cursor: not-allowed;
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
  justify-content: center; /* 居中 */
  width: 100%;
}

.progress-wrapper {
  width: 100%;
  max-width: 560px; /* 略窄于卡片，更有层次感 */
  background: var(--sl-bg-secondary, #f9f9f9); /* 可选：给个淡淡的底色背景 */
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  border: 1px solid var(--sl-border-light, #eee);
}

.progress-footer {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-size: 0.75rem;
  color: var(--sl-text-secondary);
  font-family: var(--sl-font-mono, monospace), serif;
}
</style>
