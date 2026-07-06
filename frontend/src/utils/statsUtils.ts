import { storeToRefs } from "pinia";
import { useStatsStore } from "@stores/statsStore";

const statsStore = useStatsStore();

const {
  systemInfo,
  serverSystemInfo,
  cpuUsage,
  memUsage,
  diskUsage,
  cpuHistory,
  memHistory,
  diskHistory,
  serverCpuUsage,
  serverMemUsage,
  serverDiskUsage,
  serverCpuHistory,
  serverMemHistory,
  statsViewMode,
  statsLoading,
  serverStatsLoading,
  serverStatsError,
} = storeToRefs(statsStore);

const { fetchSystemInfo, fetchServerResourceUsage, resetStatsHistory } = statsStore;

export {
  systemInfo,
  serverSystemInfo,
  cpuUsage,
  memUsage,
  diskUsage,
  cpuHistory,
  memHistory,
  diskHistory,
  serverCpuUsage,
  serverMemUsage,
  serverDiskUsage,
  serverCpuHistory,
  serverMemHistory,
  statsViewMode,
  statsLoading,
  serverStatsLoading,
  serverStatsError,
  fetchSystemInfo,
  fetchServerResourceUsage,
  resetStatsHistory,
};
