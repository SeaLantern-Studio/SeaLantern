<script setup lang="ts">
import { computed, useSlots } from "vue";
import { useRegisterNextProtectedShell } from "@src/composables/useNextProtectedShell";

interface Props {
  railLocked?: boolean;
  shellHeaderMode?: "title" | "hidden";
}

const props = withDefaults(defineProps<Props>(), {
  railLocked: false,
  shellHeaderMode: "title",
});

const slots = useSlots();

const railLocked = computed(() => props.railLocked);
const shellHeaderMode = computed(() => props.shellHeaderMode);
const headerPrimaryActions = computed(() => {
  if (!slots["header-primary-actions"]) {
    return null;
  }

  return () => slots["header-primary-actions"]?.() ?? [];
});

useRegisterNextProtectedShell({
  railLocked,
  shellHeaderMode,
  headerPrimaryActions,
});
</script>

<template>
  <slot />
</template>
