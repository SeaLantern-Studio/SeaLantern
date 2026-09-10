<script setup lang="ts">
/**
 * UI 组件测试面板
 * 覆盖 cmzya-modern-ui 与项目内自研组件
 */
import { ref } from "vue";
import { i18n } from "@language";
import { useToast } from "cmzya-modern-ui";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";

const toast = useToast();

// Toast 测试
const toastMessage = ref("测试消息");
const toastDuration = ref(3000);
function showToast(type: "success" | "error" | "info" | "warning") {
  const msg = toastMessage.value || "测试消息";
  toast[type]?.(msg);
}

// Confirm 弹窗测试
const showConfirm = ref(false);
const confirmRequireInput = ref(false);
const confirmDangerous = ref(false);
const confirmResult = ref("");
function openConfirm() {
  confirmResult.value = "";
  showConfirm.value = true;
}
function onConfirm() {
  confirmResult.value = "用户点击了确认";
  showConfirm.value = false;
}
function onCancel() {
  confirmResult.value = "用户点击了取消";
  showConfirm.value = false;
}

// Switch 测试
const switchValue = ref(false);

// Input 测试
const inputValue = ref("");
const inputMultiline = ref("");

// Button 测试
const btnLoading = ref(false);
let btnTimer: ReturnType<typeof setTimeout> | null = null;
function triggerBtnLoading() {
  btnLoading.value = true;
  if (btnTimer) clearTimeout(btnTimer);
  btnTimer = setTimeout(() => {
    btnLoading.value = false;
  }, 1500);
}

// Modal 测试
const showModal = ref(false);
</script>

<template>
  <div class="dev-panel">
    <!-- Toast -->
    <cmz-card title="Toast 通知" padding="md" data-dev-section="toast">
      <div class="test-row">
        <span class="test-label">消息内容</span>
        <cmz-input v-model="toastMessage" />
      </div>
      <div class="test-row">
        <span class="test-label">持续时长(ms)</span>
        <cmz-input v-model.number="toastDuration" type="number" />
      </div>
      <div class="test-actions">
        <cmz-button size="sm" color="#22c55e" @click="showToast('success')">success</cmz-button>
        <cmz-button size="sm" color="#ef4444" @click="showToast('error')">error</cmz-button>
        <cmz-button size="sm" color="#3b82f6" @click="showToast('info')">info</cmz-button>
        <cmz-button size="sm" color="#fbbf24" @click="showToast('warning')">warning</cmz-button>
      </div>
    </cmz-card>

    <!-- Confirm 弹窗 -->
    <cmz-card title="SLConfirmDialog 确认弹窗" padding="md" data-dev-section="confirm">
      <div class="test-row">
        <label class="test-checkbox">
          <cmz-switch v-model="confirmRequireInput" />
          <span>需要输入校验</span>
        </label>
        <label class="test-checkbox">
          <cmz-switch v-model="confirmDangerous" />
          <span>危险操作样式</span>
        </label>
      </div>
      <div class="test-actions">
        <cmz-button size="sm" @click="openConfirm">打开确认弹窗</cmz-button>
      </div>
      <div v-if="confirmResult" class="test-row">
        <span class="test-label">结果</span>
        <code class="test-result">{{ confirmResult }}</code>
      </div>
    </cmz-card>

    <!-- Switch -->
    <cmz-card title="Switch 开关" padding="md" data-dev-section="switch">
      <div class="test-row">
        <span class="test-label">当前值</span>
        <code class="test-result">{{ switchValue }}</code>
        <cmz-switch v-model="switchValue" />
      </div>
    </cmz-card>

    <!-- Input -->
    <cmz-card title="Input 输入框" padding="md" data-dev-section="input">
      <div class="test-row">
        <span class="test-label">单行</span>
        <cmz-input v-model="inputValue" placeholder="请输入" />
      </div>
      <div class="test-row">
        <span class="test-label">当前值</span>
        <code class="test-result">{{ inputValue }}</code>
      </div>
      <div class="test-row">
        <span class="test-label">多行</span>
        <cmz-input v-model="inputMultiline" multiline :rows="3" placeholder="多行输入" />
      </div>
    </cmz-card>

    <!-- Button -->
    <cmz-card title="Button 按钮" padding="md" data-dev-section="button">
      <div class="test-actions">
        <cmz-button variant="primary" size="sm">primary</cmz-button>
        <cmz-button variant="solid" size="sm">solid</cmz-button>
        <cmz-button variant="outline" size="sm">outline</cmz-button>
        <cmz-button variant="ghost" size="sm">ghost</cmz-button>
        <cmz-button size="sm" color="#ef4444">自定义色</cmz-button>
        <cmz-button size="sm" :loading="btnLoading" @click="triggerBtnLoading">loading</cmz-button>
        <cmz-button size="sm" disabled>disabled</cmz-button>
      </div>
    </cmz-card>

    <!-- Modal -->
    <cmz-card title="Modal 弹层" padding="md" data-dev-section="modal">
      <div class="test-actions">
        <cmz-button size="sm" @click="showModal = true">打开 Modal</cmz-button>
      </div>
    </cmz-card>

    <!-- 确认弹窗实例 -->
    <SLConfirmDialog
      :visible="showConfirm"
      :title="i18n.t('common.confirm_action')"
      message="这是一个测试确认弹窗"
      :confirm-text="i18n.t('common.confirm')"
      :cancel-text="i18n.t('common.cancel')"
      :require-input="confirmRequireInput"
      :expected-input="confirmRequireInput ? '确认' : ''"
      :input-placeholder="confirmRequireInput ? '请输入「确认」' : ''"
      :dangerous="confirmDangerous"
      @confirm="onConfirm"
      @cancel="onCancel"
      @close="showConfirm = false"
    />

    <!-- Modal 实例 -->
    <cmz-modal :visible="showModal" title="测试 Modal" width="480px" @close="showModal = false">
      <div style="padding: 16px">
        <p>这是一个测试 Modal 弹层。</p>
        <p>用于验证 cmz-modal 的显隐、尺寸与关闭行为。</p>
      </div>
      <template #footer>
        <cmz-button size="sm" @click="showModal = false">关闭</cmz-button>
      </template>
    </cmz-modal>
  </div>
</template>

<style scoped>
.dev-panel {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
.test-row {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-sm);
}
.test-label {
  min-width: 120px;
  color: var(--sl-text-secondary);
  font-size: 13px;
}
.test-result {
  color: var(--sl-primary);
  word-break: break-all;
}
.test-actions {
  display: flex;
  gap: var(--sl-space-sm);
  flex-wrap: wrap;
  margin-bottom: var(--sl-space-sm);
}
.test-checkbox {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  font-size: 13px;
  color: var(--sl-text-secondary);
}
</style>
