<script setup lang="ts">
import { i18n } from "@language";

defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: "update:visible", value: boolean): void;
  (e: "confirm"): void;
}>();

function close() {
  emit("update:visible", false);
}
</script>

<template>
  <cmz-modal :visible="visible" :title="i18n.t('settings.reset_title')" @close="close">
    <p class="text-body">{{ i18n.t("settings.reset_desc") }}</p>
    <template #footer>
      <cmz-button variant="outline" @click="close">{{ i18n.t("settings.cancel") }}</cmz-button>
      <cmz-button variant="solid" color="#ef4444" @click="emit('confirm')">{{
        i18n.t("settings.reset_confirm")
      }}</cmz-button>
    </template>
  </cmz-modal>
</template>

<style scoped>
.text-body {
  font-size: 0.875rem;
  color: var(--sl-text-primary);
  line-height: 1.6;
}
</style>
