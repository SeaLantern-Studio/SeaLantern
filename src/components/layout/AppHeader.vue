<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, ChevronDown, ChevronUp, Copy, Check, Globe } from "lucide-vue-next";
import { useI18nStore } from "@stores/i18nStore";
import { i18n } from "@language";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { settingsApi, type AppSettings } from "@api/settings";
import { Menu, MenuButton, MenuItems, MenuItem } from "@headlessui/vue";
import { isMacOSPlatform } from "@utils/platform";
import {
  dispatchSettingsUpdate,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";

const route = useRoute();
const appWindow = getCurrentWindow();
const i18nStore = useI18nStore();
const showCloseModal = ref(false);
const settings = ref<AppSettings | null>(null);
const closeAction = ref<string>("ask"); // ask, minimize, close
const rememberChoice = ref(false);
const isMaximized = ref(false);
const isMacOS = isMacOSPlatform();

const pageTitle = computed(() => {
  const titleKey = route.meta?.titleKey as string;
  if (titleKey) {
    return i18n.t(titleKey);
  }
  return i18n.t("common.app_name");
});

const primaryLanguages = computed(() => {
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
let unlistenResize: (() => void) | null = null;
let unlistenCloseRequested: UnlistenFn | null = null;
let isUnmounted = false;

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
  isUnmounted = false;
  await loadSettings();

  // 初始化最大化状态
  isMaximized.value = await appWindow.isMaximized();

  // 监听窗口大小变化
  unlistenResize = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized();
  });

  const unlisten = await listen("close-requested", () => {
    showCloseModal.value = true;
  });
  if (isUnmounted) {
    unlisten();
  } else {
    unlistenCloseRequested = unlisten;
  }

  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
});

onUnmounted(() => {
  isUnmounted = true;
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
  if (unlistenResize) {
    unlistenResize();
  }
  if (unlistenCloseRequested) {
    unlistenCloseRequested();
    unlistenCloseRequested = null;
  }
});

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>) {
  const { settings: newSettings } = e.detail;
  settings.value = newSettings;
  closeAction.value = newSettings.close_action || "ask";
}

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
      const result = await settingsApi.saveWithDiff(settings.value);
      dispatchSettingsUpdate(result.changed_groups, result.settings);
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  if (option === "minimize") {
    await minimizeToTray();
  } else {
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

const isChangingLanguage = ref(false);

async function handleLanguageClick(locale: string, close?: () => void) {
  if (isChangingLanguage.value) return;

  isChangingLanguage.value = true;
  try {
    // For local languages we can just switch immediately
    if (locale === "zh-CN" || locale === "en-US") {
      setLanguage(locale);
      close?.();
      return;
    }

    // trigger download and then switch (downloadLocale logs errors internally)
    await i18nStore.downloadLocale(locale);
    setLanguage(locale);
    close?.();
  } finally {
    isChangingLanguage.value = false;
  }
}

function isActive(code: string) {
  return i18nStore.currentLocale == code;
}
</script>

<template>
  <header
    class="app-header"
    :class="{ 'macos-overlay': isMacOS, 'glass-strong': !isMacOS }"
    data-tauri-drag-region
  >
    <div class="header-left" v-if="!isMacOS">
      <h2 class="page-title" data-tauri-drag-region>{{ pageTitle }}</h2>
    </div>

    <div class="header-center">
      <h2 class="page-title" v-if="isMacOS" data-tauri-drag-region>{{ pageTitle }}</h2>
    </div>

    <div class="header-right">
      <Menu as="div" class="language-selector">
        <MenuButton class="language-button">
          <Globe class="language-text" :size="16" />
        </MenuButton>
        <MenuItems class="language-menu">
          <!-- 主要语言 -->
          <MenuItem v-for="option in primaryLanguages" :key="option.code" v-slot="{ close }">
            <div
              class="language-item"
              :class="{ active: isActive(option.code) }"
              @click="() => handleLanguageClick(option.code, close)"
            >
              <div class="language-item-main">
                <span class="language-label">{{ option.label }}</span>
              </div>
              <Check v-if="isActive(option.code)" :size="16" aria-hidden="true" />
            </div>
          </MenuItem>

          <!-- 其他语言（仅在展开时显示） -->
          <Transition name="language-toggle">
            <div v-if="showMoreLanguages" class="language-more-list">
              <MenuItem v-for="option in otherLanguages" :key="option.code" v-slot="{ close }">
                <div
                  class="language-item"
                  :class="{ active: isActive(option.code) }"
                  @click="() => handleLanguageClick(option.code, close)"
                >
                  <div class="language-item-main">
                    <span class="language-label">{{ option.label }}</span>
                  </div>
                  <Check v-if="isActive(option.code)" :size="16" aria-hidden="true" />
                </div>
              </MenuItem>
            </div>
          </Transition>

          <!-- 更多语言选项（固定在最底部） -->
          <div class="language-item-full-width">
            <div class="language-item language-item-arrow" @click="toggleMoreLanguages">
              <div class="language-item-main">
                <ChevronUp v-if="showMoreLanguages" :size="16" class="arrow-icon" />
                <ChevronDown v-else :size="16" class="arrow-icon" />
              </div>
            </div>
          </div>
        </MenuItems>
      </Menu>

      <div class="header-status">
        <span class="status-dot online"></span>
        <span class="status-text">{{ i18n.t("common.app_name") }}</span>
      </div>

      <div v-if="!isMacOS" class="window-controls">
        <button class="win-btn" @click="minimizeWindow" :title="i18n.t('common.minimize')">
          <Minus :size="12" />
        </button>
        <button
          class="win-btn"
          @click="toggleMaximize"
          :title="isMaximized ? i18n.t('common.restore') : i18n.t('common.maximize')"
        >
          <Copy v-if="isMaximized" :size="12" />
          <Square v-else :size="12" />
        </button>
        <button class="win-btn win-btn-close" @click="closeWindow" :title="i18n.t('common.close')">
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
        <SLCheckbox v-model="rememberChoice" :label="i18n.t('home.remember_choice')" />
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
<style src="@styles/components/layout/AppHeader.css" scoped></style>
