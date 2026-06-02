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
import { useServerStore } from "@stores/serverStore";
import {
  cleanupQuoteResources,
  initQuote,
  pauseQuoteUpdates,
  resumeQuoteUpdates,
} from "@utils/quoteUtils";
import { fetchSystemInfo } from "@utils/statsUtils";
import { useSerialPolling } from "@composables/useSerialPolling";
import {
  actionError,
  deleteServerName,
  showDeleteConfirm,
  confirmDelete,
  cancelDelete,
  closeDeleteConfirm,
} from "@utils/serverUtils";
import { i18n } from "@language";

const router = useRouter();
const store = useServerStore();

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
</script>

<template>
  <div class="home-view animate-fade-in-up">
    <ErrorBanner :message="actionError" @close="actionError = null" />

    <div class="top-row">
      <QuickStartCard @create="handleCreate" />
      <SystemStatsCard />
    </div>

    <ServerListSection :servers="store.servers" :loading="store.loading" />

    <AlertsSection />

    <ChangePathModal />

    <SLConfirmDialog
      :visible="showDeleteConfirm"
      :title="i18n.t('home.delete_server')"
      :message="
        i18n.t('home.delete_confirm_message', {
          server: '<strong>' + deleteServerName + '</strong>',
        })
      "
      :confirmText="i18n.t('home.delete_confirm')"
      :cancelText="i18n.t('home.delete_cancel')"
      confirmVariant="danger"
      :requireInput="true"
      :inputPlaceholder="i18n.t('home.delete_input_placeholder')"
      :expectedInput="deleteServerName"
      @confirm="confirmDelete"
      @cancel="cancelDelete"
      @close="closeDeleteConfirm"
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
