import { computed, onMounted, shallowRef } from "vue";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import type { ServerStatusInfo } from "@api/server";
import type { LocalStartupMode, ServerInstance } from "@type/server";
import { formatServerPath } from "@utils/formatters";
import { formatServerCoreTypeLabel } from "@utils/serverCoreLabel";

export type ServersPageTarget = "console" | "config" | "players";

export interface ServersPageActionItem {
  target: ServersPageTarget;
  label: string;
}

export interface ServersPageServerItem {
  id: string;
  name: string;
  isCurrent: boolean;
  statusLabel: string;
  statusTone: "running" | "starting" | "stopping" | "stopped" | "error";
  runtimeSummary: string;
  coreSummary: string;
  versionSummary: string | null;
  pathSummary: string;
  pathTooltip: string;
  portSummary: string;
  memorySummary: string;
  detailSummary: string | null;
  actions: ServersPageActionItem[];
}

const SERVER_ACTIONS: ServersPageActionItem[] = [
  { target: "console", label: i18n.t("common.console") },
  { target: "config", label: i18n.t("common.config_edit") },
  { target: "players", label: i18n.t("common.player_manage") },
];

function formatStatusLabel(status: ServerStatusInfo["status"] | undefined): string {
  if (status === "Running") return i18n.t("home.running");
  if (status === "Starting") return i18n.t("home.starting");
  if (status === "Stopping") return i18n.t("home.stopping");
  if (status === "Error") return i18n.t("home.error");
  if (status === "Stopped") return i18n.t("home.stopped");
  return i18n.t("common.loading");
}

function formatStatusTone(
  status: ServerStatusInfo["status"] | undefined,
): ServersPageServerItem["statusTone"] {
  if (status === "Running") return "running";
  if (status === "Starting") return "starting";
  if (status === "Stopping") return "stopping";
  if (status === "Error") return "error";
  return "stopped";
}

function formatLocalStartupMode(mode: LocalStartupMode): string {
  if (mode === "starter") return "Starter";
  if (mode === "custom") return "Custom";
  return mode.toUpperCase();
}

function formatRuntimeSummary(server: ServerInstance): string {
  if (server.runtime.kind === "docker_itzg") {
    const imageTag = server.runtime.image_tag ? `:${server.runtime.image_tag}` : "";
    return `Docker · ${server.runtime.image}${imageTag}`;
  }

  return `本地运行 · ${formatLocalStartupMode(server.runtime.startup_mode)}`;
}

function formatCoreSummary(server: ServerInstance): string {
  return formatServerCoreTypeLabel(server.core_type);
}

function formatVersionSummary(server: ServerInstance): string | null {
  const parts: string[] = [];

  if (server.mc_version) {
    parts.push(`MC ${server.mc_version}`);
  }

  if (server.core_version) {
    parts.push(`Core ${server.core_version}`);
  }

  return parts.length > 0 ? parts.join(" · ") : null;
}

function formatDetailSummary(status: ServerStatusInfo | undefined): string | null {
  if (status?.error_message) {
    return status.error_message;
  }

  if (status?.detail_message) {
    return status.detail_message;
  }

  return null;
}

function buildClassicPath(target: ServersPageTarget, serverId?: string): string {
  const encodedId = serverId ? `/${encodeURIComponent(serverId)}` : "";
  return `/${target}${encodedId}`;
}

function navigateToPath(path: string): void {
  window.location.assign(path);
}

export function useServersPage() {
  const serverStore = useServerStore();
  const bootstrapping = shallowRef(true);
  const refreshing = shallowRef(false);
  const loadedOnce = shallowRef(false);

  function ensureCurrentServer(): void {
    if (
      serverStore.currentServerId &&
      serverStore.servers.some((server) => server.id === serverStore.currentServerId)
    ) {
      return;
    }

    serverStore.setCurrentServer(serverStore.servers[0]?.id ?? null);
  }

  async function loadData(manual = false): Promise<void> {
    if (manual) {
      refreshing.value = true;
    } else if (!loadedOnce.value) {
      bootstrapping.value = true;
    }

    try {
      await serverStore.refreshList();
      ensureCurrentServer();
      await serverStore.refreshAllStatuses();
      ensureCurrentServer();
      loadedOnce.value = true;
    } finally {
      bootstrapping.value = false;
      refreshing.value = false;
    }
  }

  function selectServer(serverId: string): void {
    serverStore.setCurrentServer(serverId);
  }

  function navigateToServerTarget(serverId: string, target: ServersPageTarget): void {
    serverStore.setCurrentServer(serverId);
    navigateToPath(buildClassicPath(target, serverId));
  }

  function navigateToCreate(): void {
    navigateToPath("/create");
  }

  function navigateToImport(): void {
    navigateToPath("/add-existing");
  }

  const serverItems = computed<ServersPageServerItem[]>(() =>
    serverStore.servers.map((server) => {
      const status = serverStore.statuses[server.id];

      return {
        id: server.id,
        name: server.name,
        isCurrent: server.id === serverStore.currentServerId,
        statusLabel: formatStatusLabel(status?.status),
        statusTone: formatStatusTone(status?.status),
        runtimeSummary: formatRuntimeSummary(server),
        coreSummary: formatCoreSummary(server),
        versionSummary: formatVersionSummary(server),
        pathSummary: formatServerPath(server.path),
        pathTooltip: server.path,
        portSummary: String(server.port || 25565),
        memorySummary: `${server.max_memory} MB`,
        detailSummary: formatDetailSummary(status),
        actions: SERVER_ACTIONS,
      };
    }),
  );

  const totalCount = computed(() => serverItems.value.length);
  const runningCount = computed(
    () =>
      Object.values(serverStore.statuses).filter((status) => status.status === "Running").length,
  );
  const hasServers = computed(() => totalCount.value > 0);
  const isLoading = computed(
    () => bootstrapping.value || refreshing.value || serverStore.listLoading,
  );
  const errorMessage = computed(() => serverStore.error);

  onMounted(() => {
    void loadData(false);
  });

  return {
    serverStore,
    serverItems,
    totalCount,
    runningCount,
    hasServers,
    isLoading,
    isBootstrapping: bootstrapping,
    isRefreshing: refreshing,
    errorMessage,
    loadData,
    selectServer,
    navigateToServerTarget,
    navigateToCreate,
    navigateToImport,
  };
}
