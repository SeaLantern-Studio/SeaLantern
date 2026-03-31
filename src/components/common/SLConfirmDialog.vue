<script setup lang="ts">
import { ref, watch, computed } from "vue";
import SLModal from "@components/common/SLModal.vue";
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
  confirm: [];
  cancel: [];
  close: [];
  "update:visible": [value: boolean];
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

function handleClose(): void {
  emit("close");
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Enter" && !isConfirmDisabled.value) {
    handleConfirm();
  }
}
</script>

<template>
  <SLModal :visible="visible" :title="title" width="420px" @close="handleClose">
    <div
      class="confirm-content"
      :class="{ 'confirm-content--danger': dangerous }"
      @keydown="handleKeydown"
    >
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

    <template #footer>
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
    </template>
  </SLModal>
</template>

<style scoped>
.confirm-content {
  padding: var(--sl-space-lg);
}

.confirm-content--danger {
  border-left: 3px solid var(--sl-error);
  margin-left: calc(-1 * var(--sl-space-lg));
  padding-left: calc(var(--sl-space-lg) - 3px);
}

.confirm-message {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0;
}

.confirm-input-group {
  margin-top: var(--sl-space-md);
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
}
</style>
