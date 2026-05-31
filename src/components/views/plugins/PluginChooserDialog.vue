<script setup lang="ts">
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";
import { File, Folder, X } from "lucide-vue-next";

defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "pick-file"): void;
  (e: "pick-folder"): void;
}>();
</script>

<template>
  <DialogRoot :open="open" @update:open="emit('update:open', $event)">
    <DialogPortal>
      <DialogOverlay class="plugin-chooser-overlay" />
      <DialogContent class="plugin-chooser-content">
        <div class="plugin-chooser-header">
          <DialogTitle class="plugin-chooser-title">{{
            i18n.t("plugins.choose_title")
          }}</DialogTitle>
          <button
            class="plugin-chooser-close"
            @click="emit('update:open', false)"
            :aria-label="i18n.t('common.close_modal')"
          >
            <X :size="18" />
          </button>
        </div>
        <DialogDescription class="plugin-chooser-description">
          {{ i18n.t("plugins.choose_description") }}
        </DialogDescription>
        <div class="plugin-chooser-actions">
          <SLButton
            variant="primary"
            size="lg"
            class="plugin-chooser-option"
            @click="emit('pick-file')"
          >
            <File :size="22" />
            <span>{{ i18n.t("plugins.select_file") }}</span>
          </SLButton>
          <SLButton
            variant="secondary"
            size="lg"
            class="plugin-chooser-option"
            @click="emit('pick-folder')"
          >
            <Folder :size="22" />
            <span>{{ i18n.t("plugins.select_folder") }}</span>
          </SLButton>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.plugin-chooser-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.45);
  z-index: 3000;
}

.plugin-chooser-content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: min(420px, calc(100vw - 32px));
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-lg);
  box-shadow: var(--sl-shadow-lg);
  padding: var(--sl-space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
  z-index: 3001;
}

.plugin-chooser-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sl-space-xs);
}

.plugin-chooser-title {
  margin: 0;
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
}

.plugin-chooser-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--sl-radius-md);
  border: none;
  background: transparent;
  color: var(--sl-text-tertiary);
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition:
    background-color 0.2s ease,
    color 0.2s ease,
    transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.plugin-chooser-close::before {
  content: "";
  position: absolute;
  inset: 0;
  background: var(--sl-error);
  border-radius: inherit;
  opacity: 0;
  transform: scale(0.5);
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.plugin-chooser-close:hover {
  color: var(--sl-error);
  transform: rotate(90deg);
}

.plugin-chooser-close:hover::before {
  opacity: 0.1;
  transform: scale(1);
}

.plugin-chooser-close:active {
  transform: rotate(90deg) scale(0.9);
}

.plugin-chooser-description {
  margin: 0;
  font-size: var(--sl-font-size-base);
  color: var(--sl-text-secondary);
  line-height: 1.5;
}

.plugin-chooser-actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--sl-space-sm);
  margin: var(--sl-space-sm) 0;
}

.plugin-chooser-option {
  width: 100%;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-md) var(--sl-space-lg);
}
</style>
