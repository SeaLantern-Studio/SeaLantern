<script setup lang="ts">
import { onMounted, onUnmounted, computed, watch, ref } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import { useUiStore } from "../../stores/uiStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { applyAcrylic } from "../../api/settings";
import {
  applyTheme,
  applyFontFamily,
  applyFontSize,
  applyColors,
  applyDeveloperMode,
  getEffectiveTheme,
} from "../../utils/theme";
import { isMacOS } from "../../utils/platform";

const ui = useUiStore();
const settingsStore = useSettingsStore();
const isMac = ref(false);

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);

let systemThemeQuery: MediaQueryList | null = null;

/**
 * 应用亚克力效果
 */
async function applyAcrylicEffect(enabled: boolean, theme: string): Promise<void> {
  document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");

  if (!settingsStore.acrylicSupported) {
    return;
  }

  if (enabled) {
    const isDark = getEffectiveTheme(theme) === "dark";
    try {
      await applyAcrylic(true, isDark);
    } catch (e) {
      console.error("Failed to apply acrylic:", e);
    }
  } else {
    try {
      await applyAcrylic(false, false);
    } catch (e) {
      console.error("Failed to clear acrylic:", e);
    }
  }
}

/**
 * 处理系统主题变化
 */
function handleSystemThemeChange(): void {
  const settings = settingsStore.settings;
  if (settings.theme === "auto") {
    applyTheme("auto");
    if (settings.acrylic_enabled && settingsStore.acrylicSupported) {
      applyAcrylicEffect(true, "auto");
    }
    applyColors(settings);
  }
}

/**
 * 应用所有设置
 */
async function applyAllSettings(): Promise<void> {
  const settings = settingsStore.settings;

  applyTheme(settings.theme || "auto");
  applyFontSize(settings.font_size || 14);
  applyFontFamily(settings.font_family || "");

  if (settingsStore.acrylicSupported) {
    await applyAcrylicEffect(settings.acrylic_enabled, settings.theme || "auto");
  } else {
    document.documentElement.setAttribute("data-acrylic", "false");
  }

  applyColors(settings);
  applyDeveloperMode(settings.developer_mode || false);
}

onMounted(async () => {
  isMac.value = await isMacOS();
  await applyAllSettings();

  window.addEventListener("settings-updated", handleSettingsUpdated);

  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", handleSystemThemeChange);
});

onUnmounted(() => {
  window.removeEventListener("settings-updated", handleSettingsUpdated);
  if (systemThemeQuery) {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  }
});

/**
 * 处理设置更新事件
 */
async function handleSettingsUpdated(): Promise<void> {
  await settingsStore.loadSettings();
  await applyAllSettings();
}

watch(
  () => settingsStore.settings,
  async () => {
    await applyAllSettings();
  },
  { deep: true },
);

const backgroundStyle = computed(() => {
  if (!backgroundImage.value) return {};
  return {
    backgroundImage: `url(${backgroundImage.value})`,
    backgroundSize: backgroundSize.value,
    backgroundPosition: "center",
    backgroundRepeat: "no-repeat",
    opacity: backgroundOpacity.value,
    filter: `blur(${backgroundBlur.value}px) brightness(${backgroundBrightness.value})`,
  };
});
</script>

<template>
  <div class="app-layout" :class="{ 'app-layout-macos': isMac }">
    <!-- macOS 专用标题栏区域（红绿灯按钮所在区域），使用与侧边栏相同的 glass-strong 样式 -->
    <div v-if="isMac" class="macos-titlebar glass-strong" data-tauri-drag-region></div>

    <div class="app-content-wrapper">
      <div class="app-background" :style="backgroundStyle"></div>
      <AppSidebar :is-macos="isMac" />
      <div class="app-main" :class="{ 'sidebar-collapsed': ui.sidebarCollapsed }">
        <AppHeader />
        <main class="app-content">
          <router-view v-slot="{ Component }">
            <transition name="page-fade" mode="out-in">
              <keep-alive :max="5">
                <component :is="Component" />
              </keep-alive>
            </transition>
          </router-view>
        </main>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-layout {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  background-color: var(--sl-bg);
  overflow: hidden;
}

/* macOS vibrancy 模式下使用透明背景以显示圆角和毛玻璃效果 */
:global([data-vibrancy="true"]) .app-layout {
  background-color: transparent;
}

/* macOS 专用标题栏区域 - 使用 glass-strong 类继承侧边栏样式 */
.macos-titlebar {
  height: 28px;
  flex-shrink: 0;
  -webkit-app-region: drag;
  border-bottom: none;
  border-top: none;
  border-left: none;
  border-right: 1px solid var(--sl-border-light);
}

/* 内容包装器 */
.app-content-wrapper {
  position: relative;
  display: flex;
  flex: 1;
  min-height: 0;
}

.app-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 0;
  pointer-events: none;
  transition: all 0.3s ease;
}

.app-main {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  flex-direction: column;
  margin-left: var(--sl-sidebar-width);
  transition: margin-left 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  min-width: 0;
}

.app-main.sidebar-collapsed {
  margin-left: var(--sl-sidebar-collapsed-width);
}

/* macOS 下侧边栏不是 fixed，所以 app-main 不需要 margin-left */
.app-layout-macos .app-main {
  margin-left: 0;
}

.app-layout-macos .app-main.sidebar-collapsed {
  margin-left: 0;
}

.app-content {
  flex: 1;
  padding: var(--sl-space-lg);
  overflow-y: auto;
  overflow-x: hidden;
}

.page-fade-enter-active,
.page-fade-leave-active {
  transition:
    opacity 0.15s cubic-bezier(0.4, 0, 0.2, 1),
    transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(4px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-2px);
}
</style>
