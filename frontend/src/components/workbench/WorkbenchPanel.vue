<script setup lang="ts">
import SLCard from "@components/common/SLCard.vue";

interface Props {
  title?: string;
  description?: string;
  tone?: "default" | "danger";
}

withDefaults(defineProps<Props>(), {
  title: "",
  description: "",
  tone: "default",
});
</script>

<template>
  <SLCard
    variant="outline"
    padding="lg"
    class="workbench-panel"
    :class="`workbench-panel--${tone}`"
  >
    <template v-if="title || description || $slots.actions" #header>
      <div class="workbench-panel__header">
        <div v-if="title || description" class="workbench-panel__copy">
          <h4 v-if="title" class="workbench-panel__title">{{ title }}</h4>
          <p v-if="description" class="workbench-panel__description">{{ description }}</p>
        </div>

        <div v-if="$slots.actions" class="workbench-panel__actions">
          <slot name="actions" />
        </div>
      </div>
    </template>

    <div class="workbench-panel__body">
      <slot />
    </div>
  </SLCard>
</template>

<style scoped>
.workbench-panel {
  min-width: 0;
  display: grid;
  gap: 14px;
  border-radius: 22px;
  border-color: color-mix(in srgb, var(--sl-border) 72%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 92%, var(--sl-bg));
}

.workbench-panel--danger {
  border-color: color-mix(in srgb, var(--sl-error) 28%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-error) 5%, var(--sl-surface));
}

.workbench-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
}

.workbench-panel__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.workbench-panel__title,
.workbench-panel__description {
  margin: 0;
}

.workbench-panel__title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.workbench-panel__description {
  max-width: 58ch;
  font-size: 0.84rem;
  line-height: 1.45;
  color: var(--sl-text-secondary);
}

.workbench-panel__actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.workbench-panel__body {
  min-width: 0;
}

@media (max-width: 720px) {
  .workbench-panel__header {
    flex-direction: column;
  }

  .workbench-panel__actions {
    width: 100%;
  }
}
</style>
