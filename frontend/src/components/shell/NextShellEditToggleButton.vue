<script setup lang="ts">
import { useTemplateRef } from "vue";
import { Check, Pencil } from "@lucide/vue";

interface Props {
  active: boolean;
  startLabel: string;
  doneLabel: string;
}

defineProps<Props>();

const buttonRef = useTemplateRef<HTMLButtonElement>("button");

const emit = defineEmits<{
  toggle: [];
}>();

function getElement(): HTMLButtonElement | null {
  return buttonRef.value;
}

defineExpose({
  getElement,
});
</script>

<template>
  <button
    ref="button"
    class="next-shell-edit-toggle"
    type="button"
    :title="active ? doneLabel : startLabel"
    :aria-pressed="active"
    @click="emit('toggle')"
  >
    <Check v-if="active" :size="16" />
    <Pencil v-else :size="16" />
  </button>
</template>

<style scoped>
.next-shell-edit-toggle {
  width: 42px;
  height: 42px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-surface) 88%, transparent);
  color: var(--sl-text-primary);
  cursor: pointer;
}

.next-shell-edit-toggle[aria-pressed="true"] {
  color: var(--sl-primary);
  border-color: color-mix(in srgb, var(--sl-primary) 36%, white);
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
}
</style>
