<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import { useRouter, useRoute } from "vue-router";
import { downloadApi } from "@api/downloader";
import { serverApi } from "@api/server";
import { getPaperBuildInfo } from "@utils/networkapi/paperapi";
import { getServerDownloadUrl } from "@utils/networkapi/mslapi.ts";

const router = useRouter();
const route = useRoute();

const serverName = ref<string>("");
const version = ref<string>("");
const build = ref<string>("");
const source = ref<string>("official");
const sha256 = ref<string>("");
const serverNameInput = ref<string>("");
const serverPath = ref<string>("");
const maxMemory = ref<number>(0);
const minMemory = ref<number>(0);
const javaPath = ref<string>("");

const currentStep = ref<number>(1);
const stepStatus = ref<{ [key: number]: "pending" | "running" | "completed" | "error" }>({
  1: "pending",
  2: "pending",
  3: "pending",
});

const downloadProgress = ref<number>(0);
const downloadSpeed = ref<string>("0 KB/s");
const downloadedSize = ref<string>("0 MB");
const totalSize = ref<string>("0 MB");
const errorMessage = ref<string>("");

const { taskInfo, start: startDownload, stop: stopDownload } = downloadApi.useDownload();

const steps = [
  { id: 1, title: "准备下载", description: "获取文件信息" },
  { id: 2, title: "下载核心", description: `正在下载服务端核心文件` },
  { id: 3, title: "校验文件", description: "验证文件完整性" },
  { id: 4, title: "创建服务器", description: "正在创建服务器实例" },
  { id: 5, title: "完成", description: "服务器创建完成" },
];

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
};

const startProcess = async () => {
  let downloadedFilePath = "";
  let fileName = "";
  let downloadSha256 = "";

  try {
    currentStep.value = 1;
    stepStatus.value[1] = "running";

    let downloadUrl = "";

    if (source.value === "official") {
      const buildInfo = await getPaperBuildInfo(version.value, parseInt(build.value));
      fileName = buildInfo.downloads.application.name;
      downloadUrl = `https://api.papermc.io/v2/projects/paper/versions/${version.value}/builds/${build.value}/downloads/${fileName}`;
      downloadSha256 = buildInfo.downloads.application.sha256;
    } else {
      const downloadInfo = await getServerDownloadUrl(serverName.value, version.value, build.value);
      downloadUrl = downloadInfo.url;
      downloadSha256 = downloadInfo.sha256 || "";
      // 从 URL 中提取文件名
      fileName = downloadUrl.split("/").pop() || `${serverName.value}-${version.value}.jar`;
    }

    downloadedFilePath = `${serverPath.value}/${fileName}`;

    stepStatus.value[1] = "completed";
    currentStep.value = 2;
    stepStatus.value[2] = "running";

    await startDownload({
      url: downloadUrl,
      savePath: downloadedFilePath,
      threadCount: 16,
    });
  } catch (err) {
    stepStatus.value[currentStep.value] = "error";
    errorMessage.value = err instanceof Error ? err.message : "处理失败";
  }
};

let progressTimer: number | null = null;
let lastDownloaded = 0;

onMounted(() => {
  const serverParam = route.params.coreName as string;
  const versionParam = route.params.version as string;
  const buildParam = route.params.build as string;
  const sha256Param = route.query.sha256 as string;
  const nameParam = route.query.name as string;
  const pathParam = route.query.path as string;
  const maxMemParam = route.query.maxMemory as string;
  const minMemParam = route.query.minMemory as string;
  const javaParam = route.query.java as string;
  const sourceParam = route.query.source as string;

  if (serverParam && versionParam && buildParam && nameParam && pathParam && javaParam) {
    serverName.value = serverParam;
    version.value = versionParam;
    build.value = buildParam;
    sha256.value = sha256Param || "";
    serverNameInput.value = nameParam;
    serverPath.value = pathParam;
    maxMemory.value = parseInt(maxMemParam);
    minMemory.value = parseInt(minMemParam);
    javaPath.value = javaParam;
    source.value = sourceParam || "official";

    startProcess();
  }

  progressTimer = window.setInterval(async () => {
    if (taskInfo.status === "Downloading") {
      downloadProgress.value = taskInfo.progress;
      downloadedSize.value = formatBytes(taskInfo.downloaded);
      totalSize.value = formatBytes(taskInfo.totalSize);

      const speed = taskInfo.downloaded - lastDownloaded;
      downloadSpeed.value = `${formatBytes(speed * 1.25)}/s`;
      lastDownloaded = taskInfo.downloaded;
    }

    if (taskInfo.status === "Completed") {
      stepStatus.value[2] = "completed";
      currentStep.value = 3;
      stepStatus.value[3] = "running";
      downloadProgress.value = 100;

      if (progressTimer) {
        clearInterval(progressTimer);
        progressTimer = null;
      }

      try {
        let fileName = "";
        let fileSha256 = sha256.value;

        if (source.value === "official") {
          const buildInfo = await getPaperBuildInfo(version.value, parseInt(build.value));
          fileName = buildInfo.downloads.application.name;
          fileSha256 = buildInfo.downloads.application.sha256;
        } else {
          const downloadInfo = await getServerDownloadUrl(
            serverName.value,
            version.value,
            build.value,
          );
          fileName =
            downloadInfo.url.split("/").pop() || `${serverName.value}-${version.value}.jar`;
          fileSha256 = downloadInfo.sha256 || "";
        }

        const downloadedFilePath = `${serverPath.value}/${fileName}`;

        // 只有当有 SHA256 时才进行校验
        if (fileSha256) {
          const isValid = await serverApi.verifyFileSha256(downloadedFilePath, fileSha256);
          if (!isValid) {
            stepStatus.value[3] = "error";
            errorMessage.value = "文件校验失败，SHA256 不匹配";
            return;
          }
        }

        stepStatus.value[3] = "completed";
        currentStep.value = 4;
        stepStatus.value[4] = "running";

        await serverApi.create({
          name: serverNameInput.value,
          coreType: serverName.value.toLowerCase(),
          mcVersion: version.value,
          maxMemory: maxMemory.value,
          minMemory: minMemory.value,
          port: 25565,
          javaPath: javaPath.value,
          jarPath: downloadedFilePath,
          startupMode: "jar",
        });

        stepStatus.value[4] = "completed";
        currentStep.value = 5;
        stepStatus.value[5] = "completed";
      } catch (err) {
        stepStatus.value[currentStep.value] = "error";
        errorMessage.value = err instanceof Error ? err.message : "创建服务器失败";
      }
    }

    if (typeof taskInfo.status === "object" && "Error" in taskInfo.status) {
      stepStatus.value[2] = "error";
      errorMessage.value = taskInfo.status.Error;

      if (progressTimer) {
        clearInterval(progressTimer);
        progressTimer = null;
      }
    }
  }, 800);
});

onBeforeUnmount(() => {
  if (progressTimer) {
    clearInterval(progressTimer);
    progressTimer = null;
  }
  stopDownload();
});

const goToHome = () => {
  router.push("/");
};
</script>

<template>
  <div class="download-view">
    <div class="download-container">
      <h1 class="download-title">
        下载 {{ serverName.charAt(0).toUpperCase() + serverName.slice(1) }} {{ version }} #{{
          build
        }}
      </h1>
      <p class="download-subtitle">{{ serverNameInput }}</p>

      <div class="steps-container">
        <div
          v-for="step in steps"
          :key="step.id"
          class="step-item"
          :class="{
            'step-pending': stepStatus[step.id] === 'pending',
            'step-running': stepStatus[step.id] === 'running',
            'step-completed': stepStatus[step.id] === 'completed',
            'step-error': stepStatus[step.id] === 'error',
          }"
        >
          <div class="step-indicator">
            <div class="step-number">{{ step.id }}</div>
          </div>
          <div class="step-content">
            <h3 class="step-title">{{ step.title }}</h3>
            <p class="step-description">{{ step.description }}</p>
          </div>
        </div>
      </div>

      <div v-if="currentStep === 2" class="progress-section">
        <div class="progress-bar-container">
          <div class="progress-bar-track">
            <div class="progress-bar-fill" :style="{ width: `${downloadProgress}%` }">
              <div class="progress-ripple"></div>
            </div>
          </div>
        </div>

        <div class="progress-info">
          <div class="progress-text">
            <span class="progress-percent">{{ downloadProgress.toFixed(1) }}%</span>
            <span class="progress-size">{{ downloadedSize }} / {{ totalSize }}</span>
          </div>
          <div class="progress-speed">{{ downloadSpeed }}</div>
        </div>
      </div>

      <div v-if="errorMessage" class="error-message">
        <p>{{ errorMessage }}</p>
      </div>

      <div v-if="currentStep === 5 && stepStatus[5] === 'completed'" class="success-actions">
        <button class="success-button" @click="goToHome">返回首页</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.download-view {
  padding: var(--sl-space-xl);
  max-width: 800px;
  margin: 0 auto;
  animation: sl-fade-in-up var(--sl-transition-normal) forwards;
}

.download-container {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  padding: var(--sl-space-2xl);
}

.download-title {
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-xs) 0;
  text-align: center;
}

.download-subtitle {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-2xl) 0;
  text-align: center;
}

.steps-container {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-2xl);
}

.step-item {
  display: flex;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
  background: var(--sl-bg-secondary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  transition: all var(--sl-transition-fast) ease;
}

.step-pending {
  opacity: 0.5;
}

.step-running {
  border-color: var(--sl-primary);
  background: var(--sl-primary-bg);
}

.step-completed {
  border-color: var(--sl-success);
  background: var(--sl-success-bg);
}

.step-error {
  border-color: var(--sl-error);
  background: var(--sl-error-bg);
}

.step-indicator {
  flex-shrink: 0;
}

.step-number {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-surface);
  border: 2px solid var(--sl-border);
  border-radius: 50%;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  transition: all var(--sl-transition-fast) ease;
}

.step-running .step-number {
  border-color: var(--sl-primary);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  animation: pulse 2s ease-in-out infinite;
}

.step-completed .step-number {
  border-color: var(--sl-success);
  background: var(--sl-success);
  color: var(--sl-text-inverse);
}

.step-error .step-number {
  border-color: var(--sl-error);
  background: var(--sl-error);
  color: var(--sl-text-inverse);
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

.step-content {
  flex: 1;
}

.step-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-xs) 0;
}

.step-description {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  margin: 0;
}

.progress-section {
  margin-bottom: var(--sl-space-xl);
}

.progress-bar-container {
  margin-bottom: var(--sl-space-md);
}

.progress-bar-track {
  position: relative;
  height: 24px;
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-full);
  overflow: hidden;
}

.progress-bar-fill {
  position: relative;
  height: 100%;
  background: linear-gradient(90deg, var(--sl-primary), var(--sl-primary-light));
  border-radius: var(--sl-radius-full);
  transition: width 0.3s ease;
  overflow: hidden;
}

.progress-ripple {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  animation: ripple 1s linear infinite;
}

@keyframes ripple {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-text {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.progress-percent {
  font-size: 1.5rem;
  font-weight: 700;
  font-family: var(--sl-font-mono);
  color: var(--sl-text-primary);
}

.progress-size {
  font-size: 0.875rem;
  font-family: var(--sl-font-mono);
  color: var(--sl-text-secondary);
}

.progress-speed {
  font-size: 1.125rem;
  font-weight: 600;
  font-family: var(--sl-font-mono);
  color: var(--sl-primary);
}

.error-message {
  padding: var(--sl-space-md);
  background: var(--sl-error-bg);
  border: 1px solid var(--sl-error);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-xl);
}

.error-message p {
  color: var(--sl-error);
  margin: 0;
  font-size: 0.9375rem;
}

.success-actions {
  display: flex;
  justify-content: center;
}

.success-button {
  padding: var(--sl-space-md) var(--sl-space-2xl);
  background: var(--sl-primary);
  color: var(--sl-text-inverse);
  border: none;
  border-radius: var(--sl-radius-md);
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color var(--sl-transition-fast) ease;
}

.success-button:hover {
  background: var(--sl-primary-dark);
}
</style>
