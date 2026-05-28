<script setup lang="ts">
import { onMounted, onUnmounted, computed } from "vue";
import AppSidebar from "@components/layout/AppSidebar.vue";
import AppHeader from "@components/layout/AppHeader.vue";
import { settingsApi, type WindowEffect } from "@api/settings";
import { useUiStore } from "@stores/uiStore";
import {
  useSettingsStore,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";
import {
  applyTheme,
  applyFontFamily,
  applyFontSize,
  applyColors,
  applyWindowTitle,
  applyDeveloperMode,
  isThemeProviderActive,
  getEffectiveTheme,
} from "@utils/theme";
import { isWindowsPlatform } from "@utils/platform";

const ui = useUiStore();
const settingsStore = useSettingsStore();

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);
const isWindows = isWindowsPlatform();

let systemThemeQuery: MediaQueryList | null = null;
let lastNativeEffectKey: string | null = null;
let appearanceApplyQueue: Promise<void> = Promise.resolve();

function applyWindowEffectAttributes(effect: WindowEffect): void {
  const normalized = effect || "off";
  const enabled = normalized !== "off" || !backgroundImage.value;
  document.documentElement.setAttribute("data-acrylic", enabled ? "true" : "false");
  document.documentElement.setAttribute("data-window-effect", normalized);
}

function handleSystemThemeChange(): void {
  const settings = settingsStore.settings;
  if (settings.theme === "auto") {
    applyTheme("auto");
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
  await applyWindowTitle(settings);

  const effect = (settings.window_effect || "off") as WindowEffect;
  const dark = getEffectiveTheme(settings.theme || "auto") === "dark";

  applyWindowEffectAttributes(effect);
  if (isWindows) {
    const nativeEffectKey = `${effect}:${dark}`;
    if (lastNativeEffectKey !== nativeEffectKey) {
      lastNativeEffectKey = nativeEffectKey;
      await settingsApi.applyWindowEffect(effect, dark);
    }
  }

  if (!isThemeProviderActive()) {
    applyColors(settings);
  }
}

function enqueueAppearanceApply(): Promise<void> {
  appearanceApplyQueue = appearanceApplyQueue.then(
    () => applyAppearanceSettings(),
    () => applyAppearanceSettings(),
  );
  return appearanceApplyQueue;
}

function applyDeveloperSettings(): void {
  applyDeveloperMode(settingsStore.settings.developer_mode || false);
}

async function applyAllSettings(): Promise<void> {
  await enqueueAppearanceApply();
  applyDeveloperSettings();
}

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>): void {
  const { changedGroups, settings } = e.detail;
  settingsStore.settings = settings;

  if (changedGroups.includes("Appearance")) {
    void enqueueAppearanceApply();
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

const hasTransparentWindowBackdrop = computed(
  () => settingsStore.windowEffect !== "off" || !backgroundImage.value,
);

const layoutClasses = computed(() => ({
  "has-native-window-effect": hasTransparentWindowBackdrop.value,
}));

const mainClasses = computed(() => ({
  "has-native-window-effect": hasTransparentWindowBackdrop.value,
  "sidebar-collapsed": ui.sidebarCollapsed,
}));
</script>

<template>
  <div class="app-layout" :class="layoutClasses">
    <div class="app-background" :style="backgroundStyle"></div>
    <AppSidebar />
    <div class="app-main" :class="mainClasses">
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

<style src="@styles/components/layout/AppLayout.css" scoped></style>
