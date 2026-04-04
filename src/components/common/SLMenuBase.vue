<script setup lang="ts">
import type { Component } from "vue";

interface MenuItem {
  id: string | number;
  label: string;
  icon?: Component | string;
  disabled?: boolean;
  danger?: boolean;
  divider?: boolean;
}

interface Props {
  items: MenuItem[];
  header?: string;
  minWidth?: string;
  maxWidth?: string;
}

const props = withDefaults(defineProps<Props>(), {
  minWidth: "160px",
  maxWidth: "280px",
});

const emit = defineEmits<{
  select: [item: MenuItem];
}>();

const handleItemClick = (item: MenuItem) => {
  if (item.disabled) return;
  emit("select", item);
};
</script>

<template>
  <div class="sl-menu-base" :style="{ minWidth, maxWidth }">
    <div v-if="header" class="sl-menu-base-header">
      {{ header }}
    </div>
    <div class="sl-menu-base-content">
      <template v-for="item in items" :key="item.id">
        <div v-if="item.divider" class="sl-menu-base-divider" role="separator" />
        <div
          v-else
          class="sl-menu-base-item"
          :class="{
            disabled: item.disabled,
            danger: item.danger,
          }"
          role="menuitem"
          :aria-disabled="item.disabled"
          @click="handleItemClick(item)"
        >
          <component
            v-if="item.icon && typeof item.icon !== 'string'"
            :is="item.icon"
            class="sl-menu-base-icon"
            :size="16"
          />
          <span v-else-if="item.icon" class="sl-menu-base-icon-text">{{ item.icon }}</span>
          <span class="sl-menu-base-label">{{ item.label }}</span>
        </div>
      </template>
      <div v-if="items.length === 0" class="sl-menu-base-empty">
        <slot name="empty">No menu items</slot>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sl-menu-base {
  background: var(--sl-glass-bg, rgba(255, 255, 255, 0.72));
  border: 1px solid var(--sl-glass-border, rgba(255, 255, 255, 0.5));
  border-radius: var(--sl-radius-lg, 12px);
  box-shadow: var(--sl-shadow-lg);
  overflow: hidden;
  backdrop-filter: blur(var(--sl-blur-lg, 20px)) saturate(var(--sl-saturate-normal, 180%));
  -webkit-backdrop-filter: blur(var(--sl-blur-lg, 20px)) saturate(var(--sl-saturate-normal, 180%));
  will-change: backdrop-filter;
  transform: translateZ(0);
  backface-visibility: hidden;
}

[data-theme="dark"] .sl-menu-base {
  --sl-glass-bg: rgba(15, 17, 23, 0.72);
  --sl-glass-border: rgba(255, 255, 255, 0.08);
}

[data-acrylic="true"] .sl-menu-base {
  --sl-glass-bg: rgba(255, 255, 255, 0.65);
  backdrop-filter: blur(var(--sl-blur-xl, 32px)) saturate(var(--sl-saturate-normal, 180%));
  -webkit-backdrop-filter: blur(var(--sl-blur-xl, 32px)) saturate(var(--sl-saturate-normal, 180%));
}

[data-theme="dark"][data-acrylic="true"] .sl-menu-base {
  --sl-glass-bg: rgba(15, 17, 23, 0.65);
}

[data-acrylic="false"] .sl-menu-base {
  background: var(--sl-surface, #ffffff);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  will-change: auto;
}

.sl-menu-base-header {
  padding: 6px 12px;
  font-size: 0.6875rem;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.45));
  border-bottom: 1px solid var(--sl-border, rgba(255, 255, 255, 0.08));
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

[data-theme="light"] .sl-menu-base-header {
  color: var(--sl-text-tertiary, rgba(0, 0, 0, 0.4));
  border-bottom-color: var(--sl-border, rgba(0, 0, 0, 0.08));
}

.sl-menu-base-content {
  padding: var(--sl-space-xs, 4px);
}

.sl-menu-base-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm, 8px);
  padding: 10px 12px;
  border-radius: var(--sl-radius-md, 8px);
  cursor: pointer;
  color: var(--sl-text-primary, rgba(255, 255, 255, 0.9));
  font-size: 0.875rem;
  transition:
    background-color 0.15s ease,
    transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
  user-select: none;
  position: relative;
  overflow: hidden;
  animation: menu-item-fade-in 0.2s ease backwards;
}

.sl-menu-base-item:nth-child(1) {
  animation-delay: 0.02s;
}
.sl-menu-base-item:nth-child(2) {
  animation-delay: 0.04s;
}
.sl-menu-base-item:nth-child(3) {
  animation-delay: 0.06s;
}
.sl-menu-base-item:nth-child(4) {
  animation-delay: 0.08s;
}
.sl-menu-base-item:nth-child(5) {
  animation-delay: 0.1s;
}
.sl-menu-base-item:nth-child(6) {
  animation-delay: 0.12s;
}
.sl-menu-base-item:nth-child(7) {
  animation-delay: 0.14s;
}
.sl-menu-base-item:nth-child(8) {
  animation-delay: 0.16s;
}

@keyframes menu-item-fade-in {
  from {
    opacity: 0;
    transform: translateX(-8px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.sl-menu-base-item::before {
  content: "";
  position: absolute;
  inset: 0;
  background: var(--sl-primary, #0ea5e9);
  opacity: 0;
  transform: scale(0.5);
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
  border-radius: inherit;
}

.sl-menu-base-item:hover:not(.disabled) {
  background: var(--sl-surface-hover, rgba(255, 255, 255, 0.1));
}

[data-theme="light"] .sl-menu-base-item:hover:not(.disabled) {
  background: var(--sl-surface-hover, rgba(0, 0, 0, 0.05));
}

.sl-menu-base-item:active:not(.disabled)::before {
  opacity: 0.1;
  transform: scale(1);
}

.sl-menu-base-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.sl-menu-base-item.danger {
  color: var(--sl-error, #ef4444);
}

.sl-menu-base-item.danger:hover:not(.disabled) {
  background: var(--sl-error-bg, rgba(239, 68, 68, 0.1));
}

.sl-menu-base-icon {
  flex-shrink: 0;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.6));
}

.sl-menu-base-item.danger .sl-menu-base-icon {
  color: var(--sl-error, #ef4444);
}

.sl-menu-base-icon-text {
  flex-shrink: 0;
  width: 16px;
  text-align: center;
  opacity: 0.8;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.6));
}

.sl-menu-base-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sl-menu-base-item.danger .sl-menu-base-label {
  color: var(--sl-error, #ef4444);
}

.sl-menu-base-divider {
  height: 1px;
  background: var(--sl-border, rgba(255, 255, 255, 0.08));
  margin: var(--sl-space-xs, 4px) 0;
}

[data-theme="light"] .sl-menu-base-divider {
  background: var(--sl-border, rgba(0, 0, 0, 0.08));
}

.sl-menu-base-empty {
  padding: 8px 12px;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.5));
  font-size: 0.75rem;
  text-align: center;
}

[data-theme="light"] .sl-menu-base-empty {
  color: var(--sl-text-tertiary, rgba(0, 0, 0, 0.4));
}

[data-theme="light"] .sl-menu-base-item {
  color: var(--sl-text-primary, rgba(0, 0, 0, 0.85));
}
</style>
