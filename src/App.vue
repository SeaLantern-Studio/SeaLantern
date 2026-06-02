<script setup lang="ts">
import { onMounted } from "vue";
import AppLayout from "@components/layout/AppLayout.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import TermsDialog from "@components/common/TermsDialog.vue";
import SLContextMenu from "@components/common/SLContextMenu.vue";
import ToastContainer from "@components/common/ToastContainer.vue";
import { PluginComponentRenderer } from "@components/plugin";
import { useAppBootstrap } from "@composables/useAppBootstrap";
import { useGlobalContextMenu } from "@composables/useGlobalContextMenu";
import { useTauriGlobalEvents } from "@composables/useTauriGlobalEvents";

const {
  showSplash,
  isInitializing,
  showTermsDialog,
  updateStore,
  initializeApp,
  handleAgreeTerms,
  handleSplashReady,
} = useAppBootstrap();

useGlobalContextMenu();
useTauriGlobalEvents();

onMounted(async () => {
  await initializeApp();
});

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

    <TermsDialog
      :visible="showTermsDialog"
      @agree="handleAgreeTerms"
      @close="showTermsDialog = false"
    />

    <PluginComponentRenderer />
    <ToastContainer />
  </template>
  <SLContextMenu />
</template>

<style src="@styles/app.css"></style>
