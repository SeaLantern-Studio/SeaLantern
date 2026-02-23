<script setup lang="ts">
import { ref } from "vue";
import { RefreshCw, Check, XCircle } from "lucide-vue-next";
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

async function handleCheckUpdate() {
  isCheckingUpdate.value = true;
  updateError.value = null;
  updateInfo.value = null;

  try {
    const info = await checkUpdate();

    if (info) {
      updateInfo.value = info;
    } else {
      updateInfo.value = {
        has_update: false,
        latest_version: props.version,
        current_version: props.version,
      };
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
            <SLButton
              variant="primary"
              size="sm"
              @click="handleManualDownload"
              style="width: 100%"
            >
              {{ i18n.t("about.go_download") }}
            </SLButton>
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
</style>
