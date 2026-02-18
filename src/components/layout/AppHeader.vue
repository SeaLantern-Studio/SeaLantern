<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, reactive } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18nStore } from "../../stores/i18nStore";
import { i18n } from "../../locales";
import SLModal from "../common/SLModal.vue";
import SLButton from "../common/SLButton.vue";
import { settingsApi, type AppSettings } from "../../api/settings";

import { Minus, Square, X } from 'lucide-vue-next';
import { Menu, MenuButton, MenuItems, MenuItem } from '@headlessui/vue';


const route = useRoute();
const appWindow = getCurrentWindow();
const i18nStore = useI18nStore();
const showCloseModal = ref(false);
const perLocaleProgress = reactive<Record<string, { loaded: number; total: number | null }>>({});
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
  const current = languageOptions.value.find((option) => option.code === (i18nStore.currentLocale as any).value || i18nStore.currentLocale);
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
function setLanguage(locale: string) {
  i18nStore.setLocale(locale);
}

async function handleLanguageClick(locale: string) {
  // For local languages we can just switch immediately
  if (locale === "zh-CN" || locale === "en-US") {
    setLanguage(locale);
    return;
  }

  // trigger download and then switch (downloadLocale logs errors internally)
  await i18nStore.downloadLocale(locale);
  setLanguage(locale);
}

function computeOverallProgress() {
  const locales = Object.keys(perLocaleProgress);
  if (locales.length === 0) return 0;
  const completed = locales.filter((k) => perLocaleProgress[k].total && perLocaleProgress[k].loaded >= (perLocaleProgress[k].total ?? 0)).length;
  return Math.round((completed / locales.length) * 100);
}
</script>

<template>
  <header class="app-header glass-subtle">
    <div class="header-left">
      <h2 class="page-title">{{ pageTitle }}</h2>
    </div>

    <div class="header-center" data-tauri-drag-region></div>

    <div class="header-right">
      <Menu as="div" class="language-selector">
        <MenuButton class="language-text">
          {{ currentLanguageText }}
        </MenuButton>
        <MenuItems class="language-menu">
          <MenuItem v-for="option in languageOptions" :key="option.code" as="div">
            <div class="language-item">
              <div class="language-item-main" @click.stop="() => handleLanguageClick(option.code)">
                <span class="language-label">{{ option.label }}</span>
                <span class="locale-progress" v-if="i18nStore.getLocaleProgress(option.code) > 0">{{ i18nStore.getLocaleProgress(option.code) }}%</span>
                <span class="locale-progress-bar" v-if="i18nStore.getLocaleProgress(option.code) > 0 && i18nStore.getLocaleProgress(option.code) < 100">
                  <span class="locale-progress-bar-inner" :style="{ width: i18nStore.getLocaleProgress(option.code) + '%' }"></span>
                </span>
              </div>
            </div>
          </MenuItem>
        </MenuItems>
      </Menu>

      <div class="header-status">
        <span class="status-dot online"></span>
        <span class="status-text">{{ i18n.t("common.app_name") }}</span>
      </div>

      <div class="window-controls">
        <button class="win-btn" @click="minimizeWindow" title="最小化">
          <Minus :size="12" />
        </button>
        <button class="win-btn" @click="toggleMaximize" title="最大化">
          <Square :size="12" />
        </button>
        <button class="win-btn win-btn-close" @click="closeWindow" title="关闭">
          <X :size="12" />
        </button>
      </div>
    </div>
  </header>

  <!-- 关闭窗口确认模态框 -->
  <SLModal
    :visible="showCloseModal"
    :title="i18n.t('home.close_window_title')"
    @close="showCloseModal = false"
  >
    <div class="close-modal-content">
      <p>{{ i18n.t("home.close_window_message") }}</p>
      <div class="remember-option">
        <input type="checkbox" id="remember-choice" v-model="rememberChoice" />
        <label for="remember-choice">{{ i18n.t("home.remember_choice") }}</label>
      </div>
      <div class="close-options">
        <SLButton variant="secondary" @click="handleCloseOption('minimize')">{{
          i18n.t("home.close_action_minimize")
        }}</SLButton>
        <SLButton variant="danger" @click="handleCloseOption('close')">{{
          i18n.t("home.close_action_close")
        }}</SLButton>
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

/* locale download UI removed */

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
  margin-top: 8px;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border-light);
  border-radius: var(--sl-radius-lg);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  min-width: 260px;
  max-width: 300px;
  max-height: 320px;
  z-index: 9999;
  padding: 8px;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 2px;
  overflow-y: auto;
  padding: 8px;
}

.language-item {
  padding: 8px 12px;
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: all var(--sl-transition-fast);
  border-radius: var(--sl-radius-sm);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: left;
  width: auto;
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
}

.language-item:hover {
  background: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.language-item { display:flex; align-items:center; justify-content:space-between; gap:8px }
.language-item-main { flex:1 }
.language-item-action { flex:0 0 auto }
.locale-progress { font-size: 0.75rem; margin-left: 8px; color: var(--sl-text-tertiary) }
.language-label { display:inline-block }
.locale-progress-bar {
  display: inline-block;
  width: 72px;
  height: 6px;
  background: rgba(255,255,255,0.06);
  border-radius: 6px;
  margin-left: 8px;
  vertical-align: middle;
  overflow: hidden;
}
.locale-progress-bar-inner {
  height: 100%;
  background: linear-gradient(90deg, var(--sl-primary), var(--sl-success));
  transition: width 0.2s linear;
}

.language-menu::-webkit-scrollbar {
  width: 4px;
}

.language-menu::-webkit-scrollbar-track {
  background: transparent;
}

.language-menu::-webkit-scrollbar-thumb {
  background: var(--sl-border-light);
  border-radius: var(--sl-radius-full);
}

.language-menu::-webkit-scrollbar-thumb:hover {
  background: var(--sl-text-tertiary);
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
