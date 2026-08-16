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
import { desktopApi } from "@api/desktop";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { enqueueAppearanceApply } from "@utils/appearance";

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

// ============================================================
// TODO: 请在后端重构完成后恢复
// 临时开关：禁用右键捕获（开发调试用）
// 原因：右键行为依赖后端 developer_mode 设置，但后端正在重构、
//       暂时无法提供设置，导致开发者模式下右键仍被错误拦截。
// 恢复方式：后端重构完成后，将 TEMP_DISABLE_CONTEXT_MENU_CAPTURE
//          改为 false（或直接删除此开关及相关 return 分支）即可。
// ============================================================
const TEMP_DISABLE_CONTEXT_MENU_CAPTURE = true;

async function handleGlobalContextMenu(event: MouseEvent) {
  // TODO: 请在后端重构完成后恢复（临时禁用右键捕获，见上方开关说明）
  if (TEMP_DISABLE_CONTEXT_MENU_CAPTURE) {
    return;
  }

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

  await contextMenuStore.initContextMenuListener();
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

  // 关键路径:只需等待设置加载完成,用于应用主题/字体后即可关闭启动屏
  // 插件加载与服务器扫描属于非关键数据,延后到主界面显示后再异步补全
  try {
    await settingsStore.loadSettings();
    // 在启动屏消失前应用持久化外观，避免主页挂载时才突然切换透明度。
    await enqueueAppearanceApply(settingsStore.settings);
    // 设置加载完成后派发更新事件,由 AppLayout 统一应用主题/字体/开发者模式
    // (AppLayout 在父组件 onMounted 之前已 mount,可能用了默认 settings,这里通知其重新应用)
    dispatchSettingsUpdate(["Appearance", "Developer"], settingsStore.settings);
  } catch (e) {
    console.error("Failed to load settings during startup:", e);
  } finally {
    // 窗口在 Tauri 配置中隐藏创建；外观与启动屏就绪后再显示，避免 Windows
    // 先绘制系统标题栏或默认背景。先显示启动屏，再允许其结束动画。
    await nextTick();
    try {
      await desktopApi.markFrontendReady();
    } catch (error) {
      console.error("Failed to reveal the main window after frontend initialization:", error);
    }
    isInitializing.value = false;
  }

  // 非关键路径:主界面已显示,后台异步补全插件与服务器数据
  // 任一失败不影响另一个,也不阻塞首屏渲染
  Promise.allSettled([
    pluginStore.loadPlugins().catch((e) => console.warn("Failed to load plugins:", e)),
    serverStore.refreshList().catch((e) => console.warn("Failed to load servers:", e)),
  ]);
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
