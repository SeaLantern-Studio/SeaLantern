<script setup lang="ts">
import { computed, useAttrs } from "vue";
import { Loader2 } from "@lucide/vue";

defineOptions({
  inheritAttrs: false,
});

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

const emit = defineEmits<{
  (e: "click", event: MouseEvent): void;
}>();

const attrs = useAttrs();

const buttonClasses = computed(() => [
  `sl-button--${props.variant}`,
  `sl-button--${props.size}`,
  {
    "sl-button--disabled": props.disabled || props.loading,
    "sl-button--icon-only": props.iconOnly,
  },
]);

function handleClick(event: MouseEvent) {
  emit("click", event);
}
</script>

<template>
  <button
    v-bind="attrs"
    class="sl-button"
    :class="buttonClasses"
    :type="type"
    :disabled="disabled || loading"
    :aria-busy="loading"
    @click="handleClick"
  >
    <Loader2 v-if="loading" class="sl-button-spinner" :size="16" />
    <slot v-else />
  </button>
</template>

<style src="@styles/components/common/SLButton.css" scoped></style>
