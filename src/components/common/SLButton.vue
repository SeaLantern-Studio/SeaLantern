<script setup lang="ts">
import { ref, computed } from "vue";
import { Loader2 } from "lucide-vue-next";
import { useRegisterComponent } from "@composables/useRegisterComponent";

interface Props {
  variant?: "primary" | "secondary" | "ghost" | "danger" | "success";
  size?: "sm" | "md" | "lg";
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
  loading?: boolean;
  iconOnly?: boolean;
  componentId?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "primary",
  size: "md",
  type: "button",
  disabled: false,
  loading: false,
  iconOnly: false,
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

const buttonClasses = computed(() => [
  `sl-button--${props.variant}`,
  `sl-button--${props.size}`,
  {
    "sl-button--disabled": props.disabled || props.loading,
    "sl-button--icon-only": props.iconOnly,
  },
]);
</script>

<template>
  <button
    ref="elRef"
    class="sl-button"
    :class="buttonClasses"
    :type="type"
    :disabled="disabled || loading"
    :aria-busy="loading"
  >
    <Loader2 v-if="loading" class="sl-button-spinner" :size="16" />
    <slot v-else />
  </button>
</template>

<style src="@styles/components/common/SLButton.css" scoped></style>
