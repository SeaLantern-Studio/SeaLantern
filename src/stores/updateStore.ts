import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { checkUpdate, type UpdateInfo } from "../api/update";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "downloaded"
  | "error"
  | "latest";

export const useUpdateStore = defineStore("update", () => {
  const status = ref<UpdateStatus>("idle");
  const updateInfo = ref<UpdateInfo | null>(null);
  const downloadProgress = ref(0);
  const downloadedFilePath = ref<string | null>(null);
  const errorMessage = ref<string | null>(null);
  const hasCheckedOnStartup = ref(false);
  const isUpdateModalVisible = ref(false);
  const hasStartedUpdateFlow = ref(false);

  const isUpdateAvailable = computed(
    () =>
      status.value === "available" ||
      status.value === "downloading" ||
      status.value === "downloaded" ||
      status.value === "installing",
  );
  const isDownloading = computed(() => status.value === "downloading");
  const isInstalling = computed(() => status.value === "installing");
  const isReadyToInstall = computed(() => status.value === "downloaded");

  // ===== 新增：判断是否是 AUR 更新 =====
  const isAurUpdate = computed(() => updateInfo.value?.source === "arch-aur");

  function reset() {
    status.value = "idle";
    updateInfo.value = null;
    downloadProgress.value = 0;
    downloadedFilePath.value = null;
    errorMessage.value = null;
    isUpdateModalVisible.value = false;
    hasStartedUpdateFlow.value = false;
  }

  async function checkForUpdate(silent = false): Promise<UpdateInfo | null> {
    if (status.value === "installing") {
      return updateInfo.value;
    }
    if (!silent) {
      status.value = "checking";
    }
    errorMessage.value = null;

    try {
      const info = await checkUpdate();
      if (info && info.has_update) {
        updateInfo.value = info;
        status.value = "available";
        downloadProgress.value = 0;
        downloadedFilePath.value = null;

        // 如果不是 AUR 更新，才显示下载模态框
        if (info.source !== "arch-aur") {
          isUpdateModalVisible.value = true;
        }

        hasStartedUpdateFlow.value = false;
        return info;
      } else {
        if (!silent) {
          status.value = "latest";
          setTimeout(() => {
            if (status.value === "latest") {
              status.value = "idle";
            }
          }, 3000);
        }
        updateInfo.value = null;
        downloadProgress.value = 0;
        downloadedFilePath.value = null;
        isUpdateModalVisible.value = false;
        hasStartedUpdateFlow.value = false;
        return null;
      }
    } catch (error) {
      errorMessage.value = String(error);
      if (!silent) {
        status.value = "error";
        setTimeout(() => {
          if (status.value === "error") {
            status.value = "idle";
          }
        }, 3000);
      }
      throw error;
    }
  }

  async function checkForUpdateOnStartup(): Promise<UpdateInfo | null> {
    if (hasCheckedOnStartup.value) {
      return updateInfo.value;
    }
    hasCheckedOnStartup.value = true;
    return checkForUpdate(true);
  }

  function setDownloading(progress: number) {
    status.value = "downloading";
    downloadProgress.value = progress;
    hasStartedUpdateFlow.value = true;
  }

  function setDownloaded(filePath: string) {
    status.value = "downloaded";
    downloadedFilePath.value = filePath;
    downloadProgress.value = 100;
    hasStartedUpdateFlow.value = true;
  }

  function setInstalling() {
    status.value = "installing";
    hasStartedUpdateFlow.value = true;
  }

  function setDownloadError(error: string) {
    errorMessage.value = error;
    if (updateInfo.value?.has_update) {
      status.value = "available";
      return;
    }
    status.value = "error";
  }

  function setInstallError(error: string) {
    errorMessage.value = error;
    if (downloadedFilePath.value) {
      status.value = "downloaded";
      hasStartedUpdateFlow.value = true;
      return;
    }
    if (updateInfo.value?.has_update) {
      status.value = "available";
      return;
    }
    status.value = "error";
  }

  function showUpdateModal() {
    isUpdateModalVisible.value = true;
  }

  function hideUpdateModal() {
    isUpdateModalVisible.value = false;
  }

  return {
    status,
    updateInfo,
    downloadProgress,
    downloadedFilePath,
    errorMessage,
    hasCheckedOnStartup,
    isUpdateModalVisible,
    hasStartedUpdateFlow,
    isUpdateAvailable,
    isDownloading,
    isInstalling,
    isReadyToInstall,
    // ===== 新增导出 =====
    isAurUpdate,
    reset,
    checkForUpdate,
    checkForUpdateOnStartup,
    setDownloading,
    setDownloaded,
    setInstalling,
    setDownloadError,
    setInstallError,
    showUpdateModal,
    hideUpdateModal,
  };
});
