<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import {
  Plus,
  Code2,
  PenTool,
  HelpCircle,
  BookText,
  Globe,
  Megaphone,
  Info,
  Copy,
  Link,
  Check,
  ArrowRight,
  AlertCircle,
  ExternalLink,
  RefreshCw,
  XCircle,
  Code,
  Feather,
  Lightbulb,
  BookOpen,
  Rocket,
  SquarePen,
} from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import BrandIcon from "../components/common/BrandIcon.vue";
import {
  contributors as contributorsList,
  type SocialLinks,
  type Contributor,
} from "../data/contributors";
import { checkUpdate, type UpdateInfo } from "../api/update";
import { getAppVersion, BUILD_YEAR } from "../utils/version";
import { i18n } from "../language";

const version = ref("Loading...");
const buildDate = BUILD_YEAR;

const contributors = ref(contributorsList);

const PAGE_SIZE = 9;
const currentPage = ref(1);

const displayedContributors = computed(() => {
  return contributors.value.slice(0, currentPage.value * PAGE_SIZE);
});

const hasMore = computed(() => {
  return displayedContributors.value.length < contributors.value.length;
});

function loadMore() {
  currentPage.value++;
}

const isCheckingUpdate = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const updateError = ref<string | null>(null);

const showAurWindow = ref(false);
const aurInfo = ref<{
  currentVersion: string;
  latestVersion: string;
  helper: string;
  command: string;
} | null>(null);

const isAurUpdate = computed(() => updateInfo.value?.source === "arch-aur");

const copiedQQ = ref<string | null>(null);

onMounted(async () => {
  version.value = await getAppVersion();
});

function isSocialLinks(url: string | SocialLinks | undefined): url is SocialLinks {
  return typeof url === "object" && url !== null;
}

async function openLink(url: string) {
  if (!url) return;
  try {
    await openUrl(url);
  } catch (e) {
    console.error("[AboutView] 打开URL失败:", e);
    console.error("[AboutView] 无法打开链接:", e);
  }
}

async function openSocialLink(platform: string, value: string) {
  if (platform === "qq") {
    await copyQQ(value);
  } else {
    await openLink(value);
  }
}

async function copyQQ(qq: string) {
  try {
    await navigator.clipboard.writeText(qq);
    copiedQQ.value = qq;
    setTimeout(() => {
      copiedQQ.value = null;
    }, 2000);
  } catch (e) {
    console.error("[AboutView] 复制QQ失败:", e);
  }
}

function getSocialTitle(platform: string): string {
  const titles: Record<string, string> = {
    gitee: "Gitee",
    github: "GitHub",
    bilibili: "Bilibili",
    qq: "QQ (点击复制)",
  };
  return titles[platform] || platform;
}

async function handlePrimaryUpdateAction() {
  // 如果是 AUR 更新，显示提示窗口
  if (isAurUpdate.value && updateInfo.value) {
    const helper =
      updateInfo.value.release_notes?.match(/yay|paru|pamac|trizen|pacaur/)?.[0] || "yay";

    aurInfo.value = {
      currentVersion: updateInfo.value.current_version,
      latestVersion: updateInfo.value.latest_version,
      helper: helper,
      command: `${helper} -Rns sealantern && ${helper} -S sealantern`,
    };

    showAurWindow.value = true;
    return;
  }
}

function getCustomLinks(links: SocialLinks): [string, string][] {
  const predefined = ["gitee", "github", "bilibili", "qq"];
  return Object.entries(links).filter(([key, value]) => !predefined.includes(key) && value) as [
    string,
    string,
  ][];
}

async function handleCheckUpdate() {
  isCheckingUpdate.value = true;
  updateError.value = null;
  updateInfo.value = null;

  try {
    const info = await checkUpdate();

    if (info) {
      updateInfo.value = info;
    } else {
      // 没有更新
      updateInfo.value = {
        has_update: false,
        latest_version: version.value,
        current_version: version.value,
      };
    }
  } catch (error) {
    console.error("[AboutView] 检查更新失败:", error);
    updateError.value = error as string;
  } finally {
    isCheckingUpdate.value = false;
  }
}

// 手动下载
async function handleManualDownload() {
  if (updateInfo.value?.download_url) {
    try {
      await openUrl(updateInfo.value.download_url);
    } catch (error) {
      console.error("[AboutView] 打开链接失败:", error);
      alert(`打开链接失败: ${error}`);
    }
  }
}
</script>

<template>
  <div class="about-view animate-fade-in-up">
    <!-- Hero Section -->
    <div class="hero-section">
      <div class="hero-logo">
        <img src="../assets/logo.svg" alt="Sea Lantern" width="72" height="72" />
      </div>
      <h1 class="hero-title">Sea Lantern</h1>
      <p class="hero-subtitle">Minecraft 服务器管理工具</p>
      <div class="hero-badges">
        <span class="version-badge">v{{ version }}</span>
        <span class="tech-badge">Tauri 2 + Vue 3</span>
        <span class="license-badge">GPLv3</span>
      </div>
      <p class="hero-desc">
        一个由社区共同打造的 Minecraft 开服器。<br />
        不仅代码开源，连灵魂都由你们定义。
      </p>
    </div>

    <!-- Manifesto -->
    <SLCard>
      <div class="manifesto">
        <h3 class="manifesto-title">为什么叫 Sea Lantern？</h3>
        <p class="manifesto-text">
          海晶灯（Sea Lantern）是 Minecraft
          中一种发光方块——它由无数碎片组合而成，却能发出柔和而持久的光。
        </p>
        <p class="manifesto-text">
          就像这个项目一样，每一位贡献者都是一片海晶碎片。<br />
          当我们聚在一起，就能照亮整个社区。
        </p>
      </div>
    </SLCard>

    <!-- 此处缺一段代码 -->
    <!-- 点击加入开发 -->

    <!-- Contributor Wall -->
    <div class="contributor-section">
      <div class="section-header">
        <h2 class="section-title">贡献者墙</h2>
        <p class="section-desc">每一个让这个项目变得更好的人</p>
      </div>

      <div class="contributor-grid">
        <div v-for="c in displayedContributors" :key="c.name" class="contributor-card glass-card">
          <img :src="c.avatar" :alt="c.name" class="contributor-avatar" />

          <div class="contributor-right">
            <div class="contributor-info" :title="c.name + ' - ' + c.role">
              <span class="contributor-name">{{ c.name }}</span>
              <span class="contributor-role">{{ c.role }}</span>
            </div>

            <div v-if="c.url" class="contributor-social">
              <template v-if="!isSocialLinks(c.url)">
                <a
                  :href="c.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="social-icon"
                  title="个人主页"
                >
                  <Link :size="16" />
                </a>
              </template>

              <template v-else>
                <a
                  v-if="c.url.gitee"
                  :href="c.url.gitee"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="social-icon"
                  title="Gitee"
                >
                  <BrandIcon name="gitee" :size="16" />
                </a>

                <a
                  v-if="c.url.github"
                  :href="c.url.github"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="social-icon"
                  title="GitHub"
                >
                  <BrandIcon name="github" :size="16" />
                </a>

                <a
                  v-if="c.url.bilibili"
                  :href="c.url.bilibili"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="social-icon"
                  title="Bilibili"
                >
                  <BrandIcon name="bilibili" :size="16" />
                </a>

                <button
                  v-if="c.url.qq"
                  @click="openSocialLink('qq', c.url.qq)"
                  class="social-icon"
                  :class="{ copied: copiedQQ === c.url.qq }"
                  :title="copiedQQ === c.url.qq ? '已复制!' : 'QQ (点击复制)'"
                >
                  <Check v-if="copiedQQ === c.url.qq" :size="16" />
                  <BrandIcon v-else name="qq" :size="16" />
                </button>

                <a
                  v-for="[key, value] in getCustomLinks(c.url)"
                  :key="key"
                  :href="value"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="social-icon"
                  :title="key"
                >
                  <ExternalLink :size="16" />
                </a>
              </template>
            </div>
          </div>
        </div>

        <!-- Join Card -->
        <div class="contributor-card glass-card join-card">
          <div class="join-icon">
            <Plus :size="40" :stroke-width="1.5" />
          </div>
          <div class="contributor-right">
            <div class="contributor-info">
              <span class="contributor-name join-text">你的名字</span>
              <span class="contributor-role">参与贡献，加入我们</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="hasMore" class="load-more-section">
        <SLButton variant="ghost" @click="loadMore">
          加载更多 ({{ contributors.length - displayedContributors.length }} 位)
        </SLButton>
      </div>
    </div>

    <!-- Project Info -->
    <div class="info-grid">
      <SLCard title="项目信息">
        <div class="info-list">
          <div class="info-item">
            <span class="info-label">版本</span>
            <span class="info-value">{{ version }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">构建年份</span>
            <span class="info-value">{{ buildDate }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">前端</span>
            <span class="info-value">Vue 3 + TypeScript + Vite</span>
          </div>
          <div class="info-item">
            <span class="info-label">后端</span>
            <span class="info-value">Rust + Tauri 2</span>
          </div>
          <div class="info-item">
            <span class="info-label">许可证</span>
            <span class="info-value">GNU GPLv3</span>
          </div>
        </div>

        <div class="update-section">
          <SLButton
            variant="secondary"
            size="sm"
            @click="handleCheckUpdate"
            :disabled="isCheckingUpdate"
            style="width: 100%"
          >
            {{ isCheckingUpdate ? "检查中..." : "检查更新" }}
          </SLButton>

          <div v-if="updateInfo" class="update-info">
            <div v-if="updateInfo.has_update" class="update-available">
              <div class="update-message">
                <div class="update-icon">
                  <RefreshCw :size="16" :stroke-width="2" />
                </div>
                <div>
                  <div class="update-title">发现新版本 v{{ updateInfo.latest_version }}</div>
                  <div class="update-desc">当前版本: v{{ updateInfo.current_version }}</div>
                </div>
              </div>
              <div v-if="updateInfo.release_notes" class="release-notes">
                <div class="notes-title">更新内容:</div>
                <div class="notes-content">{{ updateInfo.release_notes }}</div>
              </div>
              <div class="update-buttons">
                <SLButton
                  variant="primary"
                  size="sm"
                  @click="handleManualDownload"
                  style="width: 100%"
                >
                  前往下载页面
                </SLButton>
              </div>
            </div>
            <div v-else class="update-latest">
              <div class="update-icon">
                <Check :size="16" :stroke-width="2.5" />
              </div>
              <span>已是最新版本</span>
            </div>
          </div>

          <div v-if="updateError" class="update-error">
            <div class="error-icon">
              <XCircle :size="14" :stroke-width="2.5" />
            </div>
            <span>{{ updateError }}</span>
          </div>
        </div>
      </SLCard>

      <SLCard title="参与方式">
        <div class="contribute-ways">
          <div class="way-item">
            <div class="way-icon">
              <Code :size="20" :stroke-width="2" />
            </div>
            <div class="way-info">
              <span class="way-title">写代码</span>
              <span class="way-desc">提交 PR，修 Bug 或加新功能</span>
            </div>
          </div>
          <div class="way-item">
            <div class="way-icon">
              <Feather :size="20" :stroke-width="2" />
            </div>
            <div class="way-info">
              <span class="way-title">做设计</span>
              <span class="way-desc">设计 UI、图标、主题皮肤</span>
            </div>
          </div>
          <div class="way-item">
            <div class="way-icon">
              <Lightbulb :size="20" :stroke-width="2" />
            </div>
            <div class="way-info">
              <span class="way-title">提建议</span>
              <span class="way-desc">在 Issues 里提出你的想法</span>
            </div>
          </div>
          <div class="way-item">
            <div class="way-icon">
              <BookOpen :size="20" :stroke-width="2" />
            </div>
            <div class="way-info">
              <span class="way-title">写文档</span>
              <span class="way-desc">完善教程和使用说明</span>
            </div>
          </div>
          <div class="way-item">
            <div class="way-icon">
              <Globe :size="20" :stroke-width="2" />
            </div>
            <div class="way-info">
              <span class="way-title">翻译</span>
              <span class="way-desc">帮助翻译成其他语言</span>
            </div>
          </div>
          <div class="way-item">
            <div class="way-icon">
              <Rocket :size="20" :stroke-width="2" />
            </div>
            <div class="way-info">
              <span class="way-title">推广</span>
              <span class="way-desc">分享给更多 MC 服主</span>
            </div>
          </div>
        </div>
      </SLCard>
    </div>

    <!-- Links -->
    <div class="links-section">
      <SLButton variant="primary" size="lg" @click="openLink('https://gitee.com/fps_z/SeaLantern')">
        Gitee 仓库
      </SLButton>
      <SLButton variant="primary" size="lg" @click="openLink('https://github.com/FPSZ/SeaLantern')">
        Github 仓库
      </SLButton>
      <SLButton
        variant="secondary"
        size="lg"
        @click="openLink('https://space.bilibili.com/3706927622130406?spm_id_from=333.1387.0.0')"
      >
        B站主页
      </SLButton>
    </div>

    <!-- Footer -->
    <div class="about-footer">
      <p class="footer-text">Sea Lantern 是一个开源项目，遵循 GPLv3 协议。</p>
      <p class="footer-text">Minecraft 是 Mojang Studios 的注册商标。本项目与 Mojang 无关。</p>
      <p class="footer-quote">"我们搭建了骨架，而灵魂，交给你们。"</p>
    </div>
  </div>
</template>

<style scoped>
.about-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xl);
  max-width: 900px;
  margin: 0 auto;
  padding-bottom: var(--sl-space-2xl);
}

/* Hero */
.hero-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: var(--sl-space-2xl) 0;
}

.hero-logo {
  margin-bottom: var(--sl-space-md);
  animation: sl-fade-in-up 0.6s ease forwards;
}

.hero-title {
  font-size: 2.5rem;
  font-weight: 800;
  color: var(--sl-text-primary);
  letter-spacing: -0.03em;
  margin-bottom: var(--sl-space-xs);
  animation: sl-fade-in-up 0.6s ease 0.1s both;
}

.hero-subtitle {
  font-size: 1.125rem;
  color: var(--sl-text-secondary);
  margin-bottom: var(--sl-space-md);
  animation: sl-fade-in-up 0.6s ease 0.2s both;
}

.hero-badges {
  display: flex;
  gap: var(--sl-space-sm);
  margin-bottom: var(--sl-space-lg);
  animation: sl-fade-in-up 0.6s ease 0.3s both;
}

.version-badge,
.tech-badge,
.license-badge {
  padding: 4px 14px;
  border-radius: var(--sl-radius-full);
  font-size: 0.8125rem;
  font-weight: 500;
}

.version-badge {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.tech-badge {
  background: rgba(34, 197, 94, 0.1);
  color: var(--sl-success);
}

.license-badge {
  background: rgba(168, 85, 247, 0.1);
  color: #a855f7;
}

[data-theme="dark"] .license-badge {
  color: #c084fc;
}

.hero-desc {
  font-size: 1rem;
  color: var(--sl-text-secondary);
  line-height: 1.8;
  animation: sl-fade-in-up 0.6s ease 0.4s both;
}

/* Manifesto */
.manifesto {
  text-align: center;
  padding: var(--sl-space-lg);
}

.manifesto-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-md);
}

.manifesto-text {
  font-size: 0.9375rem;
  color: var(--sl-text-secondary);
  line-height: 1.8;
  margin-bottom: var(--sl-space-sm);
}

/* Contributor Wall */
.contributor-section {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.section-header {
  text-align: center;
}

.section-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-xs);
}

.section-desc {
  font-size: 0.9375rem;
  color: var(--sl-text-tertiary);
}

.contributor-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--sl-space-md);
}

.contributor-card {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md);
  transition: all var(--sl-transition-normal);
}

.contributor-card.clickable {
  cursor: pointer;
}

.contributor-card.clickable:hover {
  transform: translateY(-4px);
  box-shadow: var(--sl-shadow-lg);
}

.contributor-avatar {
  width: 48px;
  height: 48px;
  border-radius: var(--sl-radius-md);
  flex-shrink: 0;
  image-rendering: pixelated;
}

.contributor-info {
  display: flex;
  flex-direction: row;
  align-items: baseline;
  gap: var(--sl-space-xs);
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.contributor-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  flex-shrink: 0;
}

.contributor-role {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.join-card {
  border: 2px dashed var(--sl-border);
  background: transparent;
  cursor: default;
}

.join-card:hover {
  border-color: var(--sl-primary-light);
  background: var(--sl-primary-bg);
  transform: none;
  box-shadow: none;
}

.join-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.join-text {
  color: var(--sl-primary);
}

/* Info Grid */
.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-md);
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--sl-border-light);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}

.info-value {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--sl-text-primary);
  font-family: var(--sl-font-mono);
}

/* Contribute Ways */
.contribute-ways {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-sm);
}

.way-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
  transition: background var(--sl-transition-fast);
}

.way-item:hover {
  background: var(--sl-bg-secondary);
}

.way-icon {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
  border-radius: var(--sl-radius-md);
  transition: all var(--sl-transition-fast);
}

.way-item:hover .way-icon {
  background: var(--sl-primary);
  color: white;
  transform: scale(1.05);
}

.way-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.way-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.way-desc {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

/* Links */
.links-section {
  display: flex;
  justify-content: center;
  gap: var(--sl-space-md);
}

.links-section :deep(.sl-button) {
  min-width: 140px;
}

/* Footer */
.about-footer {
  text-align: center;
  padding: var(--sl-space-xl) 0;
  border-top: 1px solid var(--sl-border-light);
}

.footer-text {
  font-size: 0.8125rem;
  color: var(--sl-text-tertiary);
  margin-bottom: var(--sl-space-xs);
}

.footer-quote {
  font-size: 1rem;
  font-weight: 500;
  color: var(--sl-primary);
  font-style: italic;
  margin-top: var(--sl-space-md);
}

/* Update Section */
.update-section {
  margin-top: var(--sl-space-md);
  padding-top: var(--sl-space-md);
  border-top: 1px solid var(--sl-border-light);
}

.update-info {
  margin-top: var(--sl-space-sm);
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
  font-size: 0.875rem;
}

.update-available {
  background: var(--sl-primary-bg);
  border: 1px solid var(--sl-primary-light);
  padding: var(--sl-space-sm);
  border-radius: var(--sl-radius-md);
}

.update-message {
  display: flex;
  align-items: flex-start;
  gap: var(--sl-space-sm);
}

.update-icon {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-primary);
  color: white;
  border-radius: var(--sl-radius-sm);
}

.update-latest .update-icon {
  background: var(--sl-success);
}

.update-title {
  font-weight: 600;
  color: var(--sl-primary);
  margin-bottom: 2px;
}

.update-desc {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
}

.release-notes {
  margin-top: var(--sl-space-sm);
  padding-top: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
}

.notes-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--sl-text-secondary);
  margin-bottom: 4px;
}

.notes-content {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  max-height: 120px;
  overflow-y: auto;
  white-space: pre-wrap;
}

.update-buttons {
  display: flex;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
}

.update-latest {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-sm);
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-success);
  font-weight: 500;
}

.update-error {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: var(--sl-space-sm);
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--sl-radius-md);
  color: var(--sl-danger);
  font-size: 0.8125rem;
}

.error-icon {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--sl-danger);
  color: white;
  border-radius: 50%;
}

@media (max-width: 768px) {
  .info-grid {
    grid-template-columns: 1fr;
  }
  .contribute-ways {
    grid-template-columns: 1fr;
  }
  .contributor-grid {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  }
}

.contributor-link {
  display: inline-block;
  cursor: pointer;
  transition: all var(--sl-transition-normal);
  border-radius: var(--sl-radius-md);
  line-height: 0;
}

.contributor-link:hover {
  transform: translateY(-4px);
}

.contributor-link:hover .contributor-avatar {
  box-shadow: var(--sl-shadow-lg);
}

.contributor-right {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xs);
  flex: 1;
  min-width: 0;
}

.contributor-social {
  display: flex;
  gap: var(--sl-space-xs);
  flex-wrap: wrap;
}

.social-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 4px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.social-icon:hover {
  color: var(--sl-primary);
  background: var(--sl-primary-bg);
  transform: scale(1.1);
}

.social-icon svg {
  width: 16px;
  height: 16px;
}

.social-icon.copied {
  color: var(--sl-success);
  background: rgba(34, 197, 94, 0.1);
}

.load-more-section {
  display: flex;
  justify-content: center;
  margin-top: var(--sl-space-lg);
}
</style>
