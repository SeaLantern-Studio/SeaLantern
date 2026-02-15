<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import AppSidebar from "./AppSidebar.vue";
import AppHeader from "./AppHeader.vue";
import { useUiStore } from "../../stores/uiStore";
import { settingsApi, applyAcrylic, checkAcrylicSupport } from "../../api/settings";
import { convertFileSrc } from "@tauri-apps/api/core";

const ui = useUiStore();
const backgroundImage = ref("");
const backgroundOpacity = ref(0.3);
const backgroundBlur = ref(0);
const backgroundBrightness = ref(1.0);
const backgroundSize = ref("cover");
const acrylicEnabled = ref(false);
const acrylicSupported = ref(false);
const currentTheme = ref("auto");

let systemThemeQuery: MediaQueryList | null = null;

function getEffectiveTheme(theme: string): "light" | "dark" {
  if (theme === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme as "light" | "dark";
}

function applyTheme(theme: string) {
  const effectiveTheme = getEffectiveTheme(theme);
  document.documentElement.setAttribute("data-theme", effectiveTheme);
  return effectiveTheme;
}

async function applyAcrylicEffect(enabled: boolean, theme: string) {
  document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");

  if (!acrylicSupported.value) {
    return;
  }

  if (enabled) {
    const effectiveTheme = getEffectiveTheme(theme);
    const isDark = effectiveTheme === "dark";
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

// 系统主题变化处理
function handleSystemThemeChange() {
  if (currentTheme.value === "auto") {
    const effectiveTheme = applyTheme("auto");
    // 如果亚克力开启，需要重新应用以匹配新主题
    if (acrylicEnabled.value && acrylicSupported.value) {
      applyAcrylicEffect(true, "auto");
    }
  }
}

onMounted(() => {
  // 立即应用默认主题，确保 UI 快速显示
  applyTheme("auto");

  // 检测亚克力支持（异步，不阻塞 UI）
  checkAcrylicSupport()
    .then((supported) => {
      acrylicSupported.value = supported;
    })
    .catch(() => {
      acrylicSupported.value = false;
    });

  // 异步加载背景设置，不阻塞 UI 渲染
  loadBackgroundSettings().catch((err) => {
    console.error("Failed to load settings initially:", err);
  });

  // 监听设置更新事件
  window.addEventListener("settings-updated", loadBackgroundSettings);

  // 监听系统主题变化
  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", handleSystemThemeChange);
});

onUnmounted(() => {
  window.removeEventListener("settings-updated", loadBackgroundSettings);
  if (systemThemeQuery) {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  }
});

async function loadBackgroundSettings() {
  try {
    const settings = await settingsApi.get();

    // 保存当前主题设置
    currentTheme.value = settings.theme || "auto";
    acrylicEnabled.value = settings.acrylic_enabled;

    // 应用主题
    const effectiveTheme = applyTheme(settings.theme || "auto");

    // 应用字体大小
    document.documentElement.style.fontSize = (settings.font_size || 14) + "px";

    // 应用亚克力效果（只有在支持的系统上）
    if (acrylicSupported.value) {
      await applyAcrylicEffect(settings.acrylic_enabled, settings.theme || "auto");
    } else {
      // 不支持亚克力时，确保 data-acrylic 为 false
      document.documentElement.setAttribute("data-acrylic", "false");
    }

    // 应用背景图片设置
    if (settings.background_image) {
      backgroundImage.value = convertFileSrc(settings.background_image);
    } else {
      backgroundImage.value = "";
    }
    backgroundOpacity.value = settings.background_opacity;
    backgroundBlur.value = settings.background_blur;
    backgroundBrightness.value = settings.background_brightness;
    backgroundSize.value = settings.background_size;
    console.log("Settings loaded:", {
      theme: settings.theme,
      effectiveTheme,
      fontSize: settings.font_size,
      acrylicEnabled: settings.acrylic_enabled,
      acrylicSupported: acrylicSupported.value,
      image: backgroundImage.value,
      opacity: backgroundOpacity.value,
      blur: backgroundBlur.value,
      brightness: backgroundBrightness.value,
      size: backgroundSize.value,
    });
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}

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
  <div class="app-layout">
    <div class="app-background" :style="backgroundStyle"></div>
    <AppSidebar />
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
</template>

<style scoped>
.app-layout {
  position: relative;
  display: flex;
  width: 100vw;
  height: 100vh;
  background-color: var(--sl-bg);
  overflow: hidden;
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
