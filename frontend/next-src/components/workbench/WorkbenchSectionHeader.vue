<script setup lang="ts">
interface Props {
  title: string;
  description: string;
}

defineProps<Props>();
</script>

<template>
  <header class="workbench-section-header">
    <div class="workbench-section-header__copy">
      <h3 class="workbench-section-header__title">{{ title }}</h3>
      <p class="workbench-section-header__description">{{ description }}</p>
    </div>

    <div v-if="$slots.actions" class="workbench-section-header__actions">
      <slot name="actions" />
    </div>
  </header>
</template>

<style scoped>
.workbench-section-header {
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.workbench-section-header__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.workbench-section-header__title,
.workbench-section-header__description {
  margin: 0;
}

.workbench-section-header__title {
  font-size: 1.02rem;
  font-weight: 600;
  line-height: 1.2;
  color: var(--sl-text-primary);
}

.workbench-section-header__description {
  max-width: 56ch;
  font-size: 0.84rem;
  line-height: 1.45;
  color: var(--sl-text-secondary);
}

.workbench-section-header__actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 720px) {
  .workbench-section-header {
    flex-direction: column;
  }

  .workbench-section-header__actions {
    width: 100%;
  }
}
</style>
