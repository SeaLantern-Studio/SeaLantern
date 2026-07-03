<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

defineProps<{
  importBusy?: boolean;
  exportBusy?: boolean;
}>();

const emit = defineEmits<{
  (e: "export-package"): void;
  (e: "import-package"): void;
}>();
</script>

<template>
  <div class="settings-actions">
    <div class="actions-left">
      <SLButton variant="secondary" :loading="importBusy" @click="emit('import-package')">
        {{ i18n.t("settings.personalization_import") }}
      </SLButton>
      <SLButton variant="secondary" :loading="exportBusy" @click="emit('export-package')">
        {{ i18n.t("settings.personalization_export") }}
      </SLButton>
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

.actions-left {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
}
</style>
