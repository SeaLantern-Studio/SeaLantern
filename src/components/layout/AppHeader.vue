<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18nStore } from "../../stores/i18nStore";
import { i18n } from "../../locales";
import SLModal from "../common/SLModal.vue";
import SLButton from "../common/SLButton.vue";
import { settingsApi, type AppSettings } from "../../api/settings";

const route = useRoute();
const appWindow = getCurrentWindow();
const i18nStore = useI18nStore();
const showLanguageMenu = ref(false);
const showCloseModal = ref(false);
const settings = ref<AppSettings | null>(null);
const closeAction = ref<string>("ask"); // ask, minimize, close
const rememberChoice = ref(false);

const pageTitle = computed(() => {
  const titleKey = route.meta?.titleKey as string;
  if (titleKey) {
    return i18n.t(titleKey);
  }
  return i18n.t("common.app_name");
});

const languageOptions = computed(() =>
  i18nStore.localeOptions.map((option) => ({
    code: option.code,
    label: i18n.t(option.labelKey),
  })),
);

const currentLanguageText = computed(() => {
  const current = languageOptions.value.find((option) => option.code === i18nStore.currentLocale);
  return current?.label ?? i18n.t("header.english");
});

onMounted(async () => {
  await loadSettings();
  
  // 监听设置更新事件
  window.addEventListener("settings-updated", loadSettings);
});

onUnmounted(() => {
  // 移除设置更新事件监听
  window.removeEventListener("settings-updated", loadSettings);
});

async function loadSettings() {
  try {
    const s = await settingsApi.get();
    settings.value = s;
    closeAction.value = s.close_action || "ask";
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  if (closeAction.value === "ask") {
    showCloseModal.value = true;
  } else if (closeAction.value === "minimize") {
    await minimizeToTray();
  } else {
    await appWindow.close();
  }
}

async function handleCloseOption(option: string) {
  if (rememberChoice.value && settings.value) {
    settings.value.close_action = option === "minimize" ? "minimize" : "close";
    closeAction.value = settings.value.close_action;
    try {
      await settingsApi.save(settings.value);
      // 触发设置更新事件，以便设置界面能够及时更新
      window.dispatchEvent(new CustomEvent("settings-updated"));
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }
  
  if (option === "minimize") {
    await minimizeToTray();
  } else {
    await appWindow.close();
  }
  showCloseModal.value = false;
  rememberChoice.value = false;
}

async function minimizeToTray() {
  try {
    const w = getCurrentWindow();
    await w.hide();
    try {
      await w.setSkipTaskbar(true);
    } catch (e) {
      console.warn("Failed to set skip taskbar:", e);
    }
  } catch (e) {
    console.warn("Failed to hide window for tray minimize:", e);
    await appWindow.minimize();
  }
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
          <div
            v-for="option in languageOptions"
            :key="option.code"
            class="language-item"
            @click.stop="setLanguage(option.code)"
          >
            {{ option.label }}
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

  <!-- 关闭窗口确认模态框 -->
  <SLModal :visible="showCloseModal" :title="i18n.t('home.close_window_title')" @close="showCloseModal = false">
    <div class="close-modal-content">
      <p>{{ i18n.t('home.close_window_message') }}</p>
      <div class="remember-option">
        <input type="checkbox" id="remember-choice" v-model="rememberChoice">
        <label for="remember-choice">{{ i18n.t('home.remember_choice') }}</label>
      </div>
      <div class="close-options">
        <SLButton variant="secondary" @click="handleCloseOption('minimize')">{{ i18n.t('home.close_action_minimize') }}</SLButton>
        <SLButton variant="danger" @click="handleCloseOption('close')">{{ i18n.t('home.close_action_close') }}</SLButton>
      </div>
    </div>
  </SLModal>
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
  background: var(--sl-surface);
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

.close-modal-content {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
  padding: var(--sl-space-md) 0;
}

.remember-option {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  margin-top: var(--sl-space-sm);
  margin-bottom: var(--sl-space-md);
}

.remember-option input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
  accent-color: var(--sl-primary);
  background-color: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-sm);
  transition: all var(--sl-transition-fast);
}

.remember-option input[type="checkbox"]:checked {
  background-color: var(--sl-primary);
  border-color: var(--sl-primary);
}

.remember-option label {
  font-size: 0.875rem;
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.close-options {
  display: flex;
  gap: var(--sl-space-md);
  justify-content: center;
  margin-top: var(--sl-space-md);
}
</style>
