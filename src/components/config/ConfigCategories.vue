<script setup lang="ts">
import { i18n } from "../../locales";
import SLInput from "../common/SLInput.vue";

interface Props {
  categories: string[];
  activeCategory: string;
  searchQuery: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "updateCategory", category: string): void;
  (e: "updateSearch", value: string): void;
}>();

const categoryLabels: Record<string, string> = {
  all: i18n.t("common.config_all"),
  network: i18n.t("common.config_network"),
  player: i18n.t("common.config_player"),
  game: i18n.t("common.config_game"),
  world: i18n.t("common.config_world"),
  performance: i18n.t("common.config_performance"),
  display: i18n.t("common.config_display"),
  other: i18n.t("common.config_other"),
};
</script>

<template>
  <div class="config-categories">
    <div class="categories-container">
      <div
        v-for="cat in categories"
        :key="cat"
        class="category-item"
        :class="{ active: activeCategory === cat }"
        @click="emit('updateCategory', cat)"
      >
        {{ categoryLabels[cat] || cat }}
      </div>
    </div>
    <div class="search-container">
      <SLInput
        :modelValue="searchQuery"
        :placeholder="i18n.t('config.search')"
        @input="emit('updateSearch', $event.target.value)"
        style="width: 220px"
      />
    </div>
  </div>
</template>

<style scoped>
.config-categories {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-sm);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  margin-bottom: var(--sl-space-md);
  overflow-x: auto;
  scrollbar-width: thin;
}

.search-container {
  flex-shrink: 0;
}

.categories-container {
  display: flex;
  gap: var(--sl-space-xs);
  flex: 1;
  overflow-x: auto;
  scrollbar-width: thin;
}

.categories-container::-webkit-scrollbar {
  height: 4px;
}

.categories-container::-webkit-scrollbar-track {
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-full);
}

.categories-container::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: var(--sl-radius-full);
}

.config-categories::-webkit-scrollbar {
  height: 4px;
}

.config-categories::-webkit-scrollbar-track {
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-full);
}

.config-categories::-webkit-scrollbar-thumb {
  background: var(--sl-border);
  border-radius: var(--sl-radius-full);
}

.category-item {
  padding: 4px 12px;
  border-radius: var(--sl-radius-full);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
  border: 1px solid transparent;
  transition: all var(--sl-transition-fast);
}

.category-item:hover {
  background: var(--sl-bg-secondary);
  border-color: var(--sl-border);
}

.category-item.active {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-color: var(--sl-primary);
}
</style>
