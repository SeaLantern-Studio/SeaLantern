<script setup lang="ts">
import { ref } from "vue";
import AppLayout from "./components/layout/AppLayout.vue";
import SplashScreen from "./components/splash/SplashScreen.vue";
import UpdateModal from "./components/common/UpdateModal.vue";
import { useUpdateStore } from "./stores/updateStore";

const showSplash = ref(true);
const updateStore = useUpdateStore();

function handleSplashReady() {
  showSplash.value = false;
  updateStore.checkForUpdateOnStartup();
}

function handleUpdateModalClose() {
  updateStore.hideUpdateModal();
}
</script>

<template>
  <transition name="splash-fade">
    <SplashScreen v-if="showSplash" @ready="handleSplashReady" />
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
