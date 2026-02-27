<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import { SLTabBar } from "@components/common";
import PluginsView from "@views/PluginsView.vue";
import MarketView from "@views/MarketView.vue";

const route = useRoute();
const router = useRouter();

const activeTab = ref<"plugins" | "market">("plugins");

const tabs = computed(() => [
  { key: "plugins" as const, label: i18n.t("plugins.title") },
  { key: "market" as const, label: i18n.t("market.title") },
]);

function handleTabChange(tab: string | null) {
  if (tab) {
    activeTab.value = tab as "plugins" | "market";
    router.replace({ query: { tab } });
  }
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
  <div class="plugins-page animate-fade-in-up">
    <SLTabBar v-model="activeTab" :tabs="tabs" :level="1" @update:modelValue="handleTabChange" />

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
  gap: var(--sl-space-md);
  height: 100%;
}

.tab-content {
  flex: 1;
  overflow: auto;
}
</style>
