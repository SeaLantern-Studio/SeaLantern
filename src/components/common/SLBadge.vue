<script setup lang="ts">
import { computed } from 'vue';

export type BadgeVariant = "primary" | "success" | "warning" | "error" | "neutral" | "info";
export type BadgeSize = "small" | "medium" | "large";

interface Props {
  text: string;
  variant?: BadgeVariant;
  size?: BadgeSize;
  rounded?: boolean | "full" | "medium";
  icon?: string;
  closable?: boolean;
  maxWidth?: number | string;
}

interface Emits {
  (e: 'close'): void;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "primary",
  size: "medium",
  rounded: true,
  closable: false,
});

const emit = defineEmits<Emits>();


const badgeStyle = computed(() => {
  const style: Record<string, string> = {};
  if (props.maxWidth) {
    style.maxWidth = typeof props.maxWidth === 'number' 
      ? `${props.maxWidth}px` 
      : props.maxWidth;
  }
  return style;
});


const handleClose = (event: MouseEvent) => {
  event.stopPropagation();
  emit('close');
};
</script>

<template>
  <span
    class="sl-badge"
    :class="[
      `sl-badge--${variant}`,
      `sl-badge--${size}`,
      {
        'sl-badge--rounded': rounded !== false,
        'sl-badge--rounded-full': rounded === 'full',
        'sl-badge--rounded-medium': rounded === 'medium',
        'sl-badge--has-icon': icon,
        'sl-badge--closable': closable,
      }
    ]"
    :style="badgeStyle"
    :title="text"
    :aria-label="text"
    role="status"
  >
    <i v-if="icon" class="sl-badge__icon" :class="`icon-${icon}`" aria-hidden="true" />
    <span class="sl-badge__text" :class="{ 'sl-badge__text--truncate': maxWidth }">
      {{ text }}
    </span>
    <button
      v-if="closable"
      class="sl-badge__close"
      @click="handleClose"
      @keydown.enter="handleClose"
      @keydown.space="handleClose"
      type="button"
      aria-label="关闭标签"
      tabindex="0"
    >
      <span class="sl-badge__close-icon" aria-hidden="true">×</span>
    </button>
  </span>
</template>

<style scoped lang="scss">

.sl-badge {
  --badge-padding-x: 8px;
  --badge-padding-y: 2px;
  --badge-font-size: 0.75rem;
  --badge-line-height: 1;
  --badge-radius: 9999px;
  --badge-gap: 4px;

  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--badge-padding-y) var(--badge-padding-x);
  font-size: var(--badge-font-size);
  font-weight: 500;
  line-height: var(--badge-line-height);
  border-radius: var(--badge-radius);
  gap: var(--badge-gap);
  vertical-align: middle;
  transition: background-color 0.2s ease, color 0.2s ease;
  box-sizing: border-box;
  user-select: none;

  &--small {
    --badge-padding-x: 6px;
    --badge-padding-y: 1px;
    --badge-font-size: 0.6875rem;
    --badge-gap: 3px;
  }

  &--large {
    --badge-padding-x: 10px;
    --badge-padding-y: 4px;
    --badge-font-size: 0.8125rem;
    --badge-gap: 6px;
  }

  &--rounded-medium {
    --badge-radius: 6px;
  }

  &--rounded-full {
    --badge-radius: 9999px;
  }

  &--rounded:not(.sl-badge--rounded-full):not(.sl-badge--rounded-medium) {
    --badge-radius: 4px;
  }

  &--primary {
    background-color: rgb(var(--sl-primary-bg) / 0.1);
    color: rgb(var(--sl-primary));
    border: 1px solid rgb(var(--sl-primary) / 0.2);
  }

  &--success {
    background-color: rgba(34, 197, 94, 0.1);
    color: rgb(var(--sl-success));
    border: 1px solid rgb(var(--sl-success) / 0.2);
  }

  &--warning {
    background-color: rgba(245, 158, 11, 0.1);
    color: rgb(var(--sl-warning));
    border: 1px solid rgb(var(--sl-warning) / 0.2);
  }

  &--error {
    background-color: rgba(239, 68, 68, 0.1);
    color: rgb(var(--sl-error));
    border: 1px solid rgb(var(--sl-error) / 0.2);
  }

  &--neutral {
    background-color: rgb(var(--sl-bg-tertiary));
    color: rgb(var(--sl-text-secondary));
    border: 1px solid rgb(var(--sl-border));
  }

  &--info {
    background-color: rgba(59, 130, 246, 0.1);
    color: rgb(59, 130, 246);
    border: 1px solid rgb(59, 130, 246 / 0.2);
  }

  &__icon {
    display: inline-flex;
    align-items: center;
    font-size: 0.875em;
  }

  &__text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;

    &--truncate {
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  }

  &__close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    padding: 0;
    margin-left: var(--badge-gap);
    background: transparent;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    color: inherit;
    opacity: 0.6;
    transition: opacity 0.2s ease, background-color 0.2s ease;
    font-size: 0.875em;

    &:hover,
    &:focus-visible {
      opacity: 1;
      background-color: rgb(0 0 0 / 0.1);
      outline: 2px solid rgb(var(--sl-primary) / 0.3);
      outline-offset: 1px;
    }

    &:active {
      transform: scale(0.95);
    }

    &-icon {
      line-height: 1;
    }
  }

  &:hover {
    &.sl-badge--closable {
      padding-right: calc(var(--badge-padding-x) - 2px);
    }
  }

  &[aria-disabled="true"] {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (prefers-contrast: high) {
    border-width: 2px;
  }
}
</style>