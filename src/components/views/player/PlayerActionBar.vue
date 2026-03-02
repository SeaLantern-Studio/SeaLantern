<script setup lang="ts">
import SLButton from "@components/common/SLButton.vue";
import { i18n } from "@language";

const props = defineProps<{
  disabled?: boolean;
  label: string;
}>();

const emit = defineEmits<{
  (e: "add"): void;
  (e: "refresh"): void;
}>();
</script>

<template>
  <div class="player-action-bar">
    <SLButton variant="primary" size="sm" :disabled="props.disabled" @click="emit('add')">
      {{ props.label }}
    </SLButton>
    <SLButton variant="ghost" size="sm" @click="emit('refresh')">
      {{ i18n.t("common.refresh") }}
    </SLButton>
  </div>
</template>

<style scoped>
.player-action-bar {
  display: flex;
  gap: var(--sl-space-sm);
}
</style>
