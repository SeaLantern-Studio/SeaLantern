<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
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
      <SLButton variant="ghost" size="sm" @click="emit('export')">{{
        i18n.t("settings.export")
      }}</SLButton>
      <SLButton variant="ghost" size="sm" @click="emit('import')">{{
        i18n.t("settings.import")
      }}</SLButton>
      <SLButton variant="danger" size="sm" @click="emit('reset')">{{
        i18n.t("settings.reset")
      }}</SLButton>
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
