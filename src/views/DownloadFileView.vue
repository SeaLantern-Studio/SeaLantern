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
import {SLProgress} from "@src/components";

const router = useRouter();
const { error: errorMsg, showError, clearError } = useMessage();
const { loading: submitting, start: startLoading, stop: stopLoading } = useLoading();

const { taskInfo, start: startTask, reset: resetTask, errorMessage: taskError } = downloadApi.useDownload();

const url = ref("");
const savePath = ref("");
const filename = ref("");
const threadCount = ref("32");

const isUrlValid = ref(false);

const isDownloading = computed(() => taskInfo.id !== "" && !taskInfo.isFinished);
const combinedLoading = computed(() => submitting.value || isDownloading.value);

function checkUrl(event: { target: { value: any; }; }) {
  const url = event.target.value;
  try {
    const urlObj = new URL(url);
    const pathName = urlObj.pathname;
    const segments = pathName.split("/");
    if (segments.length > 1) {
      filename.value = segments[segments.length - 1];
      isUrlValid.value = filename.value.length > 0;
    }
  } catch {
    isUrlValid.value = false;
  }
}

function checkFilename(event: { target: { value: any; }; }) {
  const filename = event.target.value;
  isUrlValid.value = filename.length > 0
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

  const threadCountValue = threadCount.value;
  if (threadCountValue == "") {
    showError("线程数不能为空");
    return;
  }
  if (!/^-?\d+$/.test(threadCountValue)) {
    showError("字符不合法");
    return;
  }
  if (!/^[1-9]\d*$/.test(threadCountValue)) {
    showError("线程数必须是一个正整数");
    return;
  }
  const threadCountInt = parseInt(threadCountValue, 10);


  clearError();
  resetTask();
  startLoading();

  try {
    await startTask({
      url: url.value,
      savePath: savePath.value + "/" + filename.value,
      threadCount: threadCountInt,
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
        <SLInput :label="i18n.t('download-file.url')" v-model="url" :disabled="isDownloading" @input="checkUrl" />
        <SLInput :label="i18n.t('download-file.save_folder')" v-model="savePath" :disabled="isDownloading">
          <template #suffix>
            <button class="pick-btn" @click="pickFloder" :disabled="isDownloading">
              {{ i18n.t("download-file.browse") }}
            </button>
          </template>
        </SLInput>
        <SLInput :label="i18n.t('download-file.filename')" v-model="filename" :disabled="isDownloading" @input="checkFilename" />
        <SLInput :label="i18n.t('download-file.thread_count')" v-model="threadCount" :disabled="isDownloading" />
      </div>
    </SLCard>

    <div class="create-actions">
      <SLButton variant="secondary" size="lg" @click="router.push('/')" >
        {{ i18n.t("download-file.cancel") }}
      </SLButton>
      <SLButton variant="primary" size="lg" :loading="combinedLoading" @click="handleDownload" :disabled="isDownloading || !isUrlValid">
        {{ isDownloading ? i18n.t("download-file.downloading") : i18n.t("download-file.download") }}
      </SLButton>
    </div>

    <Transition name="fade">
      <div v-if="taskInfo.id" class="bottom-progress-area">
        <div class="progress-wrapper">
          <SLProgress
              :value="taskInfo.progress"
              :variant="taskError ? 'error' : (taskInfo.isFinished ? 'success' : 'primary')"
              :label="statusLabel"
          />
          <div class="progress-footer">
            <span class="size-text">{{ formatSize(taskInfo.downloaded) }} / {{ formatSize(taskInfo.totalSize) }}</span>
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
.pick-btn:hover { background: var(--sl-primary); color: white; }
.pick-btn:disabled { filter: grayscale(1); opacity: 0.5; cursor: not-allowed; }
.create-actions {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
  margin-top: var(--sl-space-md);
}
.animate-fade-in-up { animation: fadeInUp 0.4s ease-out; }
@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
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