<script setup lang="ts">
import { computed } from "vue";
import { FolderOpen, AlertTriangle, CheckCircle, XCircle, Loader2 } from "lucide-vue-next";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import SLCard from "@components/common/SLCard.vue";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import {
  changePathModalVisible,
  changePathLoading,
  changePathValidationResult,
  selectedNewPath,
  changingPathServerId,
  closeChangePathModal,
  selectNewPath,
  confirmChangePath,
} from "@utils/serverUtils";

const store = useServerStore();

const currentServer = computed(() => {
  if (!changingPathServerId.value) return null;
  return store.getServerById(changingPathServerId.value);
});

const canConfirm = computed(() => {
  return (
    selectedNewPath.value && changePathValidationResult.value?.valid && !changePathLoading.value
  );
});

const validationStatus = computed(() => {
  if (!changePathValidationResult.value) return null;
  return changePathValidationResult.value.valid ? "success" : "error";
});
</script>

<template>
  <SLModal
    :visible="changePathModalVisible"
    :title="i18n.t('home.change_path_title')"
    @close="closeChangePathModal"
    width="560px"
  >
    <div class="change-path-content">
      <!-- 警告提示 -->
      <SLCard variant="outline" class="warning-card">
        <div class="warning-content">
          <AlertTriangle :size="20" class="warning-icon" />
          <span class="warning-text">{{ i18n.t("home.change_path_warning") }}</span>
        </div>
      </SLCard>

      <!-- 当前路径 -->
      <div class="path-section">
        <label class="path-label">{{ i18n.t("home.change_path_current") }}</label>
        <div class="path-display current">
          <code class="path-code">{{ currentServer?.path || "-" }}</code>
        </div>
      </div>

      <!-- 新路径选择 -->
      <div class="path-section">
        <label class="path-label">{{ i18n.t("home.change_path_new") }}</label>
        <div class="path-input-group">
          <div class="path-display" :class="{ selected: selectedNewPath }">
            <code class="path-code">{{
              selectedNewPath || i18n.t("home.change_path_select_folder")
            }}</code>
          </div>
          <SLButton
            variant="secondary"
            size="sm"
            @click="selectNewPath"
            :loading="changePathLoading"
          >
            <FolderOpen :size="16" />
            {{ i18n.t("add_existing.browse") }}
          </SLButton>
        </div>
      </div>

      <!-- 验证结果 -->
      <div v-if="changePathValidationResult" class="validation-result" :class="validationStatus">
        <div class="validation-header">
          <CheckCircle
            v-if="changePathValidationResult.valid"
            :size="18"
            class="validation-icon success"
          />
          <XCircle v-else :size="18" class="validation-icon error" />
          <span class="validation-title">
            {{
              changePathValidationResult.valid
                ? i18n.t("common.confirm")
                : i18n.t("home.change_path_invalid")
            }}
          </span>
        </div>
        <p class="validation-message">{{ changePathValidationResult.message }}</p>

        <!-- 检测到的启动文件信息 -->
        <div
          v-if="changePathValidationResult.valid && changePathValidationResult.jarPath"
          class="detected-info"
        >
          <div class="detected-item">
            <span class="detected-label">启动文件:</span>
            <code class="detected-value">{{ changePathValidationResult.jarPath }}</code>
          </div>
          <div class="detected-item">
            <span class="detected-label">启动方式:</span>
            <span class="detected-value">{{
              changePathValidationResult.startupMode?.toUpperCase() || "JAR"
            }}</span>
          </div>
        </div>
      </div>

      <!-- 验证中状态 -->
      <div v-else-if="changePathLoading" class="validation-loading">
        <Loader2 :size="20" class="loading-spinner" />
        <span>{{ i18n.t("home.change_path_validating") }}</span>
      </div>
    </div>

    <!-- 底部按钮 -->
    <template #footer>
      <div class="modal-actions">
        <SLButton variant="ghost" @click="closeChangePathModal" :disabled="changePathLoading">
          {{ i18n.t("home.change_path_cancel") }}
        </SLButton>
        <SLButton
          variant="primary"
          @click="confirmChangePath"
          :loading="changePathLoading"
          :disabled="!canConfirm"
        >
          {{ i18n.t("home.change_path_confirm") }}
        </SLButton>
      </div>
    </template>
  </SLModal>
</template>

<style scoped>
.change-path-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.warning-card {
  background: rgba(245, 158, 11, 0.08);
  border-color: rgba(245, 158, 11, 0.3);
}

.warning-content {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
}

.warning-icon {
  color: var(--sl-warning);
  flex-shrink: 0;
  margin-top: 2px;
}

.warning-text {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.5;
}

.path-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.path-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
}

.path-display {
  display: flex;
  align-items: center;
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: var(--sl-bg-tertiary);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  min-height: 40px;
}

.path-display.current {
  background: var(--sl-bg-secondary);
}

.path-display.selected {
  background: var(--sl-primary-bg);
  border-color: var(--sl-primary-light);
}

.path-code {
  font-family: var(--sl-font-mono);
  font-size: 0.8125rem;
  color: var(--sl-text-primary);
  word-break: break-all;
  line-height: 1.4;
}

.path-input-group {
  display: flex;
  gap: var(--sl-space-sm);
}

.path-input-group .path-display {
  flex: 1;
}

.validation-result {
  padding: var(--sl-space-md);
  border-radius: var(--sl-radius-md);
  border: 1px solid;
}

.validation-result.success {
  background: rgba(34, 197, 94, 0.08);
  border-color: rgba(34, 197, 94, 0.3);
}

.validation-result.error {
  background: rgba(239, 68, 68, 0.08);
  border-color: rgba(239, 68, 68, 0.3);
}

.validation-header {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  margin-bottom: var(--sl-space-xs);
}

.validation-icon.success {
  color: var(--sl-success);
}

.validation-icon.error {
  color: var(--sl-danger);
}

.validation-title {
  font-size: 0.875rem;
  font-weight: 600;
}

.validation-result.success .validation-title {
  color: var(--sl-success);
}

.validation-result.error .validation-title {
  color: var(--sl-danger);
}

.validation-message {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  margin: 0;
  line-height: 1.5;
}

.detected-info {
  margin-top: var(--sl-space-sm);
  padding-top: var(--sl-space-sm);
  border-top: 1px dashed var(--sl-border-light);
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}

.detected-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  font-size: 0.75rem;
}

.detected-label {
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
}

.detected-value {
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
  background: var(--sl-bg-secondary);
  padding: 2px 6px;
  border-radius: var(--sl-radius-sm);
}

.validation-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-md);
  color: var(--sl-text-secondary);
  font-size: 0.875rem;
}

.loading-spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
}
</style>
