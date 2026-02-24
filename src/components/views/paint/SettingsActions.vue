<script setup lang="ts">
import { i18n } from "@language";
</script>

<template>
  <div class="settings-actions">
    <div class="actions-left"></div>
    <div class="actions-right">{{ i18n.t("settings.personalize_page_import_export") }}</div>
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
