<script setup lang="ts">
export interface WorkbenchDirectoryItem {
  id: string;
  label: string;
  description?: string;
  disabled?: boolean;
}

interface Props {
  items: readonly WorkbenchDirectoryItem[];
  activeId: string;
  ariaLabel: string;
}

defineProps<Props>();

const emit = defineEmits<{
  select: [id: string];
}>();
</script>

<template>
  <nav class="workbench-directory" :aria-label="ariaLabel">
    <button
      v-for="item in items"
      :key="item.id"
      type="button"
      class="workbench-directory__item"
      :class="{
        'workbench-directory__item--active': activeId === item.id,
        'workbench-directory__item--disabled': item.disabled,
      }"
      :disabled="item.disabled"
      :aria-current="activeId === item.id ? 'location' : undefined"
      @click="emit('select', item.id)"
    >
      <span class="workbench-directory__label">{{ item.label }}</span>
      <span v-if="item.description" class="workbench-directory__description">
        {{ item.description }}
      </span>
    </button>
  </nav>
</template>

<style scoped>
.workbench-directory {
  min-width: 0;
  position: relative;
  display: grid;
  gap: 4px;
  padding: 4px 0;
}

.workbench-directory::before {
  content: "";
  position: absolute;
  inset: 6px auto 6px 0;
  width: 1px;
  background: color-mix(in srgb, var(--sl-border) 62%, transparent);
}

.workbench-directory__item {
  position: relative;
  width: 100%;
  min-height: 48px;
  padding: 10px 14px 10px 16px;
  display: grid;
  gap: 3px;
  justify-items: start;
  border: none;
  border-radius: 14px;
  background: transparent;
  color: var(--sl-text-secondary);
  text-align: left;
  transition:
    background-color 0.2s ease,
    color 0.2s ease,
    opacity 0.2s ease;
}

.workbench-directory__item::before {
  content: "";
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  border-radius: 999px;
  background: transparent;
  transition: background-color 0.2s ease;
}

.workbench-directory__item:hover:not(:disabled) {
  background: color-mix(in srgb, var(--sl-primary) 5%, transparent);
  color: var(--sl-text-primary);
}

.workbench-directory__item--active {
  background: color-mix(in srgb, var(--sl-primary) 8%, transparent);
  color: var(--sl-text-primary);
}

.workbench-directory__item--active::before {
  background: var(--sl-primary);
}

.workbench-directory__item--disabled {
  opacity: 0.45;
}

.workbench-directory__item:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sl-primary) 18%, transparent);
}

.workbench-directory__label {
  font-size: 0.9rem;
  font-weight: 600;
}

.workbench-directory__description {
  font-size: 0.78rem;
  line-height: 1.4;
  color: var(--sl-text-tertiary);
}

.workbench-directory__item--active .workbench-directory__description {
  color: var(--sl-text-secondary);
}

@media (max-width: 900px) {
  .workbench-directory::before {
    display: none;
  }

  .workbench-directory {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  }

  .workbench-directory__item {
    min-height: 44px;
    padding-inline: 14px;
  }

  .workbench-directory__item::before {
    display: none;
  }
}

@media (max-width: 640px) {
  .workbench-directory {
    grid-template-columns: 1fr;
  }
}
</style>
