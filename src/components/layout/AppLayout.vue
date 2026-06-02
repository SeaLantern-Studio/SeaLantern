<script setup lang="ts">
import { onMounted, onUnmounted, computed, watch } from "vue";
import AppSidebar from "@components/layout/AppSidebar.vue";
import AppHeader from "@components/layout/AppHeader.vue";
import type { WindowEffect } from "@api/settings";
import { useUiStore } from "@stores/uiStore";
import { useSettingsStore } from "@stores/settingsStore";

const ui = useUiStore();
const settingsStore = useSettingsStore();

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);
let systemThemeQuery: MediaQueryList | null = null;

function handleSystemThemeChange(): void {
  const settings = settingsStore.settings;
  if (settings.theme === "auto") {
    void settingsStore.queueClientSettingsApply(settings);
  }
}

async function applyAllSettings(): Promise<void> {
  await settingsStore.queueClientSettingsApply(settingsStore.settings);
}

onMounted(async () => {
  await settingsStore.ensureLoaded();
  await applyAllSettings();

  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", handleSystemThemeChange);
});

onUnmounted(() => {
  if (systemThemeQuery) {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  }
});

watch(
  () => settingsStore.settings,
  (nextSettings, previousSettings) => {
    if (nextSettings === previousSettings) {
      return;
    }
    void settingsStore.queueClientSettingsApply(nextSettings);
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

const hasTransparentWindowBackdrop = computed(
  () => (settingsStore.windowEffect as WindowEffect) !== "off" || !backgroundImage.value,
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
