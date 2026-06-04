<script setup lang="ts">
import { computed } from "vue";
import AppSidebar from "@components/layout/AppSidebar.vue";
import AppHeader from "@components/layout/AppHeader.vue";
import type { WindowEffect } from "@api/settings";
import { useClientSettingsSync } from "@composables/useClientSettingsSync";
import { useUiStore } from "@stores/uiStore";
import { useSettingsStore } from "@stores/settingsStore";

const ui = useUiStore();
const settingsStore = useSettingsStore();

useClientSettingsSync();

const backgroundImage = computed(() => settingsStore.backgroundImage);
const backgroundOpacity = computed(() => settingsStore.backgroundOpacity);
const backgroundBlur = computed(() => settingsStore.backgroundBlur);
const backgroundBrightness = computed(() => settingsStore.backgroundBrightness);
const backgroundSize = computed(() => settingsStore.backgroundSize);

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
