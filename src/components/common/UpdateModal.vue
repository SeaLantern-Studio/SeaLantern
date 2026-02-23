<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import { useUpdateStore } from "@stores/updateStore";
import { i18n } from "@language";
import { downloadUpdate, installUpdate, onDownloadProgress } from "@api/update";
import { serverApi } from "@api/server";
import type { UnlistenFn } from "@tauri-apps/api/event";

const updateStore = useUpdateStore();

const showInstallRiskConfirm = ref(false);
const runningServerNames = ref<string[]>([]);
let unlistenProgress: UnlistenFn | null = null;
const isInstallLaunching = computed(() => updateStore.status === "installing");

const buttonState = computed(() => {
  if (updateStore.status === "downloading") {
    return {
      text: `${i18n.t("about.update_downloading")} ${progressPercent.value}%`,
      variant: "secondary" as const,
      disabled: true,
    };
  }
  if (updateStore.status === "installing") {
    return {
      text: i18n.t("about.update_installing"),
      variant: "secondary" as const,
      disabled: true,
    };
  }
  if (updateStore.status === "downloaded") {
    return {
      text: i18n.t("about.update_restart_install"),
      variant: "success" as const,
      disabled: false,
    };
  }
  return {
    text: i18n.t("about.update_now"),
    variant: "primary" as const,
    disabled: false,
  };
});

const progressPercent = computed(() => {
  return Math.round(updateStore.downloadProgress);
});

onMounted(async () => {
  unlistenProgress = await onDownloadProgress((progress) => {
    updateStore.setDownloading(progress.percent);
  });
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
});

async function handleUpdateClick() {
  if (isInstallLaunching.value) {
    return;
  }

  if (updateStore.status === "downloading" || updateStore.status === "installing") {
    return;
  }

  if (updateStore.status === "downloaded" && updateStore.downloadedFilePath) {
    await handleInstallClick();
    return;
  }

  if (!updateStore.updateInfo?.download_url) {
    return;
  }

  try {
    updateStore.setDownloading(0);
    const filePath = await downloadUpdate(
      updateStore.updateInfo.download_url,
      updateStore.updateInfo.sha256,
    );
    updateStore.setDownloaded(filePath);
  } catch (error) {
    updateStore.setDownloadError(String(error));
  }
}

async function getRunningServerNames(): Promise<string[]> {
  const servers = await serverApi.getList();
  if (servers.length === 0) {
    return [];
  }

  const snapshots = await Promise.all(
    servers.map(async (server) => {
      try {
        const statusInfo = await serverApi.getStatus(server.id);
        return { name: server.name, status: statusInfo.status };
      } catch {
        return null;
      }
    }),
  );

  return snapshots
    .filter(
      (
        item,
      ): item is {
        name: string;
        status: "Stopped" | "Starting" | "Running" | "Stopping" | "Error";
      } => item !== null,
    )
    .filter((item) => item.status === "Running")
    .map((item) => item.name);
}

async function handleInstallClick() {
  if (!updateStore.downloadedFilePath || !updateStore.updateInfo) {
    return;
  }

  try {
    runningServerNames.value = await getRunningServerNames();
  } catch (error) {
    console.error("Failed to check running servers:", error);
    runningServerNames.value = [];
  }

  if (runningServerNames.value.length > 0) {
    showInstallRiskConfirm.value = true;
    return;
  }

  await performInstall();
}

async function performInstall() {
  if (!updateStore.downloadedFilePath || !updateStore.updateInfo) {
    return;
  }

  if (isInstallLaunching.value) {
    return;
  }

  updateStore.setInstalling();

  try {
    await installUpdate(updateStore.downloadedFilePath, updateStore.updateInfo.latest_version);
    window.close();
  } catch (error) {
    console.error("Install failed:", error);
    updateStore.setInstallError(String(error));
  }
}

async function handleForceAutoUpdate() {
  if (isInstallLaunching.value) {
    return;
  }

  showInstallRiskConfirm.value = false;
  try {
    await serverApi.forceStopAll();
  } catch (error) {
    updateStore.setInstallError(String(error));
    return;
  }
  await performInstall();
}

function closeInstallRiskConfirm() {
  showInstallRiskConfirm.value = false;
}
</script>

<template>
  <SLModal
    :visible="true"
    :title="
      updateStore.updateInfo
        ? `${i18n.t('about.update_title')} v${updateStore.updateInfo.latest_version}`
        : i18n.t('about.update_title')
    "
    @close="$emit('close')"
  >
    <div class="update-modal-content">
      <div v-if="updateStore.updateInfo" class="update-info">
        <div class="version-info">
          <span class="label">{{ i18n.t("about.update_current") }}:</span>
          <span class="value">v{{ updateStore.updateInfo.current_version }}</span>
        </div>
        <div class="version-info">
          <span class="label">{{ i18n.t("about.update_latest_version") }}:</span>
          <span class="value highlight">v{{ updateStore.updateInfo.latest_version }}</span>
        </div>
      </div>

      <div v-if="updateStore.updateInfo?.release_notes" class="release-notes">
        <div class="notes-title">{{ i18n.t("about.update_release_notes") }}:</div>
        <div class="notes-content">{{ updateStore.updateInfo.release_notes }}</div>
      </div>

      <div v-if="updateStore.errorMessage" class="error-message">
        {{ updateStore.errorMessage }}
      </div>

      <div class="update-actions">
        <SLButton
          class="update-action-btn"
          :variant="buttonState.variant"
          size="md"
          @click="handleUpdateClick"
          :disabled="buttonState.disabled"
          style="width: 100%"
        >
          <span class="update-btn-content">
            <span
              v-if="updateStore.status === 'downloading'"
              class="update-btn-progress"
              :style="{ width: `${progressPercent}%` }"
            />
            <span class="update-btn-label">{{ buttonState.text }}</span>
          </span>
        </SLButton>
      </div>
    </div>
  </SLModal>

  <SLModal
    :visible="showInstallRiskConfirm"
    :title="i18n.t('about.update_running_warning_title')"
    width="520px"
    @close="closeInstallRiskConfirm"
  >
    <div class="install-risk-content">
      <div class="install-risk-icon">!</div>
      <div class="install-risk-text">{{ i18n.t("about.update_running_warning_desc") }}</div>
      <div class="install-risk-hint">{{ i18n.t("about.update_running_warning_hint") }}</div>

      <div v-if="runningServerNames.length > 0" class="running-servers-box">
        <div class="running-servers-title">{{ i18n.t("about.update_running_servers") }}:</div>
        <div class="running-servers-value">{{ runningServerNames.join(" / ") }}</div>
      </div>

      <div class="install-risk-actions">
        <SLButton
          variant="secondary"
          size="sm"
          :disabled="isInstallLaunching"
          @click="closeInstallRiskConfirm"
        >
          {{ i18n.t("about.update_cancel_update") }}
        </SLButton>
        <SLButton
          variant="danger"
          size="md"
          :disabled="isInstallLaunching"
          @click="handleForceAutoUpdate"
        >
          {{
            isInstallLaunching
              ? i18n.t("about.update_installing")
              : i18n.t("about.update_force_auto")
          }}
        </SLButton>
      </div>
    </div>
  </SLModal>
</template>

<style scoped>
.update-modal-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.update-info {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  padding-bottom: var(--sl-space-sm);
  border-bottom: 1px solid var(--sl-border-light);
}

.version-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.version-info .label {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}

.version-info .value {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
}

.version-info .value.highlight {
  color: var(--sl-primary);
}

.release-notes {
  max-height: 200px;
  overflow-y: auto;
  padding: var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-md);
}

.notes-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-xs);
}

.notes-content {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  white-space: pre-wrap;
}

.error-message {
  padding: var(--sl-space-sm);
  background: rgba(239, 68, 68, 0.1);
  border-radius: var(--sl-radius-md);
  color: var(--sl-danger);
  font-size: 0.875rem;
}

.update-actions {
  padding-top: var(--sl-space-sm);
}

.update-btn-content {
  position: static;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.update-btn-progress {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  height: 100%;
  background: rgba(255, 255, 255, 0.22);
  transition: width 0.2s ease;
  border-radius: inherit;
  z-index: 0;
}

.update-btn-label {
  position: relative;
  z-index: 1;
  white-space: nowrap;
}

:deep(.update-action-btn) {
  position: relative;
  overflow: hidden;
}

.install-risk-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  align-items: center;
}

.install-risk-icon {
  width: 92px;
  height: 92px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 64px;
  font-weight: 800;
  line-height: 1;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.4);
}

.install-risk-text {
  font-size: 1rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  text-align: center;
}

.install-risk-hint {
  font-size: 0.875rem;
  line-height: 1.7;
  color: var(--sl-text-secondary);
  text-align: center;
}

.running-servers-box {
  width: 100%;
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.running-servers-title {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  margin-bottom: 4px;
}

.running-servers-value {
  font-size: 0.875rem;
  color: var(--sl-danger);
  font-weight: 600;
  word-break: break-all;
}

.install-risk-actions {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}

.install-risk-actions :deep(.sl-button) {
  width: 100%;
}
</style>
