﻿﻿﻿<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { Plus, Code2, PenTool, HelpCircle, BookText, Globe, Megaphone } from "lucide-vue-next";
import { openUrl } from "@tauri-apps/plugin-opener";
import SLCard from "../components/common/SLCard.vue";
import SLButton from "../components/common/SLButton.vue";
import SLNotification from "../components/common/SLNotification.vue";
import { contributors as contributorsList } from "../data/contributors";
import { useUpdateStore } from "../stores/updateStore";
import { getAppVersion, BUILD_YEAR } from "../utils/version";
import { i18n } from "../locales";
import { onDownloadProgress } from "../api/update";
import type { UnlistenFn } from "@tauri-apps/api/event";

const version = ref(i18n.t("common.loading"));
const buildDate = BUILD_YEAR;

const contributors = ref(contributorsList);

const updateStore = useUpdateStore();

const showNotification = ref(false);
const notificationMessage = ref("");
const notificationType = ref<"success" | "error" | "warning" | "info">("info");

let unlistenProgress: UnlistenFn | null = null;
let resetTimer: ReturnType<typeof setTimeout> | null = null;

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

  unlistenProgress = await onDownloadProgress((progress) => {
    updateStore.setDownloading(progress.percent);
  });
});

onUnmounted(() => {
  if (resetTimer) {
    clearTimeout(resetTimer);
    resetTimer = null;
  }
  if (unlistenProgress) {
    unlistenProgress();
  }
});

const buttonState = computed(() => {
  switch (updateStore.status) {
    case "checking":
      return {
        text: i18n.t("about.update_checking"),
        variant: "secondary" as const,
        disabled: true,
      };
    case "latest":
      return { text: i18n.t("about.update_latest"), variant: "success" as const, disabled: true };
    case "available":
      return {
        text: i18n.t("about.update_available"),
        variant: "primary" as const,
        disabled: false,
      };
    case "downloading":
      return {
        text: `${i18n.t("about.update_downloading")} ${progressPercent.value}%`,
        variant: "secondary" as const,
        disabled: false,
      };
    case "installing":
      return {
        text: i18n.t("about.update_installing"),
        variant: "secondary" as const,
        disabled: false,
      };
    case "downloaded":
      return { text: i18n.t("about.update_ready"), variant: "success" as const, disabled: false };
    case "error":
      return { text: i18n.t("about.update_error"), variant: "danger" as const, disabled: false };
    default:
      return { text: i18n.t("about.check_update"), variant: "secondary" as const, disabled: false };
  }
});

const progressPercent = computed(() => Math.round(updateStore.downloadProgress));

async function openLink(url: string) {
  if (!url) return;
  try {
    await openUrl(url);
  } catch (e) {
    alert(`${i18n.t("about.open_link_failed")}: ${String(e)}`);
  }
}

async function handleCheckUpdate() {
  if (resetTimer) {
    clearTimeout(resetTimer);
    resetTimer = null;
  }

  try {
    await updateStore.checkForUpdate();
  } catch (error) {
    showNotify(`${i18n.t("about.update_check_failed")}: ${String(error)}`, "error");
  }
}

async function handlePrimaryUpdateAction() {
  if (updateStore.hasStartedUpdateFlow && updateStore.isUpdateAvailable) {
    updateStore.showUpdateModal();
    return;
  }
  await handleCheckUpdate();
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

      <!-- Contributor Wall -->
      <div class="contributor-section">
        <div class="section-header">
          <h2 class="section-title">{{ i18n.t("about.contributor_wall") }}</h2>
          <p class="section-desc">{{ i18n.t("about.contributor_desc") }}</p>
        </div>

        <div class="contributor-grid">
          <div v-for="c in contributors" :key="c.name" class="contributor-card glass-card">
            <a
              v-if="c.url"
              :href="c.url"
              target="_blank"
              rel="noopener noreferrer"
              class="contributor-link"
            >
              <img :src="c.avatar" :alt="c.name" class="contributor-avatar" />
            </a>
            <img v-else :src="c.avatar" :alt="c.name" class="contributor-avatar" />

            <div class="contributor-info">
              <span class="contributor-name">{{ c.name }}</span>
              <span class="contributor-role">{{ c.role }}</span>
            </div>
          </div>

          <!-- Join Card -->
          <div class="contributor-card glass-card join-card">
            <div class="join-icon">
                <Plus :size="40" stroke-width="1.5" :color="'var(--sl-primary)'" />
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

          <!-- Update action button -->
          <div class="update-section">
            <SLButton
              class="update-action-btn"
              :variant="buttonState.variant"
              size="sm"
              @click="handlePrimaryUpdateAction"
              :disabled="buttonState.disabled"
              style="width: 100%"
            >
              <span class="update-btn-content">
                <span
                  v-if="updateStore.status === 'downloading'"
                  class="update-btn-progress"
                  :style="{ width: `${progressPercent}%` }"
                />
                <span class="update-btn-label">{{ buttonState.text }}</span>
              </span>
            </SLButton>
          </div>
        </SLCard>

        <SLCard :title="i18n.t('about.contribute_ways')">
          <div class="contribute-ways">
            <div class="way-item">
              <div class="way-icon">
                <Code2 :size="20" />
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_code") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_code_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <PenTool :size="20" />
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_design") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_design_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <HelpCircle :size="20" />
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_idea") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_idea_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <BookText :size="20" />
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_doc") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_doc_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <Globe :size="20" />
              </div>
              <div class="way-info">
                <span class="way-title">{{ i18n.t("about.way_translate") }}</span>
                <span class="way-desc">{{ i18n.t("about.way_translate_desc") }}</span>
              </div>
            </div>
            <div class="way-item">
              <div class="way-icon">
                <Megaphone :size="20" />
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
          @click="openLink('https://github.com/FPSZ/SeaLantern')"
        >
          {{ i18n.t("about.github_repo") }}
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

    <!-- 閫氱煡缁勪欢 -->
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
  width: 100%;
  height: 32px;
}

.update-btn-content {
  position: static;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.update-btn-progress {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  height: 100%;
  background: rgba(255, 255, 255, 0.2);
  transition: width 0.2s ease;
  border-radius: inherit;
  z-index: 0;
}

.update-btn-label {
  position: relative;
  z-index: 1;
  white-space: nowrap;
}

:deep(.update-action-btn) {
  position: relative;
  overflow: hidden;
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
