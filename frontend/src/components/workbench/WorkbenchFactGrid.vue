<script setup lang="ts">
export interface WorkbenchFactItem {
  label: string;
  value: string;
  detail?: string;
  tone?: "default" | "success" | "warning" | "danger";
}

interface Props {
  items: readonly WorkbenchFactItem[];
}

defineProps<Props>();
</script>

<template>
  <div class="workbench-fact-grid">
    <article
      v-for="item in items"
      :key="`${item.label}:${item.value}`"
      class="workbench-fact-grid__item"
      :class="`workbench-fact-grid__item--${item.tone ?? 'default'}`"
    >
      <span class="workbench-fact-grid__label">{{ item.label }}</span>
      <strong class="workbench-fact-grid__value">{{ item.value }}</strong>
      <span v-if="item.detail" class="workbench-fact-grid__detail">{{ item.detail }}</span>
    </article>
  </div>
</template>

<style scoped>
.workbench-fact-grid {
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.workbench-fact-grid__item {
  min-width: 0;
  display: grid;
  gap: 6px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 68%, transparent);
}

.workbench-fact-grid__item--success {
  border-color: color-mix(in srgb, var(--sl-success) 24%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-success) 8%, var(--sl-bg-secondary));
}

.workbench-fact-grid__item--warning {
  border-color: color-mix(in srgb, var(--sl-warning) 24%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-warning) 8%, var(--sl-bg-secondary));
}

.workbench-fact-grid__item--danger {
  border-color: color-mix(in srgb, var(--sl-error) 24%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-error) 8%, var(--sl-bg-secondary));
}

.workbench-fact-grid__label {
  font-size: 0.78rem;
  color: var(--sl-text-tertiary);
}

.workbench-fact-grid__value {
  color: var(--sl-text-primary);
  font-size: 0.98rem;
  line-height: 1.3;
  word-break: break-word;
}

.workbench-fact-grid__detail {
  color: var(--sl-text-secondary);
  font-size: 0.8rem;
  line-height: 1.45;
  word-break: break-word;
}
</style>
