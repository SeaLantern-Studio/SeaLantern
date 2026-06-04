<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import { i18n } from "@language";
import { Trash, Trash2 } from "lucide-vue-next";

defineProps<{
  visible: boolean;
  title: string;
  message: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "confirm", deleteData: boolean): void;
}>();
</script>

<template>
  <SLModal :visible="visible" :title="title" @close="emit('close')">
    <div class="batch-delete-dialog">
      <p class="dialog-message">{{ message }}</p>
      <div class="batch-delete-options">
        <SLButton variant="secondary" class="batch-delete-option" @click="emit('confirm', true)">
          <Trash2 class="option-icon delete-with-data" :size="20" />
          <span class="option-label">{{ i18n.t("plugins.delete_with_data") }}</span>
        </SLButton>
        <SLButton variant="secondary" class="batch-delete-option" @click="emit('confirm', false)">
          <Trash class="option-icon delete-without-data" :size="20" />
          <span class="option-label">{{ i18n.t("plugins.delete_without_data") }}</span>
        </SLButton>
      </div>
    </div>
    <template #footer>
      <SLButton variant="secondary" size="sm" @click="emit('close')">{{
        i18n.t("plugins.cancel")
      }}</SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.batch-delete-dialog {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.batch-delete-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.batch-delete-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border: none;
  border-radius: var(--sl-radius-md);
  background: var(--sl-bg-tertiary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.batch-delete-option:hover {
  background: var(--sl-border);
}

.batch-delete-option:active {
  transform: scale(0.98);
}

.batch-delete-option .option-icon {
  flex-shrink: 0;
}

.batch-delete-option .option-icon.delete-with-data {
  color: var(--sl-error);
}

.batch-delete-option .option-icon.delete-without-data {
  color: var(--sl-warning);
}

.batch-delete-option .option-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--sl-text-primary);
}

.dialog-message {
  margin: 0;
  color: var(--sl-text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-line;
}
</style>
