<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import AppLayout from "./components/layout/AppLayout.vue";
import SplashScreen from "./components/splash/SplashScreen.vue";
import UpdateModal from "./components/common/UpdateModal.vue";
import { useUpdateStore } from "./stores/updateStore";
import { useSettingsStore } from "./stores/settingsStore";
import { applyTheme, applyFontSize, applyFontFamily } from "./utils/theme";

const showSplash = ref(true);
const isInitializing = ref(true);
const updateStore = useUpdateStore();
const settingsStore = useSettingsStore();

onMounted(async () => {
  try {
    await settingsStore.loadSettings();
    const settings = settingsStore.settings;
    applyTheme(settings.theme || "auto");
    applyFontSize(settings.font_size || 14);
    applyFontFamily(settings.font_family || "");

    try {
      const { setupTray } = await import("./utils/tray");
      if (typeof setupTray === "function") {
        await setupTray();
        console.log("Tray setup completed");
      }
    } catch (trayErr) {
      console.warn("Failed to set up tray, tray functionality will be unavailable:", trayErr);
    }
  } catch (e) {
    console.error("Failed to load settings during startup:", e);
  } finally {
    isInitializing.value = false;
  }
});

onUnmounted(async () => {
  // 注意：通常不需要清理托盘，因为应用关闭时会自动清理
  // 但如果需要手动清理，可以取消注释以下代码
  // try {
  //   const { cleanupTray } = await import("./utils/tray");
  //   await cleanupTray();
  // } catch (e) {
  //   console.warn("Failed to cleanup tray:", e);
  // }
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
  </template>
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
