<script setup lang="ts">
import { computed } from "vue";
import { Loader2 } from "lucide-vue-next";

interface Props {
  variant?: "primary" | "secondary" | "ghost" | "danger" | "success";
  size?: "sm" | "md" | "lg";
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
  loading?: boolean;
  iconOnly?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "primary",
  size: "md",
  type: "button",
  disabled: false,
  loading: false,
  iconOnly: false,
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
