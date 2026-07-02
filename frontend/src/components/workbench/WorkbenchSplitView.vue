<script setup lang="ts">
import WorkbenchDirectoryNav, { type WorkbenchDirectoryItem } from "./WorkbenchDirectoryNav.vue";

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
  <section class="workbench-split-view">
    <aside class="workbench-split-view__directory">
      <WorkbenchDirectoryNav
        :items="items"
        :active-id="activeId"
        :aria-label="ariaLabel"
        :ariaLabel="ariaLabel"
        @select="emit('select', $event)"
      />
    </aside>

    <div class="workbench-split-view__content">
      <slot name="content-header" />
      <slot />
    </div>
  </section>
</template>

<style scoped>
.workbench-split-view {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(184px, 228px) minmax(0, 1fr);
  gap: 28px;
  align-items: start;
}

.workbench-split-view__directory {
  min-width: 0;
  position: sticky;
  top: 2px;
}

.workbench-split-view__content {
  min-width: 0;
  display: grid;
  gap: 14px;
}

@media (max-width: 900px) {
  .workbench-split-view {
    grid-template-columns: 1fr;
  }

  .workbench-split-view__directory {
    position: static;
  }
}
</style>
