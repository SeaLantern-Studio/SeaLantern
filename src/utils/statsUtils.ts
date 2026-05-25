import { ref } from "vue";
import { systemApi, type SystemInfo, type ServerResourceUsage } from "@api/system";

const systemInfo = ref<SystemInfo | null>(null);
const serverSystemInfo = ref<ServerResourceUsage | null>(null);
const cpuUsage = ref(0);
const memUsage = ref(0);
const diskUsage = ref(0);
const cpuHistory = ref<number[]>([]);
const memHistory = ref<number[]>([]);
const serverCpuUsage = ref(0);
const serverMemUsage = ref(0);
const serverDiskUsage = ref(0);
const serverCpuHistory = ref<number[]>([]);
const serverMemHistory = ref<number[]>([]);
const statsViewMode = ref<"detail" | "gauge">("gauge");
const statsLoading = ref(true);
const serverStatsLoading = ref(true);
const serverStatsError = ref(false);

function pushHistory(target: number[], value: number) {
  target.push(value);
  if (target.length > 30) target.shift();
}

function applySystemStatsInfo(info: SystemInfo) {
  systemInfo.value = info;
  cpuUsage.value = Math.min(100, Math.max(0, Math.round(info.cpu.usage)));
  memUsage.value = Math.min(100, Math.max(0, Math.round(info.memory.usage)));
  diskUsage.value = Math.min(100, Math.max(0, Math.round(info.disk.usage)));
  pushHistory(cpuHistory.value, cpuUsage.value);
  pushHistory(memHistory.value, memUsage.value);
  statsLoading.value = false;
}

function applyServerStatsInfo(info: ServerResourceUsage) {
  serverSystemInfo.value = info;
  serverCpuUsage.value = Math.min(100, Math.max(0, Math.round(info.cpu.usage)));
  serverMemUsage.value = Math.min(100, Math.max(0, Math.round(info.memory.usage)));
  serverDiskUsage.value = Math.min(100, Math.max(0, Math.round(info.disk.usage)));
  pushHistory(serverCpuHistory.value, serverCpuUsage.value);
  pushHistory(serverMemHistory.value, serverMemUsage.value);
  serverStatsError.value = false;
  serverStatsLoading.value = false;
}

async function fetchSystemInfo() {
  try {
    const info = await systemApi.getSystemInfo();
    applySystemStatsInfo(info);
  } catch (e) {
    console.error("Failed to fetch system info:", e);
    statsLoading.value = false;
  }
}

async function fetchServerResourceUsage(serverId: string) {
  try {
    const info = await systemApi.getServerResourceUsage(serverId);
    applyServerStatsInfo(info);
  } catch (e) {
    console.error("Failed to fetch server resource usage:", e);
    serverStatsError.value = true;
    serverStatsLoading.value = false;
  }
}

function resetStatsHistory() {
  serverCpuHistory.value = [];
  serverMemHistory.value = [];
}

export {
  systemInfo,
  serverSystemInfo,
  cpuUsage,
  memUsage,
  diskUsage,
  cpuHistory,
  memHistory,
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
