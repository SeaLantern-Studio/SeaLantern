<template>
  <div class="java-downloader-container">
    <div class="java-downloader-header">
      <!-- Left Side: Label & Desc -->
      <div class="sl-setting-info">
        <span class="sl-setting-label">
          {{ i18n.t("settings.java_download") }}
        </span>
        <span class="sl-setting-desc">
          {{ i18n.t("settings.java_download_desc") }}
        </span>
      </div>

      <!-- Right Side: Interaction Area -->
      <div class="java-downloader-actions">
        <!-- Idle State -->
        <template v-if="!isDownloading && !isExtracting && !successMessage">
          <div class="download-setting-div">
            <SLSelect
              v-model="selectedVersion"
              :options="versionOptions"
              :disabled="loadingUrl"
              size="sm"
            />
            <SLButton
              variant="primary"
              class="download-button"
              :loading="loadingUrl"
              @click="startDownload"
            >
              {{ downloadButtonText }}
            </SLButton>
          </div>
        </template>

        <!-- Downloading State -->
        <template v-else-if="isDownloading || isExtracting">
          <div class="downloading-state">
            <div class="progress-container">
              <div class="status-text">
                <span>{{ statusMessage }}</span>
                <span class="progress-percentage">{{
                  isExtracting ? "" : `${progress.toFixed(0)}%`
                }}</span>
              </div>
              <div class="progress-bar-container">
                <div
                  class="progress-bar"
                  :class="{ 'indeterminate-progress': isExtracting }"
                  :style="{ width: isExtracting ? '100%' : `${progress}%` }"
                ></div>
              </div>
            </div>
            <SLButton
              size="sm"
              variant="ghost"
              class="cancel-button"
              title="Cancel"
              @click="cancelDownload"
            >
              <X :size="16" :stroke-width="2" />
            </SLButton>
          </div>
        </template>

        <!-- Success State -->
        <template v-else-if="successMessage">
          <div class="success-state">
            <div class="success-message">
              <CheckCircle :size="16" />
              <span>{{ i18n.t("settings.java_install_success").replace(":", "") }}</span>
            </div>
            <SLButton size="sm" variant="ghost" @click="resetState">OK</SLButton>
          </div>
        </template>
      </div>
    </div>

    <!-- Error Message (Full Width below) -->
    <div v-if="errorMessage" class="error-message">
      <div class="error-content">
        <AlertCircle class="error-icon" :size="16" />
        <span>{{ errorMessage }}</span>
      </div>
      <SLButton size="sm" variant="ghost" @click="resetState">
        {{ i18n.t("common.close_notification") }}
      </SLButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import { i18n } from "@language";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { javaApi } from "@api/java";
import SLButton from "@components/common/SLButton.vue";
import SLSelect from "@components/common/SLSelect.vue";
import { X, CheckCircle, AlertCircle } from "lucide-vue-next";

const emit = defineEmits(["installed"]);

const selectedVersion = ref("17");
const isDownloading = ref(false);
const isExtracting = ref(false);
const loadingUrl = ref(false);
const progress = ref(0);
const statusMessage = ref("");
const errorMessage = ref("");
const successMessage = ref("");
const installedPath = ref("");
const unlistenProgress = ref<UnlistenFn | null>(null);

const versionOptions = computed(() => [
  { label: "Java 8 (LTS)", value: "8" },
  { label: "Java 17 (LTS)", value: "17" },
  { label: "Java 21 (LTS)", value: "21" },
  { label: "Java 25 (LTS)", value: "25" },
]);

const downloadButtonText = computed(() => {
  return i18n.t("settings.java_download_btn", { version: selectedVersion.value });
});

const resetState = () => {
  errorMessage.value = "";
  successMessage.value = "";
  isDownloading.value = false;
  isExtracting.value = false;
  progress.value = 0;
};

const getDownloadUrl = (version: string): string => {
  // Construct URL for Adoptium API
  const baseUrl = "https://api.adoptium.net/v3/binary/latest";
  const releaseType = "ga";

  // Detect OS and Arch
  let os = "windows";
  // Adoptium API uses 'mac' for macOS
  if (navigator.userAgent.indexOf("Mac") !== -1) os = "mac";
  if (navigator.userAgent.indexOf("Linux") !== -1) os = "linux";

  let arch = "x64";
  if (navigator.userAgent.indexOf("aarch64") !== -1 || navigator.userAgent.indexOf("arm64") !== -1)
    arch = "aarch64";

  return `${baseUrl}/${version}/${releaseType}/${os}/${arch}/jdk/hotspot/normal/eclipse`;
};

const cancelDownload = async () => {
  try {
    await javaApi.cancelInstall();
    // Reset state immediately for better UX
    isDownloading.value = false;
    isExtracting.value = false;
    loadingUrl.value = false;
    progress.value = 0;
    statusMessage.value = "";

    if (unlistenProgress.value) {
      unlistenProgress.value();
      unlistenProgress.value = null;
    }
  } catch (e) {
    console.error("Cancellation failed:", e);
  }
};

const startDownload = async () => {
  resetState();
  loadingUrl.value = true;

  try {
    const url = getDownloadUrl(selectedVersion.value);
    const versionName = `jdk-${selectedVersion.value}`;
    loadingUrl.value = false;
    isDownloading.value = true;
    progress.value = 0;
    statusMessage.value = i18n.t("settings.java_installing");

    if (unlistenProgress.value) unlistenProgress.value();

    unlistenProgress.value = await listen("java-install-progress", (event: any) => {
      const payload = event.payload as {
        state: string;
        progress: number;
        total: number;
        message: string;
      };
      statusMessage.value = payload.message;

      if (payload.state === "extracting") {
        isExtracting.value = true;
        progress.value = 100; // Force full bar or let indeterminate animation take over
      } else if (payload.state === "downloading") {
        isExtracting.value = false;
        if (payload.total > 0) {
          progress.value = (payload.progress / payload.total) * 100;
        }
      } else if (payload.state === "finished") {
        progress.value = 100;
        isExtracting.value = false;
      }
    });

    const resultPath = await javaApi.installJava(url, versionName);

    installedPath.value = resultPath;
    successMessage.value = "Success"; // Just a flag, text is in template
    emit("installed", resultPath);
  } catch (e: any) {
    console.error(e);
    isDownloading.value = false;
    isExtracting.value = false;
    errorMessage.value =
      i18n.t("settings.java_install_failed") + (typeof e === "string" ? e : e.message);
  } finally {
    isDownloading.value = false;
    isExtracting.value = false;
    if (unlistenProgress.value) {
      unlistenProgress.value();
      unlistenProgress.value = null;
    }
  }
};

onUnmounted(() => {
  if (unlistenProgress.value) unlistenProgress.value();
});
</script>

<style scoped>
.download-button {
  padding: 0.5rem 2rem;
}

.java-downloader-container {
  width: 100%;
}

.java-downloader-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.25rem 0;
}

.java-downloader-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
}

.download-setting-div {
  display: flex;
  flex-direction: row;
  gap: 10px;
  margin-top: 10px;
}

.downloading-state {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.progress-container {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.25rem;
  width: 10rem;
}

.status-text {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--sl-text-primary);
}

.progress-percentage {
  font-family: var(--sl-font-mono);
  opacity: 0.7;
}

.progress-bar-container {
  width: 100%;
  height: 0.375rem;
  background-color: var(--sl-border);
  border-radius: 9999px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background-color: var(--sl-primary);
  transition: all 0.3s ease-out;
}

@keyframes indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.indeterminate-progress {
  position: relative;
  overflow: hidden;
}

.indeterminate-progress::after {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: 50%;
  background: rgba(255, 255, 255, 0.3);
  animation: indeterminate 1.5s infinite linear;
}

.cancel-button {
  padding: 0.375rem !important;
  color: var(--sl-text-tertiary);
}

.cancel-button:hover {
  color: var(--sl-error);
}

.success-state {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  animation: fade-in 0.3s ease;
}

.success-message {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--sl-success);
  font-size: 0.875rem;
  font-weight: 500;
}

.error-message {
  margin-top: 0.5rem;
  padding: 0.75rem;
  background-color: #fef2f2;
  color: var(--sl-error);
  font-size: 0.875rem;
  border-radius: var(--sl-radius-md);
  border: 1px solid #fee2e2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  animation: fade-in 0.3s ease;
}

[data-theme="dark"] .error-message {
  background-color: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.3);
}

.error-content {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.error-icon {
  flex-shrink: 0;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(0.25rem);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
