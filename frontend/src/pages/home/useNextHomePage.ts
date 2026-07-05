import { computed, onMounted, shallowRef } from "vue";
import { useRouter } from "vue-router";
import { i18n } from "@language";
import {
  NEXT_SERVER_CREATE_ROUTE_NAME,
  NEXT_SERVER_IMPORT_ROUTE_NAME,
} from "@src/router/pageMeta";
import { useSerialPolling } from "@src/composables/useSerialPolling";
import { useHomeServerActionsStore } from "@stores/homeServerActionsStore";
import { useServerStore } from "@stores/serverStore";
import { useStatsStore } from "@stores/statsStore";
import { formatBytes } from "@utils/formatters";
import { formatServerCoreTypeLabel } from "@utils/serverCoreLabel";
import { isActiveServerStatus } from "@utils/serverStatus";
import type { ServerStatusInfo } from "@api/server";
import type { ServerInstance } from "@type/server";
import { NEXT_HOME_CARD_LAYOUTS } from "./layoutContract";

export interface NextHomePageSummaryMetric {
  id: string;
  label: string;
  value: string;
  meta: string;
  tone?: "neutral" | "primary" | "success" | "warning" | "danger";
}

export interface NextHomeSystemMetric {
  id: string;
  label: string;
  value: string;
  detail: string;
  percent: number;
  history: number[];
  tone: "primary" | "success" | "warning";
}

export interface NextHomeServerCardModel {
  id: string;
  name: string;
  statusText: string;
  statusTone: "success" | "warning" | "danger" | "neutral";
  status: ServerStatusInfo["status"] | undefined;
  runtimeLabel: string;
  coreLabel: string;
  portLabel: string;
  memoryLabel: string;
  pathLabel: string;
  detail: string;
  canStart: boolean;
  isBusy: boolean;
  actionLabel: string;
}

interface UseNextHomePageOptions {
  previewMode: boolean;
  sidebarEntryCount: number;
}

const PREVIEW_ALERTS = [
  { server: "Paper Lobby", line: "[ERROR] Failed to bind to 25566, port already in use" },
  { server: "Modpack QA", line: "[WARN] Startup scan fallback to bundled candidate list" },
];

function pushPreviewHistory(target: number[], value: number) {
  target.push(value);
  if (target.length > 30) {
    target.shift();
  }
}

function formatRuntimeLabel(server: ServerInstance): string {
  const runtimeLabel =
    server.runtime_kind === "docker_itzg"
      ? i18n.t("shell.home_runtime_docker")
      : i18n.t("shell.home_runtime_local");
  const coreLabel = formatServerCoreTypeLabel(server.core_type);
  return i18n.t("shell.home_runtime_with_core", { runtime: runtimeLabel, core: coreLabel });
}

function getStatusTone(
  status: ServerStatusInfo["status"] | undefined,
): NextHomeServerCardModel["statusTone"] {
  if (status === "Running") return "success";
  if (status === "Starting" || status === "Stopping") return "warning";
  if (status === "Error") return "danger";
  return "neutral";
}

function formatUptime(uptime: number | null | undefined): string {
  if (!uptime || uptime <= 0) return i18n.t("shell.home_uptime_idle");
  const totalMinutes = Math.floor(uptime / 60);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours > 0) {
    return i18n.t("shell.home_uptime_hours_minutes", { hours, minutes });
  }
  return i18n.t("shell.home_uptime_minutes", { minutes: Math.max(minutes, 1) });
}

function getDetailText(status: ServerStatusInfo | undefined): string {
  if (status?.display_message) return status.display_message;
  if (status?.status === "Running") return formatUptime(status.uptime);
  return i18n.t("shell.home_detail_waiting");
}

function buildPreviewServers(): NextHomeServerCardModel[] {
  return [
    {
      id: "preview-running",
      name: "Survival Fabric",
      statusText: i18n.t("shell.home_preview_status_running"),
      statusTone: "success",
      status: "Running",
      runtimeLabel: i18n.t("shell.home_runtime_with_core", {
        runtime: i18n.t("shell.home_runtime_local"),
        core: "Fabric",
      }),
      coreLabel: "Fabric",
      portLabel: "25565",
      memoryLabel: "8 GB",
      pathLabel: "servers/survival-fabric",
      detail: i18n.t("shell.home_preview_detail_running"),
      canStart: false,
      isBusy: false,
      actionLabel: i18n.t("home.stop"),
    },
    {
      id: "preview-starting",
      name: "Modpack QA",
      statusText: i18n.t("shell.home_preview_status_starting"),
      statusTone: "warning",
      status: "Starting",
      runtimeLabel: i18n.t("shell.home_runtime_with_core", {
        runtime: i18n.t("shell.home_runtime_docker"),
        core: "itzg",
      }),
      coreLabel: "itzg",
      portLabel: "25575",
      memoryLabel: "6 GB",
      pathLabel: "docker/modpack-qa",
      detail: i18n.t("shell.home_preview_detail_starting"),
      canStart: false,
      isBusy: true,
      actionLabel: i18n.t("shell.home_action_processing"),
    },
    {
      id: "preview-error",
      name: "Paper Lobby",
      statusText: i18n.t("shell.home_preview_status_error"),
      statusTone: "danger",
      status: "Error",
      runtimeLabel: i18n.t("shell.home_runtime_with_core", {
        runtime: i18n.t("shell.home_runtime_local"),
        core: "Paper",
      }),
      coreLabel: "Paper",
      portLabel: "25566",
      memoryLabel: "4 GB",
      pathLabel: "servers/paper-lobby",
      detail: i18n.t("shell.home_preview_detail_error"),
      canStart: true,
      isBusy: false,
      actionLabel: i18n.t("home.start"),
    },
  ];
}

export function useNextHomePage(options: UseNextHomePageOptions) {
  const router = useRouter();
  const serverStore = useServerStore();
  const statsStore = useStatsStore();
  const homeActionsStore = useHomeServerActionsStore();

  const isInitialLoading = shallowRef(!options.previewMode);
  const isRefreshing = shallowRef(false);
  const lastUpdatedAt = shallowRef<number | null>(null);
  const loadFailed = shallowRef(false);
  const statsViewMode = computed(() => statsStore.statsViewMode);
  const previewTick = shallowRef(0);
  const previewCpuUsage = shallowRef(34);
  const previewMemUsage = shallowRef(58);
  const previewDiskUsage = shallowRef(41);
  const previewCpuHistory = shallowRef<number[]>([24, 26, 29, 31, 34]);
  const previewMemHistory = shallowRef<number[]>([48, 50, 53, 56, 58]);
  const previewDiskHistory = shallowRef<number[]>([38, 39, 39, 40, 41]);

  const isPreviewDataset = computed(
    () => options.previewMode && loadFailed.value && serverStore.servers.length === 0,
  );

  const liveStatuses = computed(() => serverStore.statuses);
  const liveServers = computed(() => serverStore.servers);
  const previewServers = computed(() => buildPreviewServers());
  const cardLayouts = NEXT_HOME_CARD_LAYOUTS;

  function advancePreviewMetrics(): void {
    previewTick.value += 1;

    const cpuSamples = [28, 31, 34, 37, 35, 32, 30, 33];
    const memSamples = [54, 55, 57, 58, 59, 58, 57, 56];
    const diskSamples = [39, 39, 40, 41, 41, 42, 41, 41];

    previewCpuUsage.value = cpuSamples[previewTick.value % cpuSamples.length];
    previewMemUsage.value = memSamples[previewTick.value % memSamples.length];
    previewDiskUsage.value = diskSamples[previewTick.value % diskSamples.length];

    pushPreviewHistory(previewCpuHistory.value, previewCpuUsage.value);
    pushPreviewHistory(previewMemHistory.value, previewMemUsage.value);
    pushPreviewHistory(previewDiskHistory.value, previewDiskUsage.value);
  }

  const runningServerCount = computed(() => {
    if (isPreviewDataset.value) {
      return previewServers.value.filter((server) => server.status === "Running").length;
    }
    return liveServers.value.filter((server) => {
      const status = liveStatuses.value[server.id]?.status;
      return status ? isActiveServerStatus(status) : false;
    }).length;
  });

  const issueServerCount = computed(() => {
    if (isPreviewDataset.value) {
      return previewServers.value.filter((server) => server.status === "Error").length;
    }
    return liveServers.value.filter((server) => {
      const status = liveStatuses.value[server.id]?.status;
      return status === "Error" || status === "Stopping";
    }).length;
  });

  const summaryMetrics = computed<NextHomePageSummaryMetric[]>(() => {
    const cpuValue = isPreviewDataset.value ? previewCpuUsage.value : statsStore.cpuUsage;
    const memValue = isPreviewDataset.value ? previewMemUsage.value : statsStore.memUsage;
    const diskValue = isPreviewDataset.value ? previewDiskUsage.value : statsStore.diskUsage;
    return [
      {
        id: "running",
        label: i18n.t("shell.home_summary_running_label"),
        value: String(runningServerCount.value),
        meta: isPreviewDataset.value
          ? i18n.t("shell.home_summary_running_meta_preview")
          : i18n.t("shell.home_summary_running_meta_live"),
        tone: "success",
      },
      {
        id: "issues",
        label: i18n.t("shell.home_summary_issues_label"),
        value: String(issueServerCount.value),
        meta:
          issueServerCount.value > 0
            ? i18n.t("shell.home_summary_issues_meta_blocking")
            : i18n.t("shell.home_summary_issues_meta_clear"),
        tone: issueServerCount.value > 0 ? "danger" : "neutral",
      },
      {
        id: "plugins",
        label: i18n.t("shell.home_summary_plugins_label"),
        value: String(options.sidebarEntryCount),
        meta:
          options.sidebarEntryCount > 0
            ? i18n.t("shell.home_summary_plugins_meta_connected")
            : i18n.t("shell.home_summary_plugins_meta_empty"),
        tone: "primary",
      },
      {
        id: "host",
        label: i18n.t("shell.home_summary_host_label"),
        value: `${cpuValue}% / ${memValue}% / ${diskValue}%`,
        meta: i18n.t("shell.home_summary_host_meta"),
        tone: cpuValue >= 85 || memValue >= 85 ? "warning" : "neutral",
      },
    ];
  });

  const systemMetrics = computed<NextHomeSystemMetric[]>(() => {
    if (isPreviewDataset.value || !statsStore.systemInfo) {
      return [
        {
          id: "cpu",
          label: i18n.t("shell.home_system_cpu"),
          value: `${previewCpuUsage.value}%`,
          detail: i18n.t("shell.home_system_preview_cpu_detail"),
          percent: previewCpuUsage.value,
          history: previewCpuHistory.value,
          tone: "primary",
        },
        {
          id: "memory",
          label: i18n.t("shell.home_system_memory"),
          value: `${previewMemUsage.value}%`,
          detail: i18n.t("shell.home_system_preview_memory_detail"),
          percent: previewMemUsage.value,
          history: previewMemHistory.value,
          tone: "success",
        },
        {
          id: "disk",
          label: i18n.t("shell.home_system_disk"),
          value: `${previewDiskUsage.value}%`,
          detail: i18n.t("shell.home_system_preview_disk_detail"),
          percent: previewDiskUsage.value,
          history: previewDiskHistory.value,
          tone: "warning",
        },
      ];
    }
    return [
      {
        id: "cpu",
        label: i18n.t("shell.home_system_cpu"),
        value: `${statsStore.cpuUsage}%`,
        detail: i18n.t("shell.home_system_live_cpu_detail", {
          count: statsStore.systemInfo.cpu.count,
          name: statsStore.systemInfo.cpu.name,
        }),
        percent: statsStore.cpuUsage,
        history: statsStore.cpuHistory.length > 0 ? statsStore.cpuHistory : [statsStore.cpuUsage],
        tone: "primary",
      },
      {
        id: "memory",
        label: i18n.t("shell.home_system_memory"),
        value: `${statsStore.memUsage}%`,
        detail: `${formatBytes(statsStore.systemInfo.memory.used)} / ${formatBytes(statsStore.systemInfo.memory.total)}`,
        percent: statsStore.memUsage,
        history: statsStore.memHistory.length > 0 ? statsStore.memHistory : [statsStore.memUsage],
        tone: "success",
      },
      {
        id: "disk",
        label: i18n.t("shell.home_system_disk"),
        value: `${statsStore.diskUsage}%`,
        detail: `${formatBytes(statsStore.systemInfo.disk.used)} / ${formatBytes(statsStore.systemInfo.disk.total)}`,
        percent: statsStore.diskUsage,
        history:
          statsStore.diskHistory.length > 0 ? statsStore.diskHistory : [statsStore.diskUsage],
        tone: "warning",
      },
    ];
  });

  const cpuMetric = computed<NextHomeSystemMetric | null>((): NextHomeSystemMetric | null =>
    systemMetrics.value.find((metric) => metric.id === "cpu") ?? null,
  );

  const memoryMetric = computed<NextHomeSystemMetric | null>((): NextHomeSystemMetric | null =>
    systemMetrics.value.find((metric) => metric.id === "memory") ?? null,
  );

  const instanceCountMetric = computed<NextHomePageSummaryMetric | null>(() => ({
    id: "instances",
    label: i18n.t("shell.home_instance_count_label"),
    value: String(totalServerCount.value),
    meta:
      runningServerCount.value > 0
        ? i18n.t("shell.home_instance_count_meta_running", { count: runningServerCount.value })
        : i18n.t("shell.home_instance_count_meta_idle"),
    tone: runningServerCount.value > 0 ? "primary" : "neutral",
  }));

  function toggleStatsViewMode(): void {
    statsStore.statsViewMode = statsStore.statsViewMode === "gauge" ? "detail" : "gauge";
  }

  const serverCards = computed<NextHomeServerCardModel[]>(() => {
    if (isPreviewDataset.value) return previewServers.value;
    return liveServers.value.map((server) => {
      const statusInfo = liveStatuses.value[server.id];
      const canStart =
        statusInfo?.status === "Stopped" || statusInfo?.status === "Error" || !statusInfo?.status;
      return {
        id: server.id,
        name: server.name,
        statusText: homeActionsStore.getStatusText(statusInfo?.status),
        statusTone: getStatusTone(statusInfo?.status),
        status: statusInfo?.status,
        runtimeLabel: formatRuntimeLabel(server),
        coreLabel: formatServerCoreTypeLabel(server.core_type),
        portLabel: String(server.port),
        memoryLabel: `${Math.max(1, Math.round(server.max_memory / 1024))} GB`,
        pathLabel: server.path.replace(/\\/g, "/").split("/").slice(-2).join("/"),
        detail: getDetailText(statusInfo),
        canStart,
        isBusy: !!homeActionsStore.actionLoading[server.id],
        actionLabel: canStart ? i18n.t("home.start") : i18n.t("home.stop"),
      };
    });
  });

  const featuredServer = computed<NextHomeServerCardModel | null>(() => {
    if (serverCards.value.length === 0) return null;
    return (
      serverCards.value.find((server) => server.status === "Error") ||
      serverCards.value.find((server) => server.status === "Running") ||
      serverCards.value[0]
    );
  });

  const secondaryServers = computed(() => {
    if (!featuredServer.value) return serverCards.value;
    return serverCards.value.filter((server) => server.id !== featuredServer.value?.id);
  });

  const alertItems = computed(() =>
    isPreviewDataset.value ? PREVIEW_ALERTS : homeActionsStore.recentAlerts,
  );

  const totalServerCount = computed(() =>
    isPreviewDataset.value ? previewServers.value.length : liveServers.value.length,
  );
  const usingPreviewFallback = computed(() => isPreviewDataset.value);

  const lastUpdatedLabel = computed(() => {
    if (isPreviewDataset.value) return "";
    if (!lastUpdatedAt.value) return i18n.t("shell.home_last_updated_waiting");
    return new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    }).format(lastUpdatedAt.value);
  });

  async function runOverviewLoad(manual = false): Promise<void> {
    if (manual) isRefreshing.value = true;
    try {
      await serverStore.refreshList();
      await Promise.all([serverStore.refreshAllStatuses(), statsStore.fetchSystemInfo()]);
      loadFailed.value = false;
      lastUpdatedAt.value = Date.now();
    } catch (error) {
      console.warn("[next-home] Failed to load overview:", error);
      if (!options.previewMode) throw error;
      loadFailed.value = true;
      advancePreviewMetrics();
      lastUpdatedAt.value = Date.now();
    } finally {
      isInitialLoading.value = false;
      if (manual) isRefreshing.value = false;
    }
  }

  const polling = useSerialPolling({
    intervalMs: 4000,
    task: async () => {
      await runOverviewLoad(false);
    },
  });

  async function refreshNow(): Promise<void> {
    await runOverviewLoad(true);
  }

  function goToCreateServer(): void {
    void router.push({ name: NEXT_SERVER_CREATE_ROUTE_NAME });
  }

  function goToImportServer(): void {
    void router.push({ name: NEXT_SERVER_IMPORT_ROUTE_NAME });
  }

  async function toggleServer(serverId: string): Promise<void> {
    if (isPreviewDataset.value) return;
    const target = serverCards.value.find((server) => server.id === serverId);
    if (!target) return;
    if (target.canStart) await homeActionsStore.handleStart(serverId);
    else await homeActionsStore.handleStop(serverId);
    await serverStore.refreshStatus(serverId);
    lastUpdatedAt.value = Date.now();
  }

  onMounted(() => {
    void runOverviewLoad(false).finally(() => {
      polling.start();
    });
  });

  return {
    isInitialLoading,
    isRefreshing,
    statsViewMode,
    usingPreviewFallback,
    isPreviewDataset,
    cardLayouts,
    summaryMetrics,
    systemMetrics,
    cpuMetric,
    memoryMetric,
    instanceCountMetric,
    featuredServer,
    secondaryServers,
    alertItems,
    lastUpdatedLabel,
    refreshNow,
    toggleStatsViewMode,
    goToCreateServer,
    goToImportServer,
    toggleServer,
    runningServerCount,
    issueServerCount,
    totalServerCount,
  };
}
