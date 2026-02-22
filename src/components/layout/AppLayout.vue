<script setup lang="ts">
import { onMounted, onUnmounted, computed, watch } from "vue";
import AppSidebar from "@components/layout/AppSidebar.vue";
import AppHeader from "@components/layout/AppHeader.vue";
import { useUiStore } from "@stores/uiStore";
import {
  useSettingsStore,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";
import { applyAcrylic, type SettingsGroup } from "@api/settings";
import {
  applyTheme,
  applyFontFamily,
  applyFontSize,
  applyColors,
  applyDeveloperMode,
  getEffectiveTheme,
  isThemeProviderActive,
} from "@utils/theme";

const ui = useUiStore();
const settingsStore = useSettingsStore();

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);

let systemThemeQuery: MediaQueryList | null = null;

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

function handleSystemThemeChange(): void {
  const settings = settingsStore.settings;
  if (settings.theme === "auto") {
    applyTheme("auto");
    if (settings.acrylic_enabled && settingsStore.acrylicSupported) {
      applyAcrylicEffect(true, "auto");
    }
    if (!isThemeProviderActive()) {
      applyColors(settings);
    }
  }
}

async function applyAppearanceSettings(): Promise<void> {
  const settings = settingsStore.settings;

  applyTheme(settings.theme || "auto");
  applyFontSize(settings.font_size || 14);
  applyFontFamily(settings.font_family || "");

  if (settingsStore.acrylicSupported) {
    await applyAcrylicEffect(settings.acrylic_enabled, settings.theme || "auto");
  } else {
    document.documentElement.setAttribute("data-acrylic", "false");
  }

  if (!isThemeProviderActive()) {
    applyColors(settings);
  }
}

function applyDeveloperSettings(): void {
  applyDeveloperMode(settingsStore.settings.developer_mode || false);
}

async function applyAllSettings(): Promise<void> {
  await applyAppearanceSettings();
  applyDeveloperSettings();
}

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>): void {
  const { changedGroups, settings } = e.detail;
  settingsStore.settings = settings;

  if (changedGroups.includes("Appearance")) {
    applyAppearanceSettings();
  }
  if (changedGroups.includes("Developer")) {
    applyDeveloperSettings();
  }
}

onMounted(async () => {
  await applyAllSettings();

  window.addEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);

  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", handleSystemThemeChange);
});

onUnmounted(() => {
  window.removeEventListener(SETTINGS_UPDATE_EVENT, handleSettingsUpdateEvent as EventListener);
  if (systemThemeQuery) {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  }
});

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
