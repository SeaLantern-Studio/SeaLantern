<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import DataDirectorySetupModal from "@components/common/DataDirectorySetupModal.vue";
import TermsDialog from "@components/common/TermsDialog.vue";
import UpdateModal from "@components/common/UpdateModal.vue";
import SplashScreen from "@components/splash/SplashScreen.vue";
import { useAppBootstrap } from "@composables/useAppBootstrap";

interface Props {
  enabled?: boolean;
  includeUpdateModal?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  enabled: true,
  includeUpdateModal: true,
});

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

const shouldShowSplash = computed(() => props.enabled && showSplash.value);
const shouldShowShellContent = computed(() => !props.enabled || !showSplash.value);
const shouldShowUpdateModal = computed(
  () =>
    props.enabled &&
    props.includeUpdateModal &&
    updateStore.isUpdateModalVisible &&
    updateStore.isUpdateAvailable,
);

onMounted(async () => {
  if (!props.enabled) {
    return;
  }

  await initializeApp();
  bootstrapDataDirPath.value = dataDirectory.status.value?.recommended_data_dir || "";
});

function handleUpdateModalClose(): void {
  updateStore.hideUpdateModal();
}

async function handleBootstrapBrowse(): Promise<void> {
  const selected = await handleBrowseDataDir();
  if (selected) {
    bootstrapDataDirPath.value = selected;
  }
}

async function handleBootstrapConfirm(path: string): Promise<void> {
  await handleInitializeDataDir(path || bootstrapDataDirPath.value);
}
</script>

<template>
  <transition name="splash-fade">
    <SplashScreen v-if="shouldShowSplash" :loading="isInitializing" @ready="handleSplashReady" />
  </transition>

  <template v-if="shouldShowShellContent">
    <slot />

    <UpdateModal v-if="shouldShowUpdateModal" @close="handleUpdateModalClose" />

    <TermsDialog
      v-if="enabled"
      :visible="showTermsDialog"
      @agree="handleAgreeTerms"
      @close="showTermsDialog = false"
    />

    <DataDirectorySetupModal
      v-if="enabled"
      :visible="showDataDirDialog"
      :recommended-path="bootstrapDataDirPath || dataDirectory.status.value?.recommended_data_dir || ''"
      :busy="dataDirectory.isBusy.value"
      :error="dataDirectory.error.value"
      @browse="handleBootstrapBrowse"
      @confirm="handleBootstrapConfirm"
      @close="showDataDirDialog = false"
    />
  </template>
</template>
