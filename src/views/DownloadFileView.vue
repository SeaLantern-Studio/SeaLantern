<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRouter } from "vue-router";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import { useMessage } from "@composables/useMessage";
import { useLoading } from "@composables/useAsync";
import { systemApi } from "@api/system";
import { downloadApi } from "@api/downloader";
import DownloadForm from "@components/views/download/DownloadForm.vue";
import DownloadProgress from "@components/views/download/DownloadProgress.vue";

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

const isDownloading = computed(() => taskInfo.id !== "" && !taskInfo.isFinished);
const combinedLoading = computed(() => submitting.value || isDownloading.value);

const canDownload = computed(() => {
  if (
    isDownloading.value ||
    url.value.trim() === "" ||
    savePath.value.trim() === "" ||
    filename.value.trim() === "" ||
    threadCount.value.trim() === ""
  ) {
    return false;
  }

  // 验证URL格式
  try {
    new URL(url.value.trim());
    return true;
  } catch {
    return false;
  }
});

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

function checkFilename() {
  // 文件名输入时不需要特殊处理
}

async function pickFloder() {
  try {
    const result = await systemApi.pickFolder();
    if (result) savePath.value = result;
  } catch (e) {
    console.error("Pick file error:", e);
  }
}

const statusLabel = computed(() => {
  if (taskError.value) return i18n.t("download-file.failed");
  if (taskInfo.isFinished) return i18n.t("download-file.completed");
  return i18n.t("download-file.downloading");
});

async function handleDownload() {
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
      <button class="error-close" @click="clearError">x</button>
    </div>

    <SLCard :title="i18n.t('download-file.title')">
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
        @checkFilename="checkFilename()"
        @pickFolder="pickFloder"
      />
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
        :disabled="!canDownload"
      >
        {{ isDownloading ? i18n.t("download-file.downloading") : i18n.t("download-file.download") }}
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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
