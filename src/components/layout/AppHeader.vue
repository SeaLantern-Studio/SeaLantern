<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18nStore } from "../../stores/i18nStore";
import { i18n } from "../../locales";

const route = useRoute();
const appWindow = getCurrentWindow();
const i18nStore = useI18nStore();
const showLanguageMenu = ref(false);

const pageTitle = computed(() => {
  return (route.meta?.title as string) || i18n.t("common.app_name");
});

const currentLanguageText = computed(() => {
  const locale = i18n.getLocale();
  if (locale === "zh-CN") return i18n.t("header.chinese");
  if (locale === "zh-TW") return i18n.t("header.chinese_tw");
  return i18n.t("header.english");
});

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

function toggleLanguageMenu() {
  showLanguageMenu.value = !showLanguageMenu.value;
}

function setLanguage(locale: string) {
  i18nStore.setLocale(locale);
  showLanguageMenu.value = false;
}

function handleClickOutside() {
  showLanguageMenu.value = false;
}
</script>

<template>
  <header class="app-header glass-subtle">
    <div class="header-left">
      <h2 class="page-title">{{ pageTitle }}</h2>
    </div>

    <div class="header-center" data-tauri-drag-region></div>

    <div class="header-right">
      <div class="language-selector" @click="toggleLanguageMenu">
        <span class="language-text">{{ currentLanguageText }}</span>
        <div class="language-menu" v-if="showLanguageMenu">
          <div class="language-item" @click.stop="setLanguage('zh-CN')">
            {{ i18n.t("header.chinese") }}
          </div>
          <div class="language-item" @click.stop="setLanguage('zh-TW')">
            {{ i18n.t("header.chinese_tw") }}
          </div>
          <div class="language-item" @click.stop="setLanguage('en-US')">
            {{ i18n.t("header.english") }}
          </div>
        </div>
      </div>

      <div class="header-status">
        <span class="status-dot online"></span>
        <span class="status-text">{{ i18n.t("common.app_name") }}</span>
      </div>

      <div class="window-controls">
        <button class="win-btn" @click="minimizeWindow" title="最小化">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="1" y="5.5" width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="win-btn" @click="toggleMaximize" title="最大化">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="1.5"
              y="1.5"
              width="9"
              height="9"
              rx="1"
              fill="none"
              stroke="currentColor"
              stroke-width="1"
            />
          </svg>
        </button>
        <button class="win-btn win-btn-close" @click="closeWindow" title="关闭">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M2 2l8 8M10 2l-8 8"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </div>

    <div v-if="showLanguageMenu" class="click-outside" @click="handleClickOutside"></div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--sl-header-height);
  padding: 0 var(--sl-space-md) 0 var(--sl-space-lg);
  border-bottom: 1px solid var(--sl-border-light);
  flex-shrink: 0;
  user-select: none;
  position: relative;
  z-index: 100;
  /* 不要在这里加 drag */
}

.header-left,
.header-right {
  -webkit-app-region: no-drag;
}

.page-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.header-center {
  flex: 1;
  height: 100%;
  min-height: var(--sl-header-height);
  -webkit-app-region: drag;
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
}

.header-status {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 4px 12px;
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-full);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--sl-text-tertiary);
}

.status-dot.online {
  background: var(--sl-success);
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.4);
}

.status-text {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: var(--sl-space-sm);
  -webkit-app-region: no-drag;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  -webkit-app-region: no-drag;
  cursor: pointer;
  z-index: 10;
}

.win-btn:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.win-btn-close:hover {
  background: var(--sl-error);
  color: white;
}

.language-selector {
  position: relative;
  cursor: pointer;
  padding: 6px 12px;
  border-radius: var(--sl-radius-md);
  transition: background-color var(--sl-transition-fast);
  display: flex;
  align-items: center;
  gap: 4px;
}

.language-selector:hover {
  background: var(--sl-bg-tertiary);
}

.language-text {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  font-weight: 500;
}

.language-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--sl-bg-primary);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-md);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  min-width: 100px;
  z-index: 9999;
  overflow: hidden;
}

.language-item {
  padding: 8px 16px;
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
}

.language-item:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.click-outside {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 999;
  pointer-events: auto;
}
</style>
