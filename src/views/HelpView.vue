<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { i18n } from "@language";
import { ExternalLink, Menu, X } from "lucide-vue-next";
import { fetchDocContent, getDocUrl } from "@utils/docLoader";

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
const loading = ref(false);
const contentMd = ref("");

// 从 GitHub 获取原始 Markdown
async function fetchDoc(key: string) {
  loading.value = true;
  contentMd.value = "";
  try {
    contentMd.value = await fetchDocContent(key);
  } catch {
    contentMd.value = `无法加载文档内容，请检查网络连接或[在浏览器中查看](${getDocUrl(key)})。`;
  } finally {
    loading.value = false;
  }
}

// 切换页面
watch(currentSection, (key) => {
  fetchDoc(key);
  if (isMobile.value) sidebarOpen.value = false;
});

// 在浏览器打开
function openInBrowser() {
  window.open(getDocUrl(currentSection.value), "_blank");
}

// 检测移动端
function checkMobile() {
  isMobile.value = window.innerWidth < 768;
  if (!isMobile.value) sidebarOpen.value = true;
}

onMounted(() => {
  checkMobile();
  window.addEventListener("resize", checkMobile);
  fetchDoc(currentSection.value);
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
      <div v-if="loading" class="help-loading">
        <cmz-spinner size="sm" />
        <span>加载中...</span>
      </div>
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
</style>
