<script setup lang="ts">
import { ref } from "vue";
import { ArrowDownToLine, X } from "lucide-vue-next";
import SLModal from "./SLModal.vue";

interface Props {
  visible: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  close: [];
  confirm: [action: "exit" | "tray", remember: boolean];
}>();

const rememberChoice = ref(false);

function handleExit() {
  emit("confirm", "exit", rememberChoice.value);
}

function handleMinimize() {
  emit("confirm", "tray", rememberChoice.value);
}

function handleClose() {
  emit("close");
}
</script>

<template>
  <SLModal :visible="visible" title="关闭选项" width="360px" @close="handleClose">
    <div class="close-dialog-content">
      <div class="close-options">
        <button class="close-option" @click="handleExit">
          <X class="option-icon exit" :size="20" />
          <span class="option-label">关闭软件</span>
        </button>
        <button class="close-option" @click="handleMinimize">
          <ArrowDownToLine class="option-icon tray" :size="20" />
          <span class="option-label">最小化到托盘</span>
        </button>
      </div>
      <label class="remember-choice">
        <input type="checkbox" v-model="rememberChoice" />
        <span class="checkbox-custom"></span>
        <span class="remember-text">记住我的选择</span>
      </label>
    </div>
  </SLModal>
</template>

<style scoped>
.close-dialog-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.close-options {
  display: flex;
  flex-direction: row;
  gap: 12px;
}

.close-option {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  border: none;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.close-option:hover {
  background: var(--sl-border);
}

.close-option:active {
  transform: scale(0.98);
}

.option-icon {
  flex-shrink: 0;
}

.option-icon.exit {
  color: var(--sl-error);
}

.option-icon.tray {
  color: var(--sl-primary);
}

.option-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.remember-choice {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
  padding: 4px 0;
  border-top: 1px solid var(--sl-border-light);
  padding-top: 12px;
}

.remember-choice input[type="checkbox"] {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.checkbox-custom {
  width: 16px;
  height: 16px;
  border: 1.5px solid var(--sl-text-tertiary);
  border-radius: 4px;
  background: transparent;
  transition: all var(--sl-transition-fast);
  position: relative;
  flex-shrink: 0;
}

.checkbox-custom::after {
  content: "";
  position: absolute;
  left: 4px;
  top: 1px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg) scale(0);
  transition: transform var(--sl-transition-fast);
}

.remember-choice input[type="checkbox"]:checked + .checkbox-custom {
  background: var(--sl-primary);
  border-color: var(--sl-primary);
}

.remember-choice input[type="checkbox"]:checked + .checkbox-custom::after {
  transform: rotate(45deg) scale(1);
}

.remember-text {
  font-size: 13px;
  color: var(--sl-text-secondary);
}

.remember-choice:hover .checkbox-custom {
  border-color: var(--sl-primary);
}
</style>
