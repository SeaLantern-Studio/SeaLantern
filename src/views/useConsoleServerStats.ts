import { computed, onUnmounted, type Ref } from "vue";
import { useSerialPolling } from "@composables/useSerialPolling";
import { useServerStore } from "@stores/serverStore";
import {
  fetchServerResourceUsage,
  resetStatsHistory,
  serverCpuUsage,
  serverStatsError,
  serverStatsLoading,
  serverSystemInfo,
} from "@utils/statsUtils";
import { formatBytes } from "@utils/formatters";
import { i18n } from "@language";
import { Cpu, HardDrive, MemoryStick } from "lucide-vue-next";

interface UseConsoleServerStatsOptions {
  serverId: Ref<string>;
}

export function useConsoleServerStats(options: UseConsoleServerStatsOptions) {
  const serverStore = useServerStore();
  const SERVER_STATS_POLL_INTERVAL_MS = 15000;

  const currentServer = computed(
    () => serverStore.servers.find((server) => server.id === options.serverId.value) || null,
  );
  const serverProcessInfo = computed(() => serverSystemInfo.value);
  const serverStatsUnavailable = computed(() => serverStatsError.value && !serverProcessInfo.value);

  const statsSummaryItems = computed(() => [
    {
      key: "cpu",
      icon: Cpu,
      label: i18n.t("home.cpu"),
      value: serverStatsUnavailable.value ? "--" : `${serverCpuUsage.value}%`,
      detail: "",
      tone: "primary",
    },
    {
      key: "memory",
      icon: MemoryStick,
      label: i18n.t("home.memory"),
      value:
        serverProcessInfo.value && currentServer.value
          ? `${formatBytes(serverProcessInfo.value.memory.used)} / ${currentServer.value.max_memory} MB`
          : "--",
      detail: "",
      tone: "success",
    },
    {
      key: "disk",
      icon: HardDrive,
      label: i18n.t("home.disk"),
      value: serverProcessInfo.value ? formatBytes(serverProcessInfo.value.disk.used) : "--",
      detail: "",
      tone: "warning",
    },
  ]);

  async function refreshServerStats() {
    const serverId = options.serverId.value;
    if (!serverId) {
      serverStatsLoading.value = false;
      return;
    }

    await Promise.all([fetchServerResourceUsage(serverId), serverStore.refreshStatus(serverId)]);
  }

  const statsPolling = useSerialPolling({
    intervalMs: SERVER_STATS_POLL_INTERVAL_MS,
    task: async () => {
      await refreshServerStats();
    },
  });

  function startStatsPolling() {
    statsPolling.start();
  }

  function stopStatsPolling() {
    statsPolling.stop();
  }

  function resetServerStats() {
    resetStatsHistory();
    stopStatsPolling();
  }

  onUnmounted(() => {
    stopStatsPolling();
  });

  return {
    currentServer,
    serverProcessInfo,
    serverStatsUnavailable,
    statsSummaryItems,
    refreshServerStats,
    startStatsPolling,
    stopStatsPolling,
    resetServerStats,
  };
}
