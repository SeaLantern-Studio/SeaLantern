<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import AppLayout from "@components/layout/AppLayout.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import TermsDialog from "@components/common/TermsDialog.vue";
import SLContextMenu from "@components/common/SLContextMenu.vue";
import { PluginComponentRenderer } from "@components/plugin";
import { useUpdateStore } from "@stores/updateStore";
import { useSettingsStore, dispatchSettingsUpdate } from "@stores/settingsStore";
import { usePluginStore } from "@stores/pluginStore";
import { useContextMenuStore } from "@stores/contextMenuStore";
import { useServerStore } from "@stores/serverStore";
import { useToast } from "cmzya-modern-ui";
import { isBrowserEnv } from "@api/tauri";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// 主题/字体/开发者模式的应用统一由 AppLayout 负责,App.vue 仅加载设置并派发更新事件

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
const toast = useToast();

interface ServerStartFallbackEventPayload {
  serverId: string;
  serverName: string;
  fromMode: string;
  toMode: string;
  reason: string;
}

async function handleGlobalContextMenu(event: MouseEvent) {
  // 在浏览器环境（Docker 模式）下，不阻止右键菜单，允许开发者工具
  if (isBrowserEnv()) {
    return;
  }

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
let serverStartFallbackUnlisten: UnlistenFn | null = null;

onMounted(async () => {
  // 监听服务器错误事件并播放提示音（仅 Tauri 环境）
  if (!isBrowserEnv()) {
    serverErrorUnlisten = await listen("server-error", () => {
      playNotificationSound();
    });
    serverStartFallbackUnlisten = await listen<ServerStartFallbackEventPayload>(
      "server-start-fallback",
      ({ payload }) => {
        const displayName = payload.serverName || payload.serverId;
        toast.warning({
          title: `Server ${displayName}`,
          description: `Failed to start via JAR, fell back to ${payload.toMode} mode (${payload.reason})`,
          duration: 5000,
        });
      },
    );
  }

  contextMenuStore.initContextMenuListener();
  document.addEventListener("contextmenu", handleGlobalContextMenu);

  // 插件事件监听相互独立,并行初始化以缩短启动时间;任一失败不影响其他
  await Promise.allSettled([
    pluginStore.initUiEventListener(),
    pluginStore.initSidebarEventListener(),
    pluginStore.initPermissionLogListener(),
    pluginStore.initPluginLogListener(),
    pluginStore.initComponentEventListener(),
    pluginStore.initI18nEventListener(),
  ]);

  try {
    await settingsStore.loadSettings();
    // 设置加载完成后派发更新事件,由 AppLayout 统一应用主题/字体/开发者模式
    // (AppLayout 在父组件 onMounted 之前已 mount,可能用了默认 settings,这里通知其重新应用)
    dispatchSettingsUpdate(["Appearance", "Developer"], settingsStore.settings);

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
  if (serverStartFallbackUnlisten) {
    serverStartFallbackUnlisten();
    serverStartFallbackUnlisten = null;
  }

  document.removeEventListener("contextmenu", handleGlobalContextMenu);
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

  // 等待设置加载完成后再检查协议同意状态
  const checkTerms = () => {
    if (settingsStore.isLoaded) {
      const settings = settingsStore.settings;
      if (!settings.agreed_to_terms) {
        showTermsDialog.value = true;
      }
      // Dev模式下跳过更新检查, 想要检查更新去关于页面检查
      if (!import.meta.env.DEV) {
        updateStore.checkForUpdateOnStartup();
      }
    } else {
      // 如果还没加载完，等待一小段时间后重试
      setTimeout(checkTerms, 50);
    }
  };

  checkTerms();
}

function handleUpdateModalClose() {
  updateStore.hideUpdateModal();
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
    <cmz-toast />
  </template>
  <SLContextMenu />
</template>

<style src="@styles/app.css"></style>
