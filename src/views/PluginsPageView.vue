<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "../language";
import { useTabIndicator } from "../composables/useTabIndicator";
import PluginsView from "./PluginsView.vue";
import MarketView from "./MarketView.vue";

const route = useRoute();
const router = useRouter();

const activeTab = ref<"plugins" | "market">("plugins");
const { indicatorRef: tabIndicator, updatePosition: updateTabIndicator } =
  useTabIndicator(activeTab);

const localeRef = i18n.getLocaleRef();
watch(localeRef, () => {
  updateTabIndicator();
});

const tabs = computed(() => [
  { key: "plugins" as const, label: i18n.t("common.plugins") || "插件" },
  { key: "market" as const, label: i18n.t("common.market") || "插件市场" },
]);

function switchTab(tab: "plugins" | "market") {
  activeTab.value = tab;
  router.replace({ query: { tab } });
  updateTabIndicator();
}

onMounted(() => {
  const tabParam = route.query.tab;
  if (tabParam === "market") {
    activeTab.value = "market";
  } else if (tabParam === "plugins") {
    activeTab.value = "plugins";
  }
});

watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab === "market") {
      activeTab.value = "market";
    } else if (newTab === "plugins") {
      activeTab.value = "plugins";
    }
  },
);
</script>

<template>
  <div class="plugins-page">
    <div class="tab-bar">
      <div class="tab-indicator" ref="tabIndicator"></div>
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="['tab-btn', { active: activeTab === tab.key }]"
        @click="switchTab(tab.key)"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="tab-content">
      <PluginsView v-if="activeTab === 'plugins'" />
      <MarketView v-else-if="activeTab === 'market'" />
    </div>
  </div>
</template>

<style scoped>
.plugins-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: var(--sl-space-md);
  gap: var(--sl-space-md);
}

.tab-bar {
  display: flex;
  gap: var(--sl-space-xs);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-xs);
  width: fit-content;
  position: relative;
  overflow: hidden;
}

.tab-indicator {
  position: absolute;
  top: var(--sl-space-xs);
  bottom: var(--sl-space-xs);
  background: var(--sl-primary-bg);
  border-radius: var(--sl-radius-sm);
  transition: all 0.3s ease;
  box-shadow: var(--sl-shadow-sm);
  z-index: 1;
  border: 1px solid var(--sl-primary);
  opacity: 0.9;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: 8px 18px;
  border-radius: var(--sl-radius-sm);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  position: relative;
  z-index: 2;
  white-space: nowrap;
}

.tab-btn:hover {
  color: var(--sl-text-primary);
}

.tab-btn.active {
  color: var(--sl-primary);
}

.tab-content {
  flex: 1;
  overflow: auto;
}
</style>
