<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";

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
  difference: i18n.t("config.compare.different"),
};

const tabs = computed(() =>
  props.categories.map((cat) => ({
    key: cat,
    label: categoryLabels[cat] || cat,
  })),
);
</script>

<template>
  <cmz-tab-bar
    :modelValue="activeCategory"
    :tabs="tabs"
    :level="2"
    class="config-categories-bar"
    @update:modelValue="emit('updateCategory', $event ?? 'all')"
  >
    <template #extra>
      <cmz-input
        :modelValue="searchQuery"
        :placeholder="i18n.t('config.search')"
        @input="emit('updateSearch', $event.target.value)"
        style="min-width: 120px; max-width: 200px"
        class="search-input"
      />
    </template>
  </cmz-tab-bar>
</template>

<style scoped>
/* 搜索输入框样式 */
.search-input :deep(.cmz-input) {
  padding: 6px 12px;
  font-size: 13px;
}

.search-input :deep(.cmz-input-container) {
  height: 28px;
  width: 100%;
}

.search-input {
  min-width: 100px;
  max-width: 200px;
  flex-shrink: 1;
}

/* Tab 栏容器：空间不足时搜索框换行 */
.config-categories-bar {
  flex-wrap: wrap;
  gap: var(--sl-space-xs);
}
</style>
