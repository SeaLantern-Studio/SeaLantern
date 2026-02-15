<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLModal from "../components/common/SLModal.vue";
import SLNotification from "../components/common/SLNotification.vue";
import { contributors as contributorsList } from "../data/contributors";
import { checkUpdate, type UpdateInfo } from "../api/update";
import { getAppVersion, BUILD_YEAR } from "../utils/version";
import { i18n } from "../locales";

const version = ref("加载中...");
const buildDate = BUILD_YEAR;

const contributors = ref(contributorsList);

const updateInfo = ref<UpdateInfo | null>(null);
const updateError = ref<string | null>(null);
const updateStatus = ref<"idle" | "checking" | "latest" | "available" | "error">("idle");
let resetTimer: ReturnType<typeof setTimeout> | null = null;

const showUpdateModal = ref(false);
const modalUpdateInfo = ref<UpdateInfo | null>(null);

const showNotification = ref(false);
const notificationMessage = ref("");
const notificationType = ref<"success" | "error" | "warning" | "info">("info");

function showNotify(msg: string, type: "success" | "error" | "warning" | "info" = "info") {
  notificationMessage.value = msg;
  notificationType.value = type;
  showNotification.value = true;
}

function closeNotification() {
  showNotification.value = false;
}

onMounted(async () => {
  version.value = await getAppVersion();
});

onUnmounted(() => {
  if (resetTimer) {
    clearTimeout(resetTimer);
    resetTimer = null;
  }
});

function closeUpdateModal() {
  showUpdateModal.value = false;
  modalUpdateInfo.value = null;
}

function getButtonVariant(): "primary" | "secondary" | "danger" | "success" {
  switch (updateStatus.value) {
    case "checking":
      return "secondary";
    case "latest":
      return "success";
    case "available":
      return "primary";
    case "error":
      return "danger";
    default:
      return "secondary";
  }
}

async function openLink(url: string) {
  if (!url) return;
  try {
    await openUrl(url);
  } catch (e) {
    alert(`无法打开链接: ${e}`);
  }
}

async function handleCheckUpdate() {
  if (resetTimer) {
    clearTimeout(resetTimer);
    resetTimer = null;
  }

  updateStatus.value = "checking";
  updateError.value = null;
  updateInfo.value = null;

  try {
    const info = await checkUpdate();

    if (info && info.has_update) {
      updateInfo.value = info;
      updateStatus.value = "available";
      modalUpdateInfo.value = info;
      showUpdateModal.value = true;
      if (info.source === "github") {
        showNotify("Gitee 不可用，已切换到 GitHub", "warning");
      }
    } else {
      updateInfo.value = {
        has_update: false,
        latest_version: version.value,
        current_version: version.value,
      };
      updateStatus.value = "latest";

      if (info?.source === "github") {
        showNotify("Gitee 不可用，已切换到 GitHub", "warning");
      }

      resetTimer = setTimeout(() => {
        updateStatus.value = "idle";
        updateInfo.value = null;
        resetTimer = null;
      }, 3000);
    }
  } catch (error) {
    showNotify("检查更新失败: " + (error as string), "error");
    updateStatus.value = "error";

    resetTimer = setTimeout(() => {
      updateStatus.value = "idle";
      resetTimer = null;
    }, 3000);
  }
}

async function handleManualDownload() {
  if (updateInfo.value?.download_url) {
    try {
      await openUrl(updateInfo.value.download_url);
    } catch (error) {
      alert(`打开链接失败: ${error}`);
    }
  }
}
</script>

<template>
  <div>
    <div class="about-view">
      <!-- Hero Section -->
      <div class="hero-section">
        <div class="hero-logo">
          <img src="../assets/logo.svg" :alt="i18n.t('common.app_name')" width="72" height="72" />
        </div>
        <h1 class="hero-title">{{ i18n.t("common.app_name") }}</h1>
        <p class="hero-subtitle">{{ i18n.t("about.subtitle") }}</p>
        <div class="hero-badges">
          <span class="version-badge">v{{ version }}</span>
          <span class="tech-badge">{{ i18n.t("about.tech_badge") }}</span>
          <span class="license-badge">{{ i18n.t("about.license_badge") }}</span>
        </div>
        <p class="hero-desc">
          {{ i18n.t("about.hero_desc") }}
        </p>
      </div>

      <!-- Manifesto -->
      <SLCard>
        <div class="manifesto">
          <h3 class="manifesto-title">{{ i18n.t("about.manifesto_title") }}</h3>
          <p class="manifesto-text">
            {{ i18n.t("about.manifesto_text1") }}
          </p>
          <p class="manifesto-text">
            {{ i18n.t("about.manifesto_text2") }}
          </p>
        </div>
      </SLCard>

      <!-- 此处缺一段代码 -->
      <!-- 点击加入开发 -->

      <!-- Contributor Wall -->
      <div class="contributor-section">
        <div class="section-header">
          <h2 class="section-title">{{ i18n.t("about.contributor_wall") }}</h2>
          <p class="section-desc">{{ i18n.t("about.contributor_desc") }}</p>
        </div>

        <div class="contributor-grid">
          <div
            v-for="c in contributors"
            :key="c.name"
            class="contributor-card glass-card"
          >
            <!-- 如果存在 url，则用 a 标签包裹头像；否则只显示图片 -->
            <a 
              v-if="c.url" 
              :href="c.url" 
              target="_blank" 
              rel="noopener noreferrer"
              class="contributor-link"
            >
              <img :src="c.avatar" :alt="c.name" class="contributor-avatar" />
            </a>
            <img 
              v-else 
              :src="c.avatar" 
              :alt="c.name" 
              class="contributor-avatar" 
            />
            
            <div class="contributor-info">
              <span class="contributor-name">{{ c.name }}</span>
              <span class="contributor-role">{{ c.role }}</span>
            </div>
          </div>

          <!-- Join Card -->
          <div class="contributor-card glass-card join-card">
            <div class="join-icon">
              <svg
                width="40"
                height="40"
                viewBox="0 0 24 24"
                fill="none"
                stroke="var(--sl-primary)"
                stroke-width="1.5"
                stroke-linecap="round"
              >
                <path d="M12 4v16m8-8H4" />
              </svg>
            </div>
            <div class="contributor-info">
              <span class="contributor-name join-text">{{ i18n.t("about.join_text") }}</span>
              <span class="contributor-role">{{ i18n.t("about.join_desc") }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Project Info -->
      <div class="info-grid">
        <SLCard :title="i18n.t('about.project_info')">
          <div class="info-list">
            <div class="info-item">
              <span class="info-label">{{ i18n.t("about.version") }}</span>
              <span class="info-value">{{ version }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ i18n.t("about.build_year") }}</span>
              <span class="info-value">{{ buildDate }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ i18n.t("about.frontend") }}</span>
              <span class="info-value">Vue 3 + TypeScript + Vite</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ i18n.t("about.backend") }}</span>
              <span class="info-value">Rust + Tauri 2</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ i18n.t("about.license") }}</span>
              <span class="info-value">GNU GPLv3</span>
            </div>
          </div>

          <!-- 检查更新按钮 -->
          <div class="update-section">
            <SLButton
              :variant="getButtonVariant()"
              size="sm"
              @click="handleCheckUpdate"
              :disabled="updateStatus === 'checking'"
              style="width: 100%"
            >
              <span class="btn-content">
                <svg
                  v-if="updateStatus === 'checking'"
                  class="animate-spin"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  style="margin-right: 6px"
                >
                  <path
                    d="M12 2v4m0 12v4m10-10h-4M6 12H2m15.07-5.07l-2.83 2.83M9.76 14.24l-2.83 2.83m11.14 0l-2.83-2.83M9.76 9.76L6.93 6.93"
                  />
                </svg>
                <svg
                  v-else-if="updateStatus === 'latest'"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  style="margin-right: 6px"
                >
                  <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
                <svg
                  v-else-if="updateStatus === 'available'"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  style="margin-right: 6px"
                >
                  <polyline points="17 1 21 5 17 9"></polyline>
                  <path d="M3 11V9a4 4 0 0 1 4-4h14"></path>
                  <polyline points="7 23 3 19 7 15"></polyline>
                  <path d="M21 13v2a4 4 0 0 1-4 4H3"></path>
                </svg>
                <svg
                  v-else-if="updateStatus === 'error'"
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  style="margin-right: 6px"
                >
                  <circle cx="12" cy="12" r="10"></circle>
                  <line x1="12" y1="8" x2="12" y2="12"></line>
                  <line x1="12" y1="16" x2="12.01" y2="16"></line>
                </svg>
                <span v-if="updateStatus === 'checking'">{{
                  i18n.t("about.update_checking")
                }}</span>
                <span v-else-if="updateStatus === 'latest'">{{
                  i18n.t("about.update_latest")
                }}</span>
                <span v-else-if="updateStatus === 'available'">{{
                  i18n.t("about.update_available")
                }}</span>
                <span v-else-if="updateStatus === 'error'">{{ i18n.t("about.update_error") }}</span>
                <span v-else>{{ i18n.t("about.check_update") }}</span>
              </span>
            </SLButton>
          </div>
        </SLCard>

        <SLCard :title="i18n.t('about.contribute_ways')">
          <div class="contribute-ways">
            <div class="way-item">
              <div class="way-icon">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <polyline points="16 18 22 12 16 6"></polyline>
                  <polyline points="8 6 2 12 8 18"></polyline>
                </svg>
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_code") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_code_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M12 19l7 2-7-18-7 18 7-2zm0 0v-8"></path>
                </svg>
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_design") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_design_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <circle cx="12" cy="12" r="10"></circle>
                  <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                  <line x1="12" y1="17" x2="12.01" y2="17"></line>
                </svg>
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_idea") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_idea_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
                  <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
                </svg>
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_doc") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_doc_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <circle cx="12" cy="12" r="10"></circle>
                  <line x1="2" y1="12" x2="22" y2="12"></line>
                  <path
                    d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
                  ></path>
                </svg>
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_translate") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_translate_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"></path>
                  <polyline points="16 6 12 2 8 6"></polyline>
                  <line x1="12" y1="2" x2="12" y2="15"></line>
                </svg>
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_promote") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_promote_desc") }}</span>
              </div>
            </div>
          </div>
        </SLCard>
      </div>

      <!-- Links -->
      <div class="links-section">
        <SLButton
          variant="primary"
          size="lg"
          @click="openLink('https://gitee.com/fps_z/SeaLantern')"
        >
          {{ i18n.t("about.gitee_repo") }}
        </SLButton>
        <SLButton
          variant="secondary"
          size="lg"
          @click="openLink('https://space.bilibili.com/3706927622130406?spm_id_from=333.1387.0.0')"
        >
          {{ i18n.t("about.bilibili") }}
        </SLButton>
      </div>

      <!-- Footer -->
      <div class="about-footer">
        <p class="footer-text">
          {{ i18n.t("about.footer_text1") }}
        </p>
        <p class="footer-text">
          {{ i18n.t("about.footer_text2") }}
        </p>
        <p class="footer-quote">
          {{ i18n.t("about.footer_quote") }}
        </p>
      </div>
    </div>

    <!-- 更新日志弹窗 -->
    <SLModal
      :visible="showUpdateModal"
      :title="
        modalUpdateInfo
          ? `${i18n.t('about.update_title')} v${modalUpdateInfo.latest_version}`
          : i18n.t('about.update_title')
      "
      @close="closeUpdateModal"
    >
      <div v-if="modalUpdateInfo" class="modal-update-content">
        <div class="modal-update-header">
          <div class="modal-current-version">
            {{ i18n.t("about.update_current") }}: v{{ modalUpdateInfo.current_version }}
          </div>
        </div>
        <div v-if="modalUpdateInfo.release_notes" class="modal-release-notes">
          <div class="modal-notes-title">{{ i18n.t("about.update_release_notes") }}:</div>
          <div class="modal-notes-content">{{ modalUpdateInfo.release_notes }}</div>
        </div>
        <div class="modal-update-actions">
          <SLButton variant="primary" size="md" @click="handleManualDownload" style="width: 100%">
            {{ i18n.t("about.update_now") }}
          </SLButton>
        </div>
      </div>
    </SLModal>

    <!-- 通知组件 -->
    <SLNotification
      :visible="showNotification"
      :message="notificationMessage"
      :type="notificationType"
      :duration="3000"
      @close="closeNotification"
    />
  </div>
</template>

<style scoped>
.about-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-xl);
  max-width: 900px;
  margin: 0 auto;
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
  background: var(--sl-bg-tertiary);
  image-rendering: pixelated;
}

.contributor-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.contributor-name {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.contributor-role {
  font-size: 0.75rem;
  color: var(--sl-text-tertiary);
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
  padding-top: var(--sl-space-xl);
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

.update-section .sl-button {
  flex-shrink: 0;
  transition: all 0.2s ease;
  min-width: 140px;
  height: 32px;
}

.btn-content {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 120px;
  white-space: nowrap;
}

.btn-fade-enter-active,
.btn-fade-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.btn-fade-enter-from {
  opacity: 0;
  transform: translateX(-6px);
}

.btn-fade-leave-to {
  opacity: 0;
  transform: translateX(6px);
}

/* 弹窗内容样式 */
.modal-update-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.modal-update-header {
  padding-bottom: var(--sl-space-sm);
  border-bottom: 1px solid var(--sl-border-light);
}

.modal-current-version {
  font-size: 0.875rem;
  color: var(--sl-text-tertiary);
}

.modal-release-notes {
  max-height: 300px;
  overflow-y: auto;
}

.modal-notes-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--sl-text-primary);
  margin-bottom: var(--sl-space-xs);
}

.modal-notes-content {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  line-height: 1.6;
  white-space: pre-wrap;
}

.modal-update-actions {
  padding-top: var(--sl-space-sm);
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

</style>
