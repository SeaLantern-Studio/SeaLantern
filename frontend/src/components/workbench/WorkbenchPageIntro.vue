<script setup lang="ts">
interface Props {
  eyebrow?: string;
  title: string;
  description: string;
}

withDefaults(defineProps<Props>(), {
  eyebrow: "",
});
</script>

<template>
  <header class="workbench-page-intro">
    <div class="workbench-page-intro__copy">
      <p v-if="eyebrow" class="workbench-page-intro__eyebrow">{{ eyebrow }}</p>
      <h2 class="workbench-page-intro__title">{{ title }}</h2>
      <p class="workbench-page-intro__description">{{ description }}</p>
    </div>

    <div v-if="$slots.actions" class="workbench-page-intro__actions">
      <slot name="actions" />
    </div>
  </header>
</template>

<style scoped>
.workbench-page-intro {
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.workbench-page-intro__copy {
  min-width: 0;
  display: grid;
  gap: 4px;
}

.workbench-page-intro__eyebrow,
.workbench-page-intro__title,
.workbench-page-intro__description {
  margin: 0;
}

.workbench-page-intro__eyebrow {
  font-size: 0.76rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.workbench-page-intro__title {
  font-size: 1.22rem;
  font-weight: 600;
  line-height: 1.12;
  letter-spacing: -0.02em;
  color: var(--sl-text-primary);
}

.workbench-page-intro__description {
  max-width: 58ch;
  font-size: 0.87rem;
  line-height: 1.45;
  color: var(--sl-text-secondary);
}

.workbench-page-intro__actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  flex-wrap: wrap;
}

@media (max-width: 720px) {
  .workbench-page-intro {
    flex-direction: column;
  }

  .workbench-page-intro__actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
