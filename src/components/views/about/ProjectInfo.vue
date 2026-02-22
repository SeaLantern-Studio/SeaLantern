<script setup lang="ts">
import { ref } from "vue";
import { RefreshCw, Check, XCircle, Copy, Terminal } from "lucide-vue-next";
import {
  DialogRoot,
  DialogTrigger,
  DialogPortal,
  DialogOverlay,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogClose,
} from "reka-ui";
import SLCard from "@components/common/SLCard.vue";
import SLButton from "@components/common/SLButton.vue";
import { checkUpdate, type UpdateInfo } from "@api/update";
import { openUrl } from "@tauri-apps/plugin-opener";
import { BUILD_YEAR } from "@utils/version";
import { i18n } from "@language";

const props = defineProps<{
  version: string;
}>();

const buildDate = BUILD_YEAR;

const isCheckingUpdate = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const updateError = ref<string | null>(null);

const aurDialogOpen = ref(false);
const aurInfo = ref<{
  currentVersion: string;
  latestVersion: string;
  helper: string;
  command: string;
} | null>(null);

const isAurUpdate = ref(false);

async function handleCheckUpdate() {
  isCheckingUpdate.value = true;
  updateError.value = null;
  updateInfo.value = null;

  try {
    const info = await checkUpdate();

    if (info) {
      updateInfo.value = info;
      isAurUpdate.value = info.source === "arch-aur";

      if (isAurUpdate.value) {
        const helper =
          info.release_notes?.match(/yay|paru|pamac|trizen|pacaur/)?.[0] || "yay";
        aurInfo.value = {
          currentVersion: info.current_version,
          latestVersion: info.latest_version,
          helper: helper,
          command: `${helper} -Rns sealantern && ${helper} -S sealantern`,
        };
      }
    } else {
      updateInfo.value = {
        has_update: false,
        latest_version: props.version,
        current_version: props.version,
      };
      isAurUpdate.value = false;
    }
  } catch (error) {
    console.error("[ProjectInfo] 检查更新失败:", error);
    updateError.value = error as string;
  } finally {
    isCheckingUpdate.value = false;
  }
}

async function handleManualDownload() {
  if (updateInfo.value?.download_url) {
    try {
      await openUrl(updateInfo.value.download_url);
    } catch (error) {
      console.error("[ProjectInfo] 打开链接失败:", error);
      alert(`打开链接失败: ${error}`);
    }
  }
}

async function copyCommand() {
  if (aurInfo.value?.command) {
    try {
      await navigator.clipboard.writeText(aurInfo.value.command);
    } catch (e) {
      console.error("[ProjectInfo] 复制命令失败:", e);
    }
  }
}
</script>

<template>
  <SLCard :title="i18n.t('about.project_info')">
    <div class="info-list">
      <div class="info-item">
        <span class="info-label">{{ i18n.t("about.version") }}</span>
        <span class="info-value">{{ props.version }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">{{ i18n.t("about.build_year") }}</span>
        <span class="info-value">{{ buildDate }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">{{ i18n.t("about.frontend") }}</span>
        <span class="info-value">Vue 3 + TypeScript + Vite</span>
      </div>
      <div class="info-item">
        <span class="info-label">{{ i18n.t("about.backend") }}</span>
        <span class="info-value">Rust + Tauri 2</span>
      </div>
      <div class="info-item">
        <span class="info-label">{{ i18n.t("about.license") }}</span>
        <span class="info-value">GNU GPLv3</span>
      </div>
    </div>

    <div class="update-section">
      <SLButton
        variant="secondary"
        size="sm"
        @click="handleCheckUpdate"
        :disabled="isCheckingUpdate"
        style="width: 100%"
      >
        {{ isCheckingUpdate ? i18n.t("about.update_checking") : i18n.t("about.check_update") }}
      </SLButton>

      <div v-if="updateInfo" class="update-info">
        <div v-if="updateInfo.has_update" class="update-available">
          <div class="update-message">
            <div class="update-icon">
              <RefreshCw :size="16" :stroke-width="2" />
            </div>
            <div>
              <div class="update-title">
                {{ i18n.t("about.update_available") }} v{{ updateInfo.latest_version }}
              </div>
              <div class="update-desc">
                {{ i18n.t("about.update_current") }}: v{{ updateInfo.current_version }}
              </div>
            </div>
          </div>
          <div v-if="updateInfo.release_notes" class="release-notes">
            <div class="notes-title">{{ i18n.t("about.update_release_notes") }}:</div>
            <div class="notes-content">{{ updateInfo.release_notes }}</div>
          </div>
          <div class="update-buttons">
            <DialogRoot v-model:open="aurDialogOpen">
              <DialogTrigger as-child>
                <SLButton
                  v-if="isAurUpdate"
                  variant="primary"
                  size="sm"
                  style="width: 100%"
                >
                  {{ i18n.t("about.go_download") }} (AUR)
                </SLButton>
                <SLButton
                  v-else
                  variant="primary"
                  size="sm"
                  @click="handleManualDownload"
                  style="width: 100%"
                >
                  {{ i18n.t("about.go_download") }}
                </SLButton>
              </DialogTrigger>
              <DialogPortal>
                <DialogOverlay class="dialog-overlay" />
                <DialogContent class="dialog-content">
                  <DialogTitle class="dialog-title">
                    {{ i18n.t("about.aur_window_title_update") }}
                  </DialogTitle>
                  <DialogDescription class="dialog-description">
                    <p class="aur-desc">{{ i18n.t("about.aur_window_new_version") }}</p>
                    <p class="aur-desc">{{ i18n.t("about.aur_window_aur_version") }}</p>
                    <div class="aur-info">
                      <div class="aur-row">
                        <span class="aur-label">{{ i18n.t("about.aur_window_current_version") }}</span>
                        <span class="aur-value">{{ aurInfo?.currentVersion }}</span>
                      </div>
                      <div class="aur-row">
                        <span class="aur-label">{{ i18n.t("about.aur_window_latest_version") }}</span>
                        <span class="aur-value">{{ aurInfo?.latestVersion }}</span>
                      </div>
                      <div class="aur-row">
                        <span class="aur-label">Helper:</span>
                        <span class="aur-value">{{ aurInfo?.helper }}</span>
                      </div>
                    </div>
                    <div class="command-box">
                      <div class="command-header">
                        <Terminal :size="14" />
                        <span>{{ i18n.t("about.aur_window_single_update") }}</span>
                      </div>
                      <code class="command-text">{{ aurInfo?.command }}</code>
                    </div>
                    <p class="aur-tip">{{ i18n.t("about.aur_window_copy_tip") }}</p>
                  </DialogDescription>
                  <div class="dialog-actions">
                    <SLButton variant="secondary" size="sm" @click="copyCommand">
                      <Copy :size="14" style="margin-right: 6px" />
                      {{ i18n.t("about.aur_window_copy_success") }}
                    </SLButton>
                    <DialogClose as-child>
                      <SLButton variant="ghost" size="sm">
                        {{ i18n.t("about.aur_window_button") }}
                      </SLButton>
                    </DialogClose>
                  </div>
                </DialogContent>
              </DialogPortal>
            </DialogRoot>
          </div>
        </div>
        <div v-else class="update-latest">
          <div class="update-icon">
            <Check :size="16" :stroke-width="2.5" />
          </div>
          <span>{{ i18n.t("about.update_latest") }}</span>
        </div>
      </div>

      <div v-if="updateError" class="update-error">
        <div class="error-icon">
          <XCircle :size="14" :stroke-width="2.5" />
        </div>
        <span>{{ updateError }}</span>
      </div>
    </div>
  </SLCard>
</template>

<style scoped>
.info-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--sl-border-light);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}

.info-value {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
}

.update-section {
  margin-top: var(--sl-space-md);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
}

.update-info {
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}

.update-available {
  background: var(--sl-primary-bg);
  border: 1px solid var(--sl-primary-light);
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
}

.update-message {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
}

.update-icon {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-primary);
  color: white;
  border-radius: var(--sl-radius-sm);
}

.update-latest .update-icon {
  background: var(--sl-success);
}

.update-title {
  font-weight: 600;
  color: var(--sl-primary);
  margin-bottom: 2px;
}

.update-desc {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

.release-notes {
  margin-top: var(--sl-space-sm);
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
}

.notes-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--sl-text-secondary);
  margin-bottom: 4px;
}

.notes-content {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  max-height: 120px;
  overflow-y: auto;
  white-space: pre-wrap;
}

.update-buttons {
  display: flex;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.update-latest {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-sm);
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-success);
  font-weight: 500;
}

.update-error {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-sm);
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-danger);
  font-size: 0.8125rem;
}

.error-icon {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-danger);
  color: white;
  border-radius: 50%;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  z-index: 50;
  animation: overlay-show 0.15s ease;
}

.dialog-content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 90vw;
  max-width: 420px;
  max-height: 85vh;
  background: var(--sl-bg-primary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  padding: var(--sl-space-lg);
  box-shadow: var(--sl-shadow-xl);
  z-index: 51;
  animation: content-show 0.2s ease;
  overflow-y: auto;
}

.dialog-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-sm);
}

.dialog-description {
  margin-bottom: var(--sl-space-md);
}

.aur-desc {
  font-size: 0.875rem;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-xs);
}

.aur-tip {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  margin-top: var(--sl-space-sm);
}

.aur-info {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  margin-bottom: var(--sl-space-md);
  padding: var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-md);
}

.aur-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.875rem;
}

.aur-label {
  color: var(--sl-text-tertiary);
}

.aur-value {
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
}

.command-box {
  background: var(--sl-bg-tertiary);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
}

.command-header {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-xs) var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  font-size: 0.75rem;
  color: var(--sl-text-secondary);
}

.command-text {
  display: block;
  padding: var(--sl-space-sm);
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  word-break: break-all;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
}

@keyframes overlay-show {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes content-show {
  from {
    opacity: 0;
    transform: translate(-50%, -48%) scale(0.96);
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }
}
</style>
