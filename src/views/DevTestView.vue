<script setup lang="ts">
/**
 * 开发者测试工具入口页
 * 仅在开发者模式开启时由侧栏注册可达,关闭后路由仍可访问但侧栏不展示
 */
import { ref } from "vue";
import { i18n } from "@language";
import DevFrontendPanel from "@components/views/dev/DevFrontendPanel.vue";
import DevComponentsPanel from "@components/views/dev/DevComponentsPanel.vue";

type Tab = "frontend" | "components";

const activeTab = ref<Tab>("frontend");

const tabs: { key: Tab; label: string }[] = [
  { key: "frontend", label: i18n.t("dev_test.tab_frontend") },
  { key: "components", label: i18n.t("dev_test.tab_components") },
];
</script>

<template>
  <div class="dev-test-view animate-stagger-in">
    <div class="dev-test-header">
      <h1 class="dev-test-title">{{ i18n.t("dev_test.title") }}</h1>
      <p class="dev-test-desc">{{ i18n.t("dev_test.desc") }}</p>
    </div>

    <div class="dev-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="dev-tab"
        :class="{ active: activeTab === tab.key }"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div class="dev-tab-content">
      <DevFrontendPanel v-if="activeTab === 'frontend'" />
      <DevComponentsPanel v-else-if="activeTab === 'components'" />
    </div>

    <!-- 后端测试预留位,后续后端接口稳定后补 -->
    <cmz-card v-if="false" :title="i18n.t('dev_test.tab_backend')" padding="md">
      <div class="dev-placeholder">
        <p>{{ i18n.t("dev_test.backend_placeholder") }}</p>
      </div>
    </cmz-card>
  </div>
</template>

<style scoped>
.dev-test-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}
.dev-test-header {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
}
.dev-test-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--sl-text-primary);
}
.dev-test-desc {
  margin: 0;
  font-size: 13px;
  color: var(--sl-text-secondary);
  line-height: 1.75;
}
.dev-tabs {
  display: flex;
  gap: var(--sl-space-xs);
  border-bottom: 1px solid var(--sl-border);
  padding: 0 var(--sl-space-xs);
}
.dev-tab {
  padding: var(--sl-space-sm) var(--sl-space-md);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--sl-text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}
.dev-tab:hover {
  color: var(--sl-text-primary);
}
.dev-tab.active {
  color: var(--sl-primary);
  border-bottom-color: var(--sl-primary);
}
.dev-tab-content {
  flex: 1;
  min-height: 0;
}
.dev-placeholder {
  padding: var(--sl-space-xl);
  text-align: center;
  color: var(--sl-text-tertiary);
  font-size: 13px;
}
</style>
