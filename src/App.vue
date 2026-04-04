<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import AppLayout from "@components/layout/AppLayout.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import TermsDialog from "@components/common/TermsDialog.vue";
import SLContextMenu from "@components/common/SLContextMenu.vue";
import { PluginComponentRenderer } from "@components/plugin";
import { useUpdateStore } from "@stores/updateStore";
import { useSettingsStore } from "@stores/settingsStore";
import { usePluginStore } from "@stores/pluginStore";
import { useContextMenuStore } from "@stores/contextMenuStore";
import { useServerStore } from "@stores/serverStore";
import {
  applyTheme,
  applyFontSize,
  applyFontFamily,
  applyMinimalMode,
  applyDeveloperMode,
} from "@utils/theme";
import { SETTINGS_UPDATE_EVENT, type SettingsUpdateEvent } from "@stores/settingsStore";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// 播放提示音（使用 Web Audio API 生成）
function playNotificationSound() {
  try {
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    // 生成双音提示（类似系统通知声）
    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(880, audioContext.currentTime); // A5
    oscillator.frequency.setValueAtTime(1100, audioContext.currentTime + 0.1); // C#6

    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.3);

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.3);
  } catch (e) {
    console.warn("播放提示音失败:", e);
  }
}

const showSplash = ref(true);
const isInitializing = ref(true);
const showTermsDialog = ref(false);
const updateStore = useUpdateStore();
const settingsStore = useSettingsStore();
const pluginStore = usePluginStore();
const contextMenuStore = useContextMenuStore();
const serverStore = useServerStore();

async function handleGlobalContextMenu(event: MouseEvent) {
  // 当开发者模式启用时，允许默认的右键菜单行为以打开开发者工具
  if (settingsStore.settings.developer_mode) {
    return;
  }

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

let serverErrorUnlisten: UnlistenFn | null = null;

onMounted(async () => {
  // 监听服务器错误事件并播放提示音
  serverErrorUnlisten = await listen("server-error", () => {
    playNotificationSound();
  });

  contextMenuStore.initContextMenuListener();
  document.addEventListener("contextmenu", handleGlobalContextMenu);

  await pluginStore.initUiEventListener();
  await pluginStore.initSidebarEventListener();
  await pluginStore.initPermissionLogListener();
  await pluginStore.initPluginLogListener();
  await pluginStore.initComponentEventListener();
  await pluginStore.initI18nEventListener();

  await new Promise((resolve) => setTimeout(resolve, 500));

  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);

  try {
    await settingsStore.loadSettings();
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");
    applyMinimalMode(settings.minimal_mode || false);
    applyDeveloperMode(settings.developer_mode || false);

    // 托盘图标已在 Rust 后端创建，前端不需要再创建
    // 相关代码在 src-tauri/src/lib.rs 的 .setup() 中

    try {
      await pluginStore.loadPlugins();
    } catch (pluginErr) {
      console.warn("Failed to load plugins during startup:", pluginErr);
    }

    // 加载服务器列表并扫描端口信息
    try {
      await serverStore.refreshList();
    } catch (serverErr) {
      console.warn("Failed to load servers during startup:", serverErr);
    }
  } catch (e) {
    console.error("Failed to load settings during startup:", e);
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(() => {
  // 清理 server-error 事件监听器
  if (serverErrorUnlisten) {
    serverErrorUnlisten();
    serverErrorUnlisten = null;
  }

  document.removeEventListener("contextmenu", handleGlobalContextMenu);
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdate as EventListener);
  contextMenuStore.cleanupContextMenuListener();

  pluginStore.cleanupUiEventListener();
  pluginStore.cleanupSidebarEventListener();
  pluginStore.cleanupPermissionLogListener();
  pluginStore.cleanupPluginLogListener();
  pluginStore.cleanupComponentEventListener();
  pluginStore.cleanupI18nEventListener();
});

async function handleAgreeTerms() {
  try {
    await settingsStore.updatePartial({ agreed_to_terms: true });
    showTermsDialog.value = false;
  } catch (error) {
    console.error("Failed to save terms agreement:", error);
  }
}

function handleSplashReady() {
  if (isInitializing.value) return;
  showSplash.value = false;

  // 检查是否需要显示协议同意弹窗
  const settings = settingsStore.settings;
  if (!settings.agreed_to_terms) {
    showTermsDialog.value = true;
  }

  // Dev模式下跳过更新检查, 想要检查更新去关于页面检查
  if (!import.meta.env.DEV) {
    updateStore.checkForUpdateOnStartup();
  }
}

function handleUpdateModalClose() {
  updateStore.hideUpdateModal();
}

function handleSettingsUpdate(e: CustomEvent<SettingsUpdateEvent>) {
  const { settings } = e.detail;
  applyDeveloperMode(settings.developer_mode || false);
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

    <TermsDialog
      :visible="showTermsDialog"
      @agree="handleAgreeTerms"
      @close="showTermsDialog = false"
    />

    <PluginComponentRenderer />
  </template>
  <SLContextMenu />
</template>

<style src="@styles/app.css"></style>
