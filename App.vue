<script setup lang="ts">
import { ref, onMounted } from "vue";
import AppLayout from "./components/layout/AppLayout.vue";
import SplashScreen from "./components/splash/SplashScreen.vue";
import { settingsApi } from "./api/settings";
import { useUiStore } from "./stores/uiStore";
import type { ColorScheme, ColorMode } from "./types/common";

const uiStore = useUiStore();
const showSplash = ref(true);

async function initTheme() {
  try {
    const settings = await settingsApi.get();
    uiStore.initTheme(
      settings.color_scheme as ColorScheme,
      settings.color_mode as ColorMode,
      settings.custom_color_scheme
    );
  } catch (e) {
    console.error("Failed to init theme:", e);
  }
}

function handleSplashReady() {
  showSplash.value = false;
}

onMounted(() => {
  initTheme();
});
</script>

<template>
  <transition name="splash-fade">
    <SplashScreen v-if="showSplash" @ready="handleSplashReady" />
  </transition>
  <AppLayout v-if="!showSplash" />
</template>

<style>
#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.splash-fade-leave-active {
  transition: opacity 0.3s ease;
}

.splash-fade-leave-to {
  opacity: 0;
}
</style>
