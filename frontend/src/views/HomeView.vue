<script setup lang="ts">
import { onActivated, onDeactivated, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import ErrorBanner from "@components/views/home/ErrorBanner.vue";
import QuickStartCard from "@components/views/home/QuickStartCard.vue";
import SystemStatsCard from "@components/views/home/SystemStatsCard.vue";
import ServerListSection from "@components/views/home/ServerListSection.vue";
import AlertsSection from "@components/views/home/AlertsSection.vue";
import ChangePathModal from "@components/views/home/ChangePathModal.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import { useHomeServerActionsStore } from "@stores/homeServerActionsStore";
import { useServerStore } from "@stores/serverStore";
import {
  cleanupQuoteResources,
  initQuote,
  pauseQuoteUpdates,
  resumeQuoteUpdates,
} from "@utils/quoteUtils";
import { fetchSystemInfo } from "@utils/statsUtils";
import { useSerialPolling } from "@composables/useSerialPolling";
import { i18n } from "@language";

const router = useRouter();
const store = useServerStore();
const homeServerActionsStore = useHomeServerActionsStore();

async function loadServers() {
  try {
    await store.refreshList();
    await Promise.all(store.servers.map((server) => store.refreshStatus(server.id)));
  } catch (e) {
    console.error("Failed to load servers:", e);
  }
}

const systemInfoPolling = useSerialPolling({
  intervalMs: 3000,
  task: async () => {
    await fetchSystemInfo();
  },
});

const serverStatusPolling = useSerialPolling({
  intervalMs: 3000,
  task: async () => {
    await Promise.all(store.servers.map((server) => store.refreshStatus(server.id)));
  },
});

function startHomePolling() {
  systemInfoPolling.start();
  serverStatusPolling.start();
}

function stopHomePolling() {
  systemInfoPolling.stop();
  serverStatusPolling.stop();
}

onMounted(() => {
  initQuote();

  void loadServers().finally(() => {
    startHomePolling();
  });
  resumeQuoteUpdates();
});

onActivated(() => {
  startHomePolling();
  resumeQuoteUpdates();
});

onDeactivated(() => {
  stopHomePolling();
  pauseQuoteUpdates();
});

onUnmounted(() => {
  stopHomePolling();
  cleanupQuoteResources();
});

function handleCreate() {
  router.push("/create");
}

function handleAddExisting() {
  router.push("/add-existing");
}
</script>

<template>
  <div class="home-view animate-fade-in-up">
    <ErrorBanner
      :message="homeServerActionsStore.actionError"
      @close="homeServerActionsStore.actionError = null"
    />

    <div class="top-row">
      <QuickStartCard @create="handleCreate" @add-existing="handleAddExisting" />
      <SystemStatsCard />
    </div>

    <ServerListSection :servers="store.servers" :loading="store.loading" />

    <AlertsSection />

    <ChangePathModal />

    <SLConfirmDialog
      :visible="homeServerActionsStore.showDeleteConfirm"
      :title="i18n.t('home.delete_server')"
      :message="
        i18n.t('home.delete_confirm_message', {
          server: '<strong>' + homeServerActionsStore.deleteServerName + '</strong>',
        })
      "
      :confirmText="i18n.t('home.delete_confirm')"
      :cancelText="i18n.t('home.delete_cancel')"
      confirmVariant="danger"
      :requireInput="true"
      :inputPlaceholder="i18n.t('home.delete_input_placeholder')"
      :expectedInput="homeServerActionsStore.deleteServerName"
      @confirm="homeServerActionsStore.confirmDelete"
      @cancel="homeServerActionsStore.cancelDelete"
      @close="homeServerActionsStore.closeDeleteConfirm"
      dangerous
    />
  </div>
</template>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-md);
}

.top-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sl-space-md);
}

@media (max-width: 900px) {
  .top-row {
    grid-template-columns: 1fr;
  }
}
</style>
