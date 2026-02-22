<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import { i18n } from "@language";

interface Props {
  visible: boolean;
  title?: string;
  message?: string;
  confirmText?: string;
  cancelText?: string;
  confirmVariant?: "primary" | "danger" | "secondary";
  requireInput?: boolean;
  inputPlaceholder?: string;
  expectedInput?: string;
  loading?: boolean;
  dangerous?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  title: () => i18n.t("common.confirm_action"),
  message: "",
  confirmText: () => i18n.t("common.confirm"),
  cancelText: () => i18n.t("common.cancel"),
  confirmVariant: "primary",
  requireInput: false,
  inputPlaceholder: "",
  expectedInput: "",
  loading: false,
  dangerous: false,
});

const emit = defineEmits<{
  (e: "confirm"): void;
  (e: "cancel"): void;
  (e: "close"): void;
  (e: "update:visible", value: boolean): void;
}>();

const inputValue = ref("");
const inputError = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

const isConfirmDisabled = computed(() => {
  if (props.loading) return true;
  if (props.requireInput) {
    return inputValue.value.trim() !== props.expectedInput;
  }
  return false;
});

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      inputValue.value = "";
      inputError.value = "";
      setTimeout(() => {
        inputRef.value?.focus();
      }, 100);
    }
  },
);

function handleConfirm(): void {
  if (isConfirmDisabled.value) return;
  emit("confirm");
}

function handleCancel(): void {
  emit("cancel");
  emit("close");
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    handleCancel();
  } else if (event.key === "Enter" && !isConfirmDisabled.value) {
    handleConfirm();
  }
}

function handleOverlayClick(event: MouseEvent): void {
  if (event.target === event.currentTarget) {
    handleCancel();
  }
}

onMounted(() => {
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm-fade">
      <div v-if="visible" class="confirm-overlay" @click="handleOverlayClick">
        <div class="confirm-dialog" :class="{ 'confirm-dialog--danger': dangerous }" @click.stop>
          <div class="confirm-header">
            <h3 class="confirm-title">{{ title }}</h3>
          </div>

          <div class="confirm-body">
            <p v-if="message" class="confirm-message" v-html="message"></p>

            <div v-if="requireInput" class="confirm-input-group">
              <SLInput
                ref="inputRef"
                v-model="inputValue"
                :placeholder="inputPlaceholder"
                @keyup.enter="handleConfirm"
                @keyup.escape="handleCancel"
              />
              <p v-if="inputError" class="confirm-error">{{ inputError }}</p>
            </div>
          </div>

          <div class="confirm-footer">
            <SLButton variant="secondary" :disabled="loading" @click="handleCancel">
              {{ cancelText }}
            </SLButton>
            <SLButton
              :variant="confirmVariant"
              :loading="loading"
              :disabled="isConfirmDisabled"
              @click="handleConfirm"
            >
              {{ confirmText }}
            </SLButton>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--sl-space-md);
}

.confirm-dialog {
  background: var(--sl-surface);
  border-radius: var(--sl-radius-lg);
  box-shadow: var(--sl-shadow-lg);
  width: 100%;
  max-width: 420px;
  overflow: hidden;
}

.confirm-dialog--danger {
  border: 1px solid var(--sl-error);
}

.confirm-header {
  padding: var(--sl-space-md) var(--sl-space-lg);
  border-bottom: 1px solid var(--sl-border-light);
}

.confirm-title {
  font-size: 1.0625rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
}

.confirm-body {
  padding: var(--sl-space-lg);
}

.confirm-message {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0 0 var(--sl-space-md) 0;
}

.confirm-message:last-child {
  margin-bottom: 0;
}

.confirm-input-group {
  margin-top: var(--sl-space-sm);
}

.confirm-error {
  margin-top: var(--sl-space-xs);
  font-size: 0.75rem;
  color: var(--sl-error);
}

.confirm-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-md) var(--sl-space-lg);
  border-top: 1px solid var(--sl-border-light);
  background: var(--sl-bg-secondary);
}

.confirm-fade-enter-active,
.confirm-fade-leave-active {
  transition: opacity 0.2s ease;
}

.confirm-fade-enter-active .confirm-dialog,
.confirm-fade-leave-active .confirm-dialog {
  transition: transform 0.2s ease;
}

.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}

.confirm-fade-enter-from .confirm-dialog {
  transform: scale(0.95) translateY(-10px);
}

.confirm-fade-leave-to .confirm-dialog {
  transform: scale(0.95) translateY(10px);
}
</style>
