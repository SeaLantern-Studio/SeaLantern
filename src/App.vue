<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { exit } from "@tauri-apps/plugin-process";
import AppLayout from "./components/layout/AppLayout.vue";
import SplashScreen from "./components/splash/SplashScreen.vue";
import UpdateModal from "./components/common/UpdateModal.vue";
import SLCloseDialog from "./components/common/SLCloseDialog.vue";
import { useUpdateStore } from "./stores/updateStore";
import { useSettingsStore } from "./stores/settingsStore";
import { useI18nStore } from "./stores/i18nStore";
import { settingsApi } from "./api/settings";
import { applyTheme, applyFontSize, applyFontFamily } from "./utils/theme";

const showSplash = ref(true);
const isInitializing = ref(true);
const showCloseDialog = ref(false);
const updateStore = useUpdateStore();
const settingsStore = useSettingsStore();
const i18nStore = useI18nStore();
const appWindow = getCurrentWindow();

let unlistenCloseDialog: UnlistenFn | null = null;

onMounted(async () => {
  try {
    await settingsStore.loadSettings();
    await i18nStore.loadLanguageSetting();
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");

    // 监听后端发来的关闭确认事件
    unlistenCloseDialog = await listen("show-close-dialog", () => {
      showCloseDialog.value = true;
    });

    try {
      const { setupTray } = await import("./utils/tray");
      if (typeof setupTray === "function") {
        await setupTray();
        console.log("Tray setup completed");
      }
    } catch (trayErr) {
      console.warn("Failed to set up tray, tray functionality will be unavailable:", trayErr);
    }
  } catch (e) {
    console.error("Failed to load settings during startup:", e);
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(async () => {
  // 清理事件监听
  if (unlistenCloseDialog) {
    unlistenCloseDialog();
  }
});

function handleSplashReady() {
  if (isInitializing.value) return;
  showSplash.value = false;
  appWindow.show();
  updateStore.checkForUpdateOnStartup();
}

function handleUpdateModalClose() {
  updateStore.hideUpdateModal();
}

function handleCloseDialogClose() {
  showCloseDialog.value = false;
}

async function handleCloseDialogConfirm(action: "exit" | "tray", remember: boolean) {
  showCloseDialog.value = false;

  // 如果用户选择记住，保存设置
  if (remember) {
    try {
      const settings = await settingsApi.get();
      settings.close_action = action === "exit" ? "close" : "minimize";
      await settingsApi.save(settings);
    } catch (e) {
      console.error("Failed to save close action setting:", e);
    }
  }

  if (action === "exit") {
    // 使用 exit 强制退出，避免再次触发 CloseRequested 事件
    await exit(0);
  } else {
    // 最小化到托盘
    await appWindow.hide();
  }
}
</script>

<template>
  <transition name="splash-fade">
    <SplashScreen v-if="showSplash" :loading="isInitializing" @ready="handleSplashReady" />
  </transition>

  <template v-if="!showSplash">
    <AppLayout />

    <UpdateModal
      v-if="updateStore.isUpdateModalVisible && updateStore.isUpdateAvailable"
      @close="handleUpdateModalClose"
    />

    <SLCloseDialog
      :visible="showCloseDialog"
      @close="handleCloseDialogClose"
      @confirm="handleCloseDialogConfirm"
    />
  </template>
</template>

<style>
#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.splash-fade-leave-active {
  transition: opacity 0.3s ease;
}

.splash-fade-leave-to {
  opacity: 0;
}
</style>
