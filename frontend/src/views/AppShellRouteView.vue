<script setup lang="ts">
import { onMounted, ref } from "vue";
import AppLayout from "@components/layout/AppLayout.vue";
import DataDirectorySetupModal from "@components/common/DataDirectorySetupModal.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import TermsDialog from "@components/common/TermsDialog.vue";
import ToastContainer from "@components/common/ToastContainer.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import { PluginComponentRenderer } from "@components/plugin";
import { useAppBootstrap } from "@composables/useAppBootstrap";
import { useGlobalContextMenu } from "@composables/useGlobalContextMenu";
import { useTauriGlobalEvents } from "@composables/useTauriGlobalEvents";

const {
  showSplash,
  isInitializing,
  showTermsDialog,
  showDataDirDialog,
  updateStore,
  dataDirectory,
  initializeApp,
  handleAgreeTerms,
  handleBrowseDataDir,
  handleInitializeDataDir,
  handleSplashReady,
} = useAppBootstrap();

const bootstrapDataDirPath = ref("");

useGlobalContextMenu();
useTauriGlobalEvents();

onMounted(async () => {
  await initializeApp();
  bootstrapDataDirPath.value = dataDirectory.status.value?.recommended_data_dir || "";
});

function handleUpdateModalClose() {
  updateStore.hideUpdateModal();
}

async function handleBootstrapBrowse() {
  const selected = await handleBrowseDataDir();
  if (selected) {
    bootstrapDataDirPath.value = selected;
  }
}

async function handleBootstrapConfirm(path: string) {
  await handleInitializeDataDir(path || bootstrapDataDirPath.value);
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

    <DataDirectorySetupModal
      :visible="showDataDirDialog"
      :recommended-path="
        bootstrapDataDirPath || dataDirectory.status.value?.recommended_data_dir || ''
      "
      :busy="dataDirectory.isBusy.value"
      :error="dataDirectory.error.value"
      @browse="handleBootstrapBrowse"
      @confirm="handleBootstrapConfirm"
      @close="showDataDirDialog = false"
    />

    <PluginComponentRenderer />
    <ToastContainer />
  </template>
</template>
