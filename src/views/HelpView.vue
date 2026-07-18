<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { i18n } from "@language";
import { ExternalLink, Menu, X } from "lucide-vue-next";
import { helpDocs, introFeatures, introTop, introFooter } from "@data/helpDocs";

// 文档页面配置
const docPages = [
  { key: "intro", label: "项目简介" },
  { key: "download", label: "下载安装" },
  { key: "getting-started", label: "快速开始" },
  { key: "server-jar", label: "核心获取" },
  { key: "tutorial", label: "使用教程" },
  { key: "features", label: "功能总览" },
  { key: "faq", label: "常见问题" },
  { key: "contributor", label: "贡献者" },
];

const currentSection = ref("intro");
const sidebarOpen = ref(true);
const isMobile = ref(false);

// 是否是项目简介页面（需渲染卡片）
const isIntro = computed(() => currentSection.value === "intro");

// 其他页面直接从静态数据获取内容
const contentMd = computed(() => (isIntro.value ? "" : (helpDocs[currentSection.value] ?? "")));

// 切换页面
function switchSection(key: string) {
  currentSection.value = key;
  if (isMobile.value) sidebarOpen.value = false;
}

// 在浏览器打开
function openInBrowser() {
  const url = `https://docs.ideaflash.cn/zh/${currentSection.value}`;
  window.open(url, "_blank");
}

// 检测移动端
function checkMobile() {
  isMobile.value = window.innerWidth < 768;
  if (!isMobile.value) sidebarOpen.value = true;
}

onMounted(() => {
  checkMobile();
  window.addEventListener("resize", checkMobile);
});
</script>

<template>
  <div class="help-view">
    <!-- 移动端侧栏切换 -->
    <button v-if="isMobile" class="sidebar-toggle" @click="sidebarOpen = !sidebarOpen">
      <Menu v-if="!sidebarOpen" :size="20" />
      <X v-else :size="20" />
    </button>

    <!-- 侧栏 TabBar 导航 -->
    <aside class="help-sidebar" :class="{ open: sidebarOpen }">
      <div class="sidebar-header">
        <span class="sidebar-title">{{ i18n.t("help.title") }}</span>
      </div>
      <cmz-tab-bar
        v-model="currentSection"
        :tabs="docPages"
        :level="1"
        vertical
        class="sidebar-nav"
      />
      <div class="sidebar-footer">
        <cmz-button variant="outline" size="sm" class="open-browser-btn" @click="openInBrowser">
          <ExternalLink :size="16" />
          <span>在浏览器打开</span>
        </cmz-button>
      </div>
    </aside>

    <!-- 内容区域 -->
    <main class="help-content">
      <!-- 项目简介：顶部 + 特性卡片网格 + 底部 -->
      <template v-if="isIntro">
        <cmz-markdown :content="introTop" variant="plain" />
        <h2 class="features-heading">特性</h2>
        <div class="feature-grid">
          <div v-for="feature in introFeatures" :key="feature.title" class="feature-card">
            <h3 class="feature-title">{{ feature.title }}</h3>
            <p class="feature-desc">{{ feature.desc }}</p>
            <p class="feature-note">{{ feature.note }}</p>
          </div>
        </div>
        <cmz-markdown :content="introFooter" variant="plain" />
      </template>
      <!-- 其他页面 -->
      <cmz-markdown v-else :content="contentMd" variant="plain" />
    </main>
  </div>
</template>

<style scoped>
.help-view {
  display: flex;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.sidebar-toggle {
  position: fixed;
  top: var(--sl-space-md);
  left: var(--sl-space-md);
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: var(--sl-bg-elevated);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  color: var(--sl-text);
  cursor: pointer;
  transition: all 0.2s ease;
}

.sidebar-toggle:hover {
  background: var(--sl-bg-hover);
}

.help-sidebar {
  width: 220px;
  height: 100%;
  background: var(--sl-bg-elevated);
  border-right: 1px solid var(--sl-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: transform 0.3s ease;
}

.sidebar-header {
  padding: var(--sl-space-md);
  border-bottom: 1px solid var(--sl-border);
  flex-shrink: 0;
}

.sidebar-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--sl-text);
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  margin: 0 !important;
  border: none !important;
}

.sidebar-footer {
  padding: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border);
  flex-shrink: 0;
}

.open-browser-btn {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--sl-space-xs);
}

.help-content {
  flex: 1;
  height: 100%;
  overflow-y: auto;
  padding: var(--sl-space-2xl) var(--sl-space-2xl);
  scroll-behavior: smooth;
}

.help-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  gap: var(--sl-space-md);
  color: var(--sl-text-tertiary);
}

@media (max-width: 768px) {
  .help-sidebar {
    position: fixed;
    left: 0;
    top: 0;
    height: 100%;
    z-index: 50;
    transform: translateX(-100%);
    box-shadow: var(--sl-shadow-lg);
  }

  .help-sidebar.open {
    transform: translateX(0);
  }

  .help-content {
    padding: var(--sl-space-lg);
    padding-top: calc(48px + var(--sl-space-lg));
  }
}

/* 特性卡片网格 */
.features-heading {
  font-size: var(--sl-font-size-2xl);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: var(--sl-space-lg) 0 var(--sl-space-md);
  padding-bottom: var(--sl-space-sm);
  border-bottom: 1px solid var(--sl-border-light);
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-lg);
}

.feature-card {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-lg);
  backdrop-filter: blur(var(--sl-blur-sm, 8px));
  -webkit-backdrop-filter: blur(var(--sl-blur-sm, 8px));
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.feature-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.feature-title {
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-xs);
}

.feature-desc {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
  line-height: 1.6;
  margin: 0 0 var(--sl-space-xs);
}

.feature-note {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  line-height: 1.5;
  margin: 0;
}

@media (max-width: 768px) {
  .help-sidebar {
    position: fixed;
    left: 0;
    top: 0;
    height: 100%;
    z-index: 50;
    transform: translateX(-100%);
    box-shadow: var(--sl-shadow-lg);
  }

  .help-sidebar.open {
    transform: translateX(0);
  }

  .help-content {
    padding: var(--sl-space-lg);
    padding-top: calc(48px + var(--sl-space-lg));
  }
}
</style>
