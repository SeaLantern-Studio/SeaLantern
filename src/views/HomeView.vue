<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import ErrorBanner from "@components/views/home/ErrorBanner.vue";
import QuickStartCard from "@components/views/home/QuickStartCard.vue";
import SystemStatsCard from "@components/views/home/SystemStatsCard.vue";
import ServerListSection from "@components/views/home/ServerListSection.vue";
import AlertsSection from "@components/views/home/AlertsSection.vue";
import ChangePathModal from "@components/views/home/ChangePathModal.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import { useServerStore } from "@stores/serverStore";
import { initQuote, startQuoteTimer, cleanupQuoteResources } from "@utils/quoteUtils";
import { fetchSystemInfo, startThemeObserver, cleanupStatsResources } from "@utils/statsUtils";
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

let statsTimer: ReturnType<typeof setInterval> | null = null;
let refreshTimer: ReturnType<typeof setInterval> | null = null;
// 页面隐藏时暂停轮询,避免后台无意义 CPU/IO 开销
let isPageVisible = true;

const refreshAllStatuses = async () => {
  await Promise.all(store.servers.map((s) => store.refreshStatus(s.id)));
};

const startTimers = () => {
  stopTimers();
  // 拉长到 5 秒,降低无谓刷新带来的 UI 卡顿
  statsTimer = setInterval(fetchSystemInfo, 5000);
  refreshTimer = setInterval(refreshAllStatuses, 5000);
};

const stopTimers = () => {
  if (statsTimer) {
    clearInterval(statsTimer);
    statsTimer = null;
  }
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
};

const handleVisibilityChange = () => {
  const visible = document.visibilityState === "visible";
  if (visible === isPageVisible) return;
  isPageVisible = visible;
  if (visible) {
    // 重新可见时立即刷新一次并重启轮询
    fetchSystemInfo();
    refreshAllStatuses();
    startTimers();
  } else {
    stopTimers();
  }
};

onMounted(() => {
  initQuote();

  const loadServers = async () => {
    try {
      await store.refreshList();
      await refreshAllStatuses();
    } catch (e) {
      console.error("Failed to load servers:", e);
    }
  };

  loadServers();
  fetchSystemInfo();
  startTimers();
  startQuoteTimer();
  startThemeObserver();
  document.addEventListener("visibilitychange", handleVisibilityChange);
});

onUnmounted(() => {
  stopTimers();
  cleanupQuoteResources();
  cleanupStatsResources();
  document.removeEventListener("visibilitychange", handleVisibilityChange);
});

function handleCreate() {
  router.push("/create");
}
</script>

<template>
  <div class="home-view animate-stagger-in">
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
