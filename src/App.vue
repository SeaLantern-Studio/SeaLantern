<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { exit } from "@tauri-apps/plugin-process";
import AppLayout from "./components/layout/AppLayout.vue";
import SplashScreen from "./components/splash/SplashScreen.vue";
import UpdateModal from "./components/common/UpdateModal.vue";
import SLContextMenu from "./components/common/SLContextMenu.vue";
import SLCloseDialog from "./components/common/SLCloseDialog.vue";
import { PluginComponentRenderer } from "./components/plugin";
import { useUpdateStore } from "./stores/updateStore";
import { useSettingsStore } from "./stores/settingsStore";
import { usePluginStore } from "./stores/pluginStore";
import { useContextMenuStore } from "./stores/contextMenuStore";
import { useI18nStore } from "./stores/i18nStore";
import { settingsApi } from "./api/settings";
import { applyTheme, applyFontSize, applyFontFamily } from "./utils/theme";

const showSplash = ref(true);
const isInitializing = ref(true);
const showCloseDialog = ref(false);
const updateStore = useUpdateStore();
const settingsStore = useSettingsStore();
const pluginStore = usePluginStore();
const contextMenuStore = useContextMenuStore();
const i18nStore = useI18nStore();
const appWindow = getCurrentWindow();

let unlistenCloseDialog: UnlistenFn | null = null;

async function handleGlobalContextMenu(event: MouseEvent) {
  event.preventDefault();

  const wasVisible = contextMenuStore.visible;
  if (wasVisible) {
    contextMenuStore.hideContextMenu();
    await nextTick();
  }

  const allElements = document.elementsFromPoint(event.clientX, event.clientY) as HTMLElement[];
  const filteredElements = allElements.filter((el) => !el.closest(".sl-context-menu-backdrop"));

  let ctx = "global";
  let targetData = "";

  for (const el of filteredElements) {
    if (el.dataset?.contextMenu) {
      ctx = el.dataset.contextMenu;
      targetData = el.dataset.contextMenuTarget ?? "";
      break;
    }
  }

  if (!targetData) {
    const target = filteredElements[0];
    if (target) {
      const tag = target.tagName.toLowerCase();
      const text = target.textContent?.trim() || "";
      if (text.length > 100) {
        targetData = `${tag}(${text.substring(0, 100)}...)`;
      } else if (text) {
        targetData = `${tag}(${text})`;
      } else {
        targetData = tag;
      }
    }
  }

  if (ctx !== "global" && !contextMenuStore.hasMenuItems(ctx)) {
    ctx = "global";
  }

  if (!contextMenuStore.hasMenuItems(ctx)) return;

  contextMenuStore.showContextMenu(ctx, event.clientX, event.clientY, targetData);
}

onMounted(async () => {
  contextMenuStore.initContextMenuListener();
  document.addEventListener("contextmenu", handleGlobalContextMenu);

  await pluginStore.initUiEventListener();
  await pluginStore.initSidebarEventListener();
  await pluginStore.initPermissionLogListener();
  await pluginStore.initPluginLogListener();
  await pluginStore.initComponentEventListener();
  await pluginStore.initI18nEventListener();

  await new Promise((resolve) => setTimeout(resolve, 500));

  try {
    await settingsStore.loadSettings();
    await i18nStore.loadLanguageSetting();
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");

    // 托盘图标已在 Rust 后端创建，前端不需要再创建
    // 相关代码在 src-tauri/src/lib.rs 的 .setup() 中

    // 监听后端发来的关闭确认事件
    unlistenCloseDialog = await listen("show-close-dialog", () => {
      showCloseDialog.value = true;
    });

    try {
      await pluginStore.loadPlugins();
    } catch (pluginErr) {
      console.warn("Failed to load plugins during startup:", pluginErr);
    }
  } catch (e) {
    console.error("Failed to load settings during startup:", e);
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  document.removeEventListener("contextmenu", handleGlobalContextMenu);
  contextMenuStore.cleanupContextMenuListener();

  pluginStore.cleanupUiEventListener();
  pluginStore.cleanupSidebarEventListener();
  pluginStore.cleanupPermissionLogListener();
  pluginStore.cleanupPluginLogListener();
  pluginStore.cleanupComponentEventListener();
  pluginStore.cleanupI18nEventListener();

  // 清理关闭对话框事件监听
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

    <PluginComponentRenderer />

    <SLCloseDialog
      :visible="showCloseDialog"
      @close="handleCloseDialogClose"
      @confirm="handleCloseDialogConfirm"
    />
  </template>
  <SLContextMenu />
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
