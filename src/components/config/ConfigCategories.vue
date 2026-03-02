<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import { SLTabBar } from "@components/common";
import SLInput from "@components/common/SLInput.vue";

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

const tabs = computed(() =>
  props.categories.map((cat) => ({
    key: cat,
    label: categoryLabels[cat] || cat,
  })),
);
</script>

<template>
  <SLTabBar
    :modelValue="activeCategory"
    :tabs="tabs"
    :level="2"
    @update:modelValue="emit('updateCategory', $event ?? 'all')"
  >
    <template #extra>
      <SLInput
        :modelValue="searchQuery"
        :placeholder="i18n.t('config.search')"
        @input="emit('updateSearch', $event.target.value)"
        style="width: 180px"
        class="search-input"
      />
    </template>
  </SLTabBar>
</template>

<style scoped>
.search-input :deep(.sl-input) {
  padding: 6px 12px;
  font-size: 13px;
}

.search-input :deep(.sl-input-container) {
  height: 28px;
}
</style>
