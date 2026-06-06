<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

defineProps<{
  selectedCount: number;
}>();

const emit = defineEmits<{
  (e: "select-all"): void;
  (e: "invert-selection"): void;
  (e: "deselect-all"): void;
  (e: "batch-delete"): void;
}>();
</script>

<template>
  <div class="batch-action-bar">
    <div class="batch-action-left">
      <span class="selected-count">{{
        i18n.t("plugins.selected_count", { count: selectedCount })
      }}</span>
    </div>
    <div class="batch-action-right">
      <SLButton variant="secondary" size="sm" @click="emit('select-all')">
        {{ i18n.t("plugins.select_all") }}
      </SLButton>
      <SLButton variant="secondary" size="sm" @click="emit('invert-selection')">
        {{ i18n.t("plugins.invert_selection") }}
      </SLButton>
      <SLButton variant="secondary" size="sm" @click="emit('deselect-all')">
        {{ i18n.t("plugins.deselect_all") }}
      </SLButton>
      <SLButton
        variant="danger"
        size="sm"
        :disabled="selectedCount === 0"
        @click="emit('batch-delete')"
      >
        {{ i18n.t("plugins.batch_delete") }}
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.batch-action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-sm) var(--sl-space-md);
  margin-bottom: var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
}

.batch-action-left,
.batch-action-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.selected-count {
  font-size: 13px;
  color: var(--sl-text-secondary);
}
</style>
