<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import AppLayout from "@components/layout/AppLayout.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import SLContextMenu from "@components/common/SLContextMenu.vue";
import { PluginComponentRenderer } from "@components/plugin";
import { useUpdateStore } from "@stores/updateStore";
import { useSettingsStore } from "@stores/settingsStore";
import { usePluginStore } from "@stores/pluginStore";
import { useContextMenuStore } from "@stores/contextMenuStore";
import { useServerStore } from "@stores/serverStore";
import { applyTheme, applyFontSize, applyFontFamily } from "@utils/theme";

const showSplash = ref(true);
const isInitializing = ref(true);
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
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");

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
  document.removeEventListener("contextmenu", handleGlobalContextMenu);
  contextMenuStore.cleanupContextMenuListener();

  pluginStore.cleanupUiEventListener();
  pluginStore.cleanupSidebarEventListener();
  pluginStore.cleanupPermissionLogListener();
  pluginStore.cleanupPluginLogListener();
  pluginStore.cleanupComponentEventListener();
  pluginStore.cleanupI18nEventListener();
});

function handleSplashReady() {
  if (isInitializing.value) return;
  showSplash.value = false;
  updateStore.checkForUpdateOnStartup();
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

    <PluginComponentRenderer />
  </template>
  <SLContextMenu />
</template>

<style src="@styles/app.css"></style>
