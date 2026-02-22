<script setup lang="ts">
import { ref } from "vue";
import { Loader2 } from "lucide-vue-next";
import { useRegisterComponent } from "@composables/useRegisterComponent";

interface Props {
  variant?: "primary" | "secondary" | "ghost" | "danger" | "success";
  size?: "sm" | "md" | "lg";
  disabled?: boolean;
  loading?: boolean;
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "primary",
  size: "md",
  disabled: false,
  loading: false,
});

const elRef = ref<HTMLElement | null>(null);
const id = props.componentId ?? `sl-button-${Math.random().toString(36).slice(2, 8)}`;
useRegisterComponent(id, {
  type: "SLButton",
  get: (prop) => (prop === "disabled" ? props.disabled : undefined),
  set: () => {},
  call: (method) => {
    if (method === "click") elRef.value?.click();
  },
  on: () => () => {},
  el: () => elRef.value,
});
</script>

<template>
  <button
    ref="elRef"
    class="sl-button"
    :class="[
      `sl-button--${variant}`,
      `sl-button--${size}`,
      { 'sl-button--disabled': disabled || loading },
    ]"
    :disabled="disabled || loading"
    :aria-busy="loading"
  >
    <Loader2 v-if="loading" class="sl-button-spinner" :size="16" />
    <slot />
  </button>
</template>

<style src="@styles/components/common/SLButton.css" scoped></style>
