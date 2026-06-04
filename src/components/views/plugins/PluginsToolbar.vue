<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

defineProps<{
  searchQuery: string;
  batchMode: boolean;
  checkingAllUpdates: boolean;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "update:searchQuery", value: string): void;
  (e: "toggle-batch-mode"): void;
  (e: "check-all-updates"): void;
  (e: "refresh"): void;
}>();
</script>

<template>
  <div class="plugins-toolbar">
    <div class="toolbar-left">
      <input
        :value="searchQuery"
        type="text"
        class="plugin-search"
        :placeholder="i18n.t('plugins.search_placeholder')"
        @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="toolbar-right">
      <SLButton
        :variant="batchMode ? 'primary' : 'secondary'"
        size="sm"
        @click="emit('toggle-batch-mode')"
      >
        {{ i18n.t("plugins.batch_mode") }}
      </SLButton>
      <SLButton
        variant="secondary"
        size="sm"
        :loading="checkingAllUpdates"
        @click="emit('check-all-updates')"
      >
        {{ i18n.t("plugins.check_updates") }}
      </SLButton>
      <SLButton variant="secondary" size="sm" :loading="loading" @click="emit('refresh')">
        {{ i18n.t("plugins.refresh") }}
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.plugins-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-md);
  padding: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-md);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}

.plugin-search {
  padding: 6px 12px;
  border-radius: var(--sl-radius-sm);
  border: 1px solid var(--sl-border);
  background: var(--sl-bg-secondary);
  color: var(--sl-text-primary);
  font-size: 13px;
  width: 180px;
  transition: all var(--sl-transition-fast);
}

.plugin-search:focus {
  outline: none;
  border-color: var(--sl-primary);
}
</style>
