<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
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

const tabIndicator = ref<HTMLElement | null>(null);

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

// 更新标签指示器位置
function updateTabIndicator() {
  setTimeout(() => {
    if (!tabIndicator.value) return;

    const activeCategoryItem = document.querySelector(".category-item.active");
    if (activeCategoryItem) {
      const { offsetLeft, offsetWidth } = activeCategoryItem as HTMLElement;
      tabIndicator.value.style.left = `${offsetLeft}px`;
      tabIndicator.value.style.width = `${offsetWidth}px`;
    }
  }, 100); // 添加延迟，确保DOM已完全渲染
}

// 监听分类变化，更新指示器位置
watch(
  () => props.activeCategory,
  () => {
    updateTabIndicator();
  },
);

// 监听分类列表变化，更新指示器位置
watch(
  () => props.categories,
  () => {
    updateTabIndicator();
  },
  { deep: true },
);

// 组件挂载后初始化指示器位置
onMounted(() => {
  updateTabIndicator();
});
</script>

<template>
  <div class="config-categories">
    <div class="categories-container">
      <div class="tab-indicator" ref="tabIndicator"></div>
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
        style="width: 180px"
        class="search-input"
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
  position: relative;
  overflow: hidden;
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

.tab-indicator {
  position: absolute;
  top: 0;
  bottom: 0;
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all 0.3s ease;
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}

.category-item {
  padding: 4px 12px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
  border: 1px solid transparent;
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  color: var(--sl-text-secondary);
}

.category-item:hover {
  color: var(--sl-text-primary);
}

.category-item.active {
  color: var(--sl-primary);
}

.search-input :deep(.sl-input) {
  padding: 6px 12px;
  font-size: 13px;
}

.search-input :deep(.sl-input-container) {
  height: 28px;
}
</style>
