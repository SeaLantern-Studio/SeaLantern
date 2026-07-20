<script setup lang="ts">
import { i18n } from "@language";

const emit = defineEmits<{
  (e: "export"): void;
  (e: "import"): void;
  (e: "reset"): void;
}>();
</script>

<template>
  <div class="settings-actions">
    <div class="actions-left"></div>
    <div class="actions-right">
      <cmz-button variant="ghost" size="sm" @click="emit('export')">{{
        i18n.t("settings.export")
      }}</cmz-button>
      <cmz-button variant="ghost" size="sm" @click="emit('import')">{{
        i18n.t("settings.import")
      }}</cmz-button>
      <cmz-button variant="solid" color="#ef4444" size="sm" @click="emit('reset')">{{
        i18n.t("settings.reset")
      }}</cmz-button>
    </div>
  </div>
</template>

<style scoped>
.settings-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sl-space-md) 0;
  border-top: 1px solid var(--sl-border);
}

.actions-left,
.actions-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}
</style>
