<script setup lang="ts">
import { onMounted, onUnmounted, computed } from "vue";
import AppSidebar from "@components/layout/AppSidebar.vue";
import AppHeader from "@components/layout/AppHeader.vue";
import {
  useSettingsStore,
  SETTINGS_UPDATE_EVENT,
  type SettingsUpdateEvent,
} from "@stores/settingsStore";
import { applyDeveloperMode } from "@utils/theme";
import { enqueueAppearanceApply } from "@utils/appearance";
import { isMacOSPlatform } from "@utils/platform";

const settingsStore = useSettingsStore();

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);
const isMacOS = isMacOSPlatform();

let systemThemeQuery: MediaQueryList | null = null;

function handleSystemThemeChange(): void {
  if (settingsStore.settings.theme === "auto") {
    void enqueueAppearanceApply(settingsStore.settings);
  }
}

function applyDeveloperSettings(): void {
  applyDeveloperMode(settingsStore.settings.developer_mode || false);
}

async function applyAllSettings(): Promise<void> {
  await enqueueAppearanceApply(settingsStore.settings);
  applyDeveloperSettings();
}

function handleSettingsUpdateEvent(e: CustomEvent<SettingsUpdateEvent>): void {
  const { changedGroups, settings } = e.detail;
  settingsStore.settings = settings;

  if (changedGroups.includes("Appearance")) {
    void enqueueAppearanceApply(settings);
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
  const style: Record<string, string | number> = {
    backgroundImage: `url(${backgroundImage.value})`,
    backgroundSize: backgroundSize.value,
    backgroundPosition: "center",
    backgroundRepeat: "no-repeat",
    opacity: backgroundOpacity.value,
  };
  // blur=0 且 brightness=1 时不挂 filter,避免无意义的全屏重采样层
  if (backgroundBlur.value > 0 || backgroundBrightness.value !== 1) {
    style.filter = `blur(${backgroundBlur.value}px) brightness(${backgroundBrightness.value})`;
  }
  return style;
});
</script>

<template>
  <div class="app-layout" :class="{ 'macos-native-vibrancy': isMacOS }">
    <div class="app-background" :style="backgroundStyle"></div>
    <AppSidebar />
    <div class="app-main" :class="{ 'macos-native-vibrancy': isMacOS }">
      <AppHeader />
      <main class="app-content">
        <router-view v-slot="{ Component }">
          <keep-alive :max="5">
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </main>
    </div>
  </div>
</template>

<style src="@styles/components/layout/AppLayout.css" scoped></style>
