<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, Copy, Check, Globe } from "@lucide/vue";
import { useI18nStore } from "@stores/i18nStore";
import { i18n } from "@language";
import SLModal from "@components/common/SLModal.vue";
import SLButton from "@components/common/SLButton.vue";
import SLCheckbox from "@components/common/SLCheckbox.vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Menu, MenuButton, MenuItems, MenuItem } from "@headlessui/vue";
import { isMacOSPlatform } from "@utils/platform";
import { getAppDisplayName } from "@utils/theme";
import { useSettingsStore } from "@stores/settingsStore";

const route = useRoute();
const appWindow = getCurrentWindow();
const i18nStore = useI18nStore();
const settingsStore = useSettingsStore();
const showCloseModal = ref(false);
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

const appDisplayName = computed(() => {
  if (!settingsStore.isLoaded) {
    return i18n.t("common.app_name");
  }

  return getAppDisplayName(settingsStore.settings);
});

const primaryLanguages = computed(() => i18nStore.localeOptions);

let unlistenResize: (() => void) | null = null;
let unlistenCloseRequested: UnlistenFn | null = null;
let isUnmounted = false;

onMounted(async () => {
  isUnmounted = false;
  await settingsStore.ensureLoaded();
  closeAction.value = settingsStore.settings.close_action || "ask";

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
});

onUnmounted(() => {
  isUnmounted = true;
  if (unlistenResize) {
    unlistenResize();
  }
  if (unlistenCloseRequested) {
    unlistenCloseRequested();
    unlistenCloseRequested = null;
  }
});

watch(
  () => settingsStore.settings.close_action,
  (value) => {
    closeAction.value = value || "ask";
  },
);

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
  if (rememberChoice.value) {
    const nextCloseAction = option === "minimize" ? "minimize" : "close";
    closeAction.value = nextCloseAction;
    try {
      await settingsStore.setCloseAction(nextCloseAction);
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
const isChangingLanguage = ref(false);

async function handleLanguageClick(locale: string, close?: () => void) {
  if (isChangingLanguage.value) return;

  isChangingLanguage.value = true;
  try {
    const switched = await i18nStore.setLocale(locale);
    if (switched) {
      close?.();
    }
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
        </MenuItems>
      </Menu>

      <div class="header-status">
        <span class="status-dot online"></span>
        <span class="status-text">{{ appDisplayName }}</span>
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
