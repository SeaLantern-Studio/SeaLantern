<script setup lang="ts">
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
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
  <SLModal :visible="visible" :title="i18n.t('settings.reset_confirm')" @close="close">
    <p class="text-body">{{ i18n.t("settings.reset_desc") }}</p>
    <template #footer>
      <SLButton variant="secondary" @click="close">{{ i18n.t("settings.cancel") }}</SLButton>
      <SLButton variant="danger" @click="emit('confirm')">{{
        i18n.t("settings.confirm_reset")
      }}</SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.text-body {
  font-size: 0.875rem;
  color: var(--sl-text-primary);
  line-height: 1.6;
}
</style>
