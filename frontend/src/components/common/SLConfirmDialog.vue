<script setup lang="ts">
import { ref, watch, computed } from "vue";
import DOMPurify from "dompurify";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import SLInput from "@components/common/SLInput.vue";
import type { ConfirmDialogOption } from "@components/common/confirmDialogTypes";
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
  options?: ConfirmDialogOption[];
  selectedOption?: string;
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
  options: () => [],
  selectedOption: "",
});

const emit = defineEmits<{
  confirm: [];
  cancel: [];
  close: [];
  "update:visible": [value: boolean];
  "update:selectedOption": [value: string];
}>();

const inputValue = ref("");
const inputError = ref("");
const inputRef = ref<HTMLInputElement | null>(null);

const hasOptions = computed(() => props.options.length > 0);

const isConfirmDisabled = computed(() => {
  if (props.loading) return true;
  if (hasOptions.value && !props.selectedOption) {
    return true;
  }
  if (props.requireInput) {
    return inputValue.value.trim() !== props.expectedInput;
  }
  return false;
});

const safeMessage = computed(() => {
  return DOMPurify.sanitize(props.message, {
    FORBID_TAGS: ["script", "iframe", "style", "link"],
    FORBID_ATTR: ["style"],
  });
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

function handleOptionChange(value: string): void {
  emit("update:selectedOption", value);
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
    return;
  }

  if (event.key === "Escape") {
    handleCancel();
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
      <p v-if="message" class="confirm-message" v-html="safeMessage"></p>

      <div v-if="hasOptions" class="confirm-options" role="radiogroup">
        <label
          v-for="option in options"
          :key="option.value"
          class="confirm-option"
          :class="{ 'confirm-option--selected': selectedOption === option.value }"
        >
          <input
            class="confirm-option__radio"
            type="radio"
            :name="`${title}-option`"
            :value="option.value"
            :checked="selectedOption === option.value"
            @change="handleOptionChange(option.value)"
          />
          <span class="confirm-option__body">
            <span class="confirm-option__label">{{ option.label }}</span>
            <span v-if="option.description" class="confirm-option__description">
              {{ option.description }}
            </span>
          </span>
        </label>
      </div>

      <div v-if="requireInput" class="confirm-input-group">
        <SLInput ref="inputRef" v-model="inputValue" :placeholder="inputPlaceholder" />
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

.confirm-options {
  display: grid;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-md);
}

.confirm-option {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
  padding: 12px;
  border: 1px solid var(--sl-border-color);
  border-radius: 12px;
  cursor: pointer;
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease;
}

.confirm-option--selected {
  border-color: var(--sl-primary);
  background: color-mix(in srgb, var(--sl-primary) 8%, transparent);
}

.confirm-option__radio {
  margin-top: 2px;
}

.confirm-option__body {
  display: grid;
  gap: 4px;
}

.confirm-option__label {
  color: var(--sl-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.confirm-option__description {
  color: var(--sl-text-secondary);
  font-size: 0.8125rem;
  line-height: 1.5;
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
