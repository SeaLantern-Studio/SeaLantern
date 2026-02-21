<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, reactive } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, ChevronDown, ChevronUp } from "lucide-vue-next";
import { useI18nStore } from "../../stores/i18nStore";
import { i18n } from "../../language";
import SLModal from "../common/SLModal.vue";
import SLButton from "../common/SLButton.vue";
import { settingsApi, type AppSettings } from "../../api/settings";
import { Menu, MenuButton, MenuItems, MenuItem } from "@headlessui/vue";

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

const primaryLanguages = computed(() => {
  // 为了确保语言切换时重新计算，我们使用 i18nStore.currentLocale 作为依赖
  const currentLocale = i18nStore.currentLocale;
  const primaryCodes = ["zh-CN", "zh-TW", "en-US", "ja-JP"];

  return primaryCodes.map((code) => {
    // 尝试从语言文件中获取 languageName
    const translations = i18n.getTranslations();
    const languageName = translations[code as keyof typeof translations]?.languageName;

    // 如果有 languageName，直接使用；否则使用原来的标签键
    let label = "";
    if (languageName) {
      label = languageName;
    } else {
      const labelKey = {
        "zh-CN": "header.chinese",
        "en-US": "header.english",
        "zh-TW": "header.chinese_tw",
        "ja-JP": "header.japanese",
      }[code];
      label = i18n.t(labelKey || "header.english");
    }

    return {
      code,
      label,
    };
  });
});

const otherLanguages = computed(() => {
  // 为了确保语言切换时重新计算，我们使用 i18nStore.currentLocale 作为依赖
  const currentLocale = i18nStore.currentLocale;
  const primaryCodes = new Set(["zh-CN", "zh-TW", "en-US", "ja-JP"]);
  const allLocales = i18n.getAvailableLocales();

  return allLocales
    .filter((code) => !primaryCodes.has(code))
    .map((code) => {
      // 尝试从语言文件中获取 languageName
      const translations = i18n.getTranslations();
      const languageName = translations[code as keyof typeof translations]?.languageName;

      // 如果有 languageName，直接使用；否则使用原来的标签键
      let label = "";
      if (languageName) {
        label = languageName;
      } else {
        const labelKey = {
          "de-DE": "header.deutsch",
          "es-ES": "header.spanish",
          "ru-RU": "header.russian",
          "vi-VN": "header.vietnamese",
          "ko-KR": "header.korean",
          "fr-FA": "header.french",
        }[code];
        label = i18n.t(labelKey || code);
      }

      return {
        code,
        label,
      };
    });
});

const showMoreLanguages = ref(false);

function toggleMoreLanguages() {
  showMoreLanguages.value = !showMoreLanguages.value;
}

const currentLanguageText = computed(() => {
  const currentLocale = i18nStore.currentLocale;

  // 尝试从语言文件中获取 languageName
  const translations = i18n.getTranslations();
  const languageName = translations[currentLocale as keyof typeof translations]?.languageName;

  if (languageName) {
    return languageName;
  }

  // 如果没有 languageName，使用原来的逻辑
  const labelKey = {
    "zh-CN": "header.chinese",
    "en-US": "header.english",
    "zh-TW": "header.chinese_tw",
    "de-DE": "header.deutsch",
    "es-ES": "header.spanish",
    "ja-JP": "header.japanese",
    "ru-RU": "header.russian",
    "vi-VN": "header.vietnamese",
    "ko-KR": "header.korean",
    "fr-FA": "header.french",
  }[currentLocale];
  return labelKey ? i18n.t(labelKey) : i18n.t("header.english");
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
    // 使用 exit() 强制退出应用，绕过 CloseRequested 事件
    const { exit } = await import("@tauri-apps/plugin-process");
    await exit(0);
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
  const completed = locales.filter(
    (k) =>
      perLocaleProgress[k].total &&
      perLocaleProgress[k].loaded >= (perLocaleProgress[k].total ?? 0),
  ).length;
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
          <!-- 主要语言 -->
          <MenuItem
            v-for="option in primaryLanguages"
            :key="option.code"
            as="div"
            @click="
              (e) => {
                e.preventDefault();
                e.stopPropagation();
                handleLanguageClick(option.code);
              }
            "
          >
            <div class="language-item">
              <div class="language-item-main">
                <span class="language-label">{{ option.label }}</span>
              </div>
            </div>
          </MenuItem>

          <!-- 更多语言选项 -->
          <MenuItem
            as="div"
            @click="
              (e) => {
                e.preventDefault();
                e.stopPropagation();
                toggleMoreLanguages();
              }
            "
            class="language-item-full-width"
          >
            <div class="language-item language-item-arrow">
              <div class="language-item-main">
                <ChevronDown v-if="!showMoreLanguages" :size="16" class="arrow-icon" />
                <ChevronUp v-else :size="16" class="arrow-icon" />
              </div>
            </div>
          </MenuItem>

          <!-- 其他语言（仅在展开时显示） -->
          <template v-if="showMoreLanguages">
            <MenuItem
              v-for="option in otherLanguages"
              :key="option.code"
              as="div"
              @click="
                (e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  handleLanguageClick(option.code);
                }
              "
            >
              <div class="language-item">
                <div class="language-item-main">
                  <span class="language-label">{{ option.label }}</span>
                </div>
              </div>
            </MenuItem>
          </template>
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
  color: var(--sl-text-inverse);
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

.language-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.language-item-main {
  flex: 1;
}
.language-item-action {
  flex: 0 0 auto;
}
.language-label {
  display: inline-block;
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

/* 让更多语言选项占据两列宽度 */
.language-item-full-width {
  grid-column: span 2;
}

/* 箭头图标居中 */
.language-item-arrow {
  justify-content: center;
}

.arrow-icon {
  color: var(--sl-text-secondary);
  transition: transform var(--sl-transition-fast);
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
