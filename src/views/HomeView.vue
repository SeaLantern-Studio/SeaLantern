<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import ErrorBanner from "@components/views/home/ErrorBanner.vue";
import QuickStartCard from "@components/views/home/QuickStartCard.vue";
import SystemStatsCard from "@components/views/home/SystemStatsCard.vue";
import ServerListSection from "@components/views/home/ServerListSection.vue";
import AlertsSection from "@components/views/home/AlertsSection.vue";
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
import { getCurrentWindow } from "@tauri-apps/api/window";

const router = useRouter();
const store = useServerStore();

let statsTimer: ReturnType<typeof setInterval> | null = null;
let refreshTimer: ReturnType<typeof setInterval> | null = null;

async function loadServers() {
  try {
    await store.reloadServers();
    await Promise.all(store.servers.map((s) => store.refreshStatus(s.id)));
  } catch (e) {
    console.error("Failed to load servers:", e);
  }
}

// 窗口获得焦点时重新扫描服务器列表
async function handleFocus() {
  await loadServers();
}

onMounted(() => {
  initQuote();

  loadServers();
  fetchSystemInfo();

  statsTimer = setInterval(fetchSystemInfo, 3000);
  startQuoteTimer();

  refreshTimer = setInterval(async () => {
    await Promise.all(store.servers.map((s) => store.refreshStatus(s.id)));
  }, 3000);

  startThemeObserver();

  // 监听窗口焦点事件
  const appWindow = getCurrentWindow();
  appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      loadServers();
    }
  });
});

onUnmounted(() => {
  if (statsTimer) clearInterval(statsTimer);
  if (refreshTimer) clearInterval(refreshTimer);
  cleanupQuoteResources();
  cleanupStatsResources();
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
