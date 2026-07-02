<script setup lang="ts">
interface Props {
  tone?: "neutral" | "error" | "info" | "warning";
}

withDefaults(defineProps<Props>(), {
  tone: "neutral",
});
</script>

<template>
  <section class="workbench-status-banner" :class="`workbench-status-banner--${tone}`">
    <div class="workbench-status-banner__copy">
      <slot />
    </div>

    <div v-if="$slots.actions" class="workbench-status-banner__actions">
      <slot name="actions" />
    </div>
  </section>
</template>

<style scoped>
.workbench-status-banner {
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-bg-secondary) 72%, transparent);
  color: var(--sl-text-secondary);
}

.workbench-status-banner--error {
  border-color: rgba(239, 68, 68, 0.22);
  background: rgba(239, 68, 68, 0.1);
  color: var(--sl-error);
}

.workbench-status-banner--info {
  border-color: color-mix(in srgb, var(--sl-primary) 20%, transparent);
  background: color-mix(in srgb, var(--sl-primary) 10%, var(--sl-surface));
  color: var(--sl-text-primary);
}

.workbench-status-banner--warning {
  border-color: color-mix(in srgb, var(--sl-warning) 28%, transparent);
  background: color-mix(in srgb, var(--sl-warning) 10%, var(--sl-surface));
  color: var(--sl-text-primary);
}

.workbench-status-banner__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.workbench-status-banner__actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 720px) {
  .workbench-status-banner {
    flex-direction: column;
  }
}
</style>
