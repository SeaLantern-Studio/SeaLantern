<script setup lang="ts">
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
    <cmz-button size="sm" :disabled="props.disabled" @click="emit('add')">
      {{ props.label }}
    </cmz-button>
    <cmz-button variant="ghost" size="sm" @click="emit('refresh')">
      {{ i18n.t("common.refresh") }}
    </cmz-button>
  </div>
</template>

<style scoped>
.player-action-bar {
  display: flex;
  gap: var(--sl-space-sm);
}
</style>
