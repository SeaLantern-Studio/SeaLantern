<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { i18n } from "@language";
import { ExternalLink, Menu, X, Download, ChevronRight } from "lucide-vue-next";
import {
  helpDocs,
  introFeatures,
  introTop,
  introFooter,
  downloadPlatforms,
  serverTypes,
  gettingStartedSteps,
  featureItems,
  pluginRecommendations,
  memorySuggestions,
  configItems,
  getTutorialSegments,
  type DownloadPlatform,
  type ServerType,
  type StepItem,
  type FeatureItem,
  type PluginRecommendation,
  type MemorySuggestion,
  type ConfigItem,
  type TutorialSegment,
} from "@data/helpDocs";
import { openUrl } from "@tauri-apps/plugin-opener";
import { isBrowserEnv } from "@api/tauri";

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

// 页面标记（需自定义渲染的页面）
const pageType = computed<
  "intro" | "download" | "server-jar" | "getting-started" | "features" | "tutorial" | "other"
>(() => {
  const key = currentSection.value;
  if (key === "intro") return "intro";
  if (key === "download") return "download";
  if (key === "server-jar") return "server-jar";
  if (key === "getting-started") return "getting-started";
  if (key === "features") return "features";
  if (key === "tutorial") return "tutorial";
  return "other";
});

// 其他页面直接从静态数据获取内容
const contentMd = computed(() =>
  pageType.value !== "other" ? "" : (helpDocs[currentSection.value] ?? ""),
);

// 使用教程的分段内容（MD + 卡片穿插）
const tutorialSegments = computed<TutorialSegment[]>(() => getTutorialSegments());

// 切换页面
function switchSection(key: string) {
  currentSection.value = key;
  if (isMobile.value) sidebarOpen.value = false;
}

// 在浏览器打开
function openInBrowser() {
  const url = `https://docs.ideaflash.cn/zh/${currentSection.value}`;
  if (isBrowserEnv()) {
    window.open(url, "_blank");
  } else {
    openUrl(url);
  }
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
      <template v-if="pageType === 'intro'">
        <cmz-markdown :content="introTop" variant="plain" />
        <h2 class="section-heading">特性</h2>
        <div class="feature-grid">
          <div v-for="feature in introFeatures" :key="feature.title" class="feature-card">
            <h3 class="feature-title">{{ feature.title }}</h3>
            <p class="feature-desc">{{ feature.desc }}</p>
            <p class="feature-note">{{ feature.note }}</p>
          </div>
        </div>
        <cmz-markdown :content="introFooter" variant="plain" />
      </template>

      <!-- 下载安装：平台卡片 -->
      <template v-else-if="pageType === 'download'">
        <h1 class="page-title">下载安装</h1>
        <p class="page-subtitle">当前最新版本：<strong>v1.3.0</strong></p>
        <div v-for="platform in downloadPlatforms" :key="platform.name" class="platform-section">
          <h2 class="section-heading">{{ platform.name }}</h2>
          <p class="platform-subtitle">{{ platform.subtitle }}</p>
          <div class="download-grid">
            <div v-for="item in platform.items" :key="item.format" class="download-card">
              <div class="download-card-body">
                <h3 class="download-format">{{ item.format }}</h3>
                <p class="download-desc">{{ item.desc }}</p>
              </div>
              <a :href="item.url" class="download-btn" target="_blank" rel="noreferrer">
                <Download :size="16" />
                <span>下载</span>
              </a>
            </div>
          </div>
          <p v-if="platform.notes" class="platform-notes">{{ platform.notes }}</p>
        </div>
        <cmz-markdown :content="helpDocs['download']" variant="plain" />
      </template>

      <!-- 核心获取：服务端类型对比卡片 -->
      <template v-else-if="pageType === 'server-jar'">
        <h1 class="page-title">核心获取</h1>
        <p class="page-subtitle">Minecraft 服务器需要一个服务端核心（JAR 文件）才能运行。</p>
        <div class="server-grid">
          <div v-for="server in serverTypes" :key="server.name" class="server-card">
            <div class="server-card-header">
              <h2 class="server-name">{{ server.name }}</h2>
              <div class="server-tags">
                <span
                  v-for="tag in server.tags"
                  :key="tag"
                  class="server-tag"
                  :class="{ 'tag-recommend': tag === '推荐' }"
                  >{{ tag }}</span
                >
              </div>
            </div>
            <p class="server-desc">{{ server.desc }}</p>
            <div class="server-stars">
              <span>性能</span>
              <span class="stars"
                >{{ "★".repeat(server.performance) }}{{ "☆".repeat(4 - server.performance) }}</span
              >
              <span>推荐</span>
              <span class="stars"
                >{{ "★".repeat(server.recommendation)
                }}{{ "☆".repeat(4 - server.recommendation) }}</span
              >
            </div>
            <div class="server-compat">
              <span class="compat-label">插件</span>
              <span>{{ server.pluginCompat }}</span>
              <span class="compat-label">Mod</span>
              <span>{{ server.modCompat }}</span>
            </div>
            <ul v-if="server.pros.length" class="server-pros">
              <li v-for="pro in server.pros" :key="pro">{{ pro }}</li>
            </ul>
            <ul v-if="server.cons.length" class="server-cons">
              <li v-for="con in server.cons" :key="con">{{ con }}</li>
            </ul>
            <a :href="server.url" class="server-link" target="_blank" rel="noreferrer">
              <span>前往下载</span>
              <ChevronRight :size="14" />
            </a>
          </div>
        </div>
        <cmz-markdown :content="helpDocs['server-jar']" variant="plain" />
      </template>

      <!-- 快速开始：步骤编号卡片 -->
      <template v-else-if="pageType === 'getting-started'">
        <h1 class="page-title">快速开始</h1>
        <div class="steps-container">
          <div v-for="step in gettingStartedSteps" :key="step.number" class="step-card">
            <div class="step-number">{{ step.number }}</div>
            <div class="step-body">
              <h2 class="step-title">{{ step.title }}</h2>
              <p class="step-content">{{ step.content }}</p>
              <p v-if="step.detail" class="step-detail">{{ step.detail }}</p>
            </div>
          </div>
        </div>
        <cmz-markdown :content="helpDocs['getting-started']" variant="plain" />
      </template>

      <!-- 功能总览：特性卡片网格 -->
      <template v-else-if="pageType === 'features'">
        <h1 class="page-title">功能总览</h1>
        <p class="page-subtitle">快速了解 Sea Lantern 现阶段可用能力与适用场景。</p>
        <div class="feature-grid">
          <div v-for="feature in featureItems" :key="feature.title" class="feature-card">
            <h3 class="feature-title">{{ feature.title }}</h3>
            <p class="feature-desc">{{ feature.desc }}</p>
          </div>
        </div>
      </template>

      <!-- 使用教程：MD + 卡片穿插 -->
      <template v-else-if="pageType === 'tutorial'">
        <template v-for="(seg, idx) in tutorialSegments" :key="idx">
          <!-- MD 段落 -->
          <cmz-markdown v-if="seg.type === 'md'" :content="seg.content" variant="plain" />
          <!-- 配置项卡片 -->
          <template v-else-if="seg.type === 'config-cards'">
            <h2 class="section-heading" style="margin-top: var(--sl-space-lg)">常用配置项</h2>
            <div class="config-grid">
              <div v-for="item in configItems" :key="item.key" class="mini-card">
                <code class="mini-card-key">{{ item.key }}</code>
                <p class="mini-card-desc">{{ item.desc }}</p>
                <span class="mini-card-default"
                  >默认：<code>{{ item.default }}</code></span
                >
              </div>
            </div>
          </template>
          <!-- 插件推荐卡片 -->
          <template v-else-if="seg.type === 'plugin-cards'">
            <h2 class="section-heading" style="margin-top: var(--sl-space-lg)">常用插件推荐</h2>
            <p class="mini-card-note">点击插件名称可跳转到下载页面。</p>
            <div class="plugin-grid">
              <div v-for="plugin in pluginRecommendations" :key="plugin.name" class="plugin-card">
                <div class="plugin-card-header">
                  <a :href="plugin.url" class="plugin-name" target="_blank" rel="noreferrer">{{
                    plugin.name
                  }}</a>
                  <span class="plugin-category">{{ plugin.category }}</span>
                </div>
                <p class="plugin-desc">{{ plugin.desc }}</p>
              </div>
            </div>
          </template>
          <!-- 内存分配建议卡片 -->
          <template v-else-if="seg.type === 'memory-cards'">
            <h2 class="section-heading" style="margin-top: var(--sl-space-lg)">内存分配建议</h2>
            <div class="memory-grid">
              <div v-for="item in memorySuggestions" :key="item.players" class="memory-card">
                <span class="memory-players">{{ item.players }}</span>
                <span class="memory-value">{{ item.memory }}</span>
                <span class="memory-desc">{{ item.desc }}</span>
              </div>
            </div>
          </template>
        </template>
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

/* 通用标题 */
.page-title {
  font-size: var(--sl-font-size-3xl);
  font-weight: 700;
  color: var(--sl-text-primary);
  margin: 0 0 var(--sl-space-xs);
}

.page-subtitle {
  font-size: var(--sl-font-size-base);
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-xl);
  line-height: 1.6;
}

.section-heading {
  font-size: var(--sl-font-size-2xl);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: var(--sl-space-lg) 0 var(--sl-space-md);
  padding-bottom: var(--sl-space-sm);
  border-bottom: 1px solid var(--sl-border-light);
}

/* ========== 特性卡片 ========== */
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

/* ========== 下载卡片 ========== */
.platform-section {
  margin-bottom: var(--sl-space-xl);
}

.platform-subtitle {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-tertiary);
  margin: -var(--sl-space-sm) 0 var(--sl-space-md);
}

.download-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--sl-space-sm);
}

.download-card {
  display: flex;
  flex-direction: column;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  overflow: hidden;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.download-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.download-card-body {
  flex: 1;
  padding: var(--sl-space-md) var(--sl-space-md) var(--sl-space-sm);
}

.download-format {
  font-size: var(--sl-font-size-sm);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 2px;
}

.download-desc {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  margin: 0;
}

.download-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px var(--sl-space-md);
  margin: 0 var(--sl-space-md) var(--sl-space-md);
  border-radius: var(--sl-radius-sm);
  background: var(--sl-primary);
  color: #fff;
  font-size: var(--sl-font-size-sm);
  font-weight: 500;
  text-decoration: none;
  transition: opacity 0.2s;
}

.download-btn:hover {
  opacity: 0.9;
}

.platform-notes {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  margin: var(--sl-space-sm) 0 0;
  line-height: 1.5;
}

/* ========== 服务端类型对比卡片 ========== */
.server-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-lg);
}

.server-card {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-lg);
  display: flex;
  flex-direction: column;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.server-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.server-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sl-space-xs);
}

.server-name {
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0;
}

.server-tags {
  display: flex;
  gap: 4px;
}

.server-tag {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: var(--sl-radius-full);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
}

.tag-recommend {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-weight: 500;
}

.server-desc {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
  margin: 0 0 var(--sl-space-sm);
  line-height: 1.5;
}

.server-stars {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  margin-bottom: var(--sl-space-xs);
}

.stars {
  color: var(--sl-warning);
  letter-spacing: 1px;
}

.server-compat {
  display: flex;
  flex-wrap: wrap;
  gap: 2px var(--sl-space-sm);
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-secondary);
  margin-bottom: var(--sl-space-sm);
  padding: var(--sl-space-xs) var(--sl-space-sm);
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-sm);
}

.compat-label {
  color: var(--sl-text-tertiary);
  font-weight: 500;
}

.server-pros,
.server-cons {
  margin: 0 0 var(--sl-space-xs);
  padding-left: var(--sl-space-lg);
  font-size: var(--sl-font-size-xs);
  line-height: 1.6;
}

.server-pros li {
  color: var(--sl-success);
}

.server-cons li {
  color: var(--sl-error);
}

.server-link {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  margin-top: auto;
  padding-top: var(--sl-space-sm);
  font-size: var(--sl-font-size-sm);
  color: var(--sl-primary);
  text-decoration: none;
  font-weight: 500;
  border-top: 1px solid var(--sl-border-light);
}

.server-link:hover {
  opacity: 0.8;
}

/* ========== 步骤卡片 ========== */
.steps-container {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  margin-bottom: var(--sl-space-lg);
}

.step-card {
  display: flex;
  gap: var(--sl-space-md);
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-lg);
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.step-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--sl-primary);
  color: #fff;
  font-size: var(--sl-font-size-base);
  font-weight: 700;
  flex-shrink: 0;
}

.step-body {
  flex: 1;
  min-width: 0;
}

.step-title {
  font-size: var(--sl-font-size-lg);
  font-weight: 600;
  color: var(--sl-text-primary);
  margin: 0 0 4px;
}

.step-content {
  font-size: var(--sl-font-size-sm);
  color: var(--sl-text-secondary);
  margin: 0 0 2px;
  line-height: 1.5;
}

.step-detail {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  margin: 0;
  line-height: 1.5;
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

/* ========== 配置项小卡片 ========== */
.config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-lg);
}

.mini-card {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-md);
  display: flex;
  flex-direction: column;
  gap: 4px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.mini-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.mini-card-key {
  font-size: var(--sl-font-size-sm);
  font-weight: 600;
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  padding: 1px 6px;
  border-radius: var(--sl-radius-sm);
  align-self: flex-start;
}

.mini-card-desc {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-secondary);
  margin: 0;
  line-height: 1.5;
}

.mini-card-default {
  font-size: 11px;
  color: var(--sl-text-tertiary);
}

.mini-card-default code {
  font-size: 11px;
  background: var(--sl-bg-secondary);
  padding: 0 4px;
  border-radius: 2px;
}

.mini-card-note {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
  margin: -var(--sl-space-sm) 0 var(--sl-space-md);
}

/* ========== 插件推荐卡片 ========== */
.plugin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-lg);
}

.plugin-card {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-md);
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.plugin-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.plugin-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sl-space-xs);
}

.plugin-name {
  font-size: var(--sl-font-size-sm);
  font-weight: 600;
  color: var(--sl-primary);
  text-decoration: none;
}

.plugin-name:hover {
  text-decoration: underline;
}

.plugin-category {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: var(--sl-radius-full);
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-tertiary);
  flex-shrink: 0;
}

.plugin-desc {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-secondary);
  margin: 0;
  line-height: 1.5;
}

/* ========== 内存分配建议卡片 ========== */
.memory-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-lg);
}

.memory-card {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-md);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

.memory-card:hover {
  border-color: var(--sl-primary);
  box-shadow: 0 0 0 1px var(--sl-primary);
}

.memory-players {
  font-size: var(--sl-font-size-xs);
  color: var(--sl-text-tertiary);
}

.memory-value {
  font-size: var(--sl-font-size-xl);
  font-weight: 700;
  color: var(--sl-primary);
}

.memory-desc {
  font-size: 11px;
  color: var(--sl-text-tertiary);
  line-height: 1.4;
}
</style>
