<script setup lang="ts">
/**
 * 开发者测试工具入口页
 * 仅在开发者模式开启时由侧栏注册可达,关闭后路由仍可访问但侧栏不展示
 * 左侧竖向 tabbar 按容器展开,点击直接滚到对应卡片,滚动联动高亮
 */
import { ref } from "vue";
import { i18n } from "@language";
import DevFrontendPanel from "@components/views/dev/DevFrontendPanel.vue";
import DevComponentsPanel from "@components/views/dev/DevComponentsPanel.vue";

// key 对应各面板卡片上的 data-dev-section
const activeSection = ref("i18n");

const tabs = [
  { key: "i18n", label: "i18n 国际化" },
  { key: "error", label: "错误处理" },
  { key: "player_name", label: "玩家名校验" },
  { key: "json", label: "JSON 解析" },
  { key: "retry", label: "retry 重试" },
  { key: "store", label: "Store 状态快照" },
  { key: "toast", label: "Toast 通知" },
  { key: "confirm", label: "确认弹窗" },
  { key: "switch", label: "Switch 开关" },
  { key: "input", label: "Input 输入框" },
  { key: "button", label: "Button 按钮" },
  { key: "modal", label: "Modal 弹层" },
];
</script>

<template>
  <div class="dev-test-view animate-stagger-in">
    <div class="dev-test-header">
      <h1 class="dev-test-title">{{ i18n.t("dev_test.title") }}</h1>
      <p class="dev-test-desc">{{ i18n.t("dev_test.desc") }}</p>
    </div>

    <div class="dev-test-layout">
      <!-- 竖向 tabbar 按容器展开:点击滚到对应卡片,滚动更新指示 -->
      <div class="dev-tabbar-sticky">
        <cmz-tab-bar
          v-model="activeSection"
          :tabs="tabs"
          :level="1"
          vertical
          scroll-spy
          scroll-container=".app-content"
          :scroll-offset="24"
          section-selector="[data-dev-section='{key}']"
        />
      </div>

      <div class="dev-test-main">
        <DevFrontendPanel />
        <DevComponentsPanel />

        <!-- 后端测试预留位,后续后端接口稳定后补 -->
        <cmz-card v-if="false" :title="i18n.t('dev_test.tab_backend')" padding="md">
          <div class="dev-placeholder">
            <p>{{ i18n.t("dev_test.backend_placeholder") }}</p>
          </div>
        </cmz-card>
      </div>
    </div>
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
.dev-test-layout {
  display: flex;
  align-items: flex-start;
  gap: 0;
}

/* 竖 tabbar 吸顶,跟随内容滚动;宽度由 app.css 全局统一 */
.dev-tabbar-sticky {
  position: sticky;
  top: var(--sl-space-md);
  flex-shrink: 0;
  z-index: 1;
}

.dev-test-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-lg);
}

/* 点击 tab 滚动定位时卡片与容器顶部留出呼吸空间 */
.dev-test-main :deep([data-dev-section]) {
  scroll-margin-top: var(--sl-space-md);
}
.dev-placeholder {
  padding: var(--sl-space-xl);
  text-align: center;
  color: var(--sl-text-tertiary);
  font-size: 13px;
}
</style>
