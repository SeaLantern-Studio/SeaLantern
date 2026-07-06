import { computed, onMounted, shallowRef } from "vue";
import { serverApi } from "@api/server";
import { useRouter } from "vue-router";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import type { ServerStatusInfo } from "@api/server";
import type { LocalStartupMode, ServerInstance } from "@type/server";
import { formatServerPath } from "@utils/formatters";
import { formatServerCoreTypeLabel } from "@utils/serverCoreLabel";
import {
  NEXT_SERVER_INSTANCE_CONSOLE_ROUTE_NAME,
  NEXT_SERVER_CREATE_ROUTE_NAME,
  NEXT_SERVER_IMPORT_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
} from "@src/router/pageMeta";
import {
  pickDeleteConfirmationItem,
  shouldUseDeleteConfirmationItem,
} from "@src/utils/serverDeleteConfirmation";
import {
  DEFAULT_SERVER_DELETE_MODE,
  type ServerDeleteMode,
} from "@src/utils/serverDeleteMode";
import type { ConfirmDialogOption } from "@components/common/confirmDialogTypes";

let serversPageLoadedOnce = false;

export type ServersPageTarget = "delete" | "console" | "players" | "extensions" | "config";

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
  { target: "delete", label: i18n.t("home.delete_server") },
  { target: "console", label: i18n.t("common.console") },
  { target: "players", label: i18n.t("common.player_manage") },
  { target: "extensions", label: i18n.t("common.plugins") },
  { target: "config", label: i18n.t("common.config_edit") },
];

async function navigateToCreate(router: ReturnType<typeof useRouter>): Promise<void> {
  await router.push({ name: NEXT_SERVER_CREATE_ROUTE_NAME });
}

async function navigateToImport(router: ReturnType<typeof useRouter>): Promise<void> {
  await router.push({ name: NEXT_SERVER_IMPORT_ROUTE_NAME });
}

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
  if (mode === "starter") return i18n.t("servers.next.runtime.startup_mode.starter");
  if (mode === "custom") return i18n.t("servers.next.runtime.startup_mode.custom");
  return mode.toUpperCase();
}

function formatRuntimeSummary(server: ServerInstance): string {
  if (server.runtime.kind === "docker_itzg") {
    const imageTag = server.runtime.image_tag ? `:${server.runtime.image_tag}` : "";
    return `${i18n.t("servers.next.runtime.docker")} · ${server.runtime.image}${imageTag}`;
  }

  return `${i18n.t("servers.next.runtime.local")} · ${formatLocalStartupMode(server.runtime.startup_mode)}`;
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
  return status?.display_message ?? null;
}

export function useServersPage() {
  const router = useRouter();
  const serverStore = useServerStore();
  const bootstrapping = shallowRef(!serversPageLoadedOnce && serverStore.servers.length === 0);
  const refreshing = shallowRef(false);
  const loadedOnce = shallowRef(false);
  const deleteDialogVisible = shallowRef(false);
  const deleteSubmitting = shallowRef(false);
  const deletingServerId = shallowRef("");
  const deleteExpectedInput = shallowRef("");
  const deletePromptMessage = shallowRef("");
  const deleteInputPlaceholder = shallowRef(i18n.t("home.delete_input_placeholder"));
  const deleteSelectedMode = shallowRef<ServerDeleteMode>(DEFAULT_SERVER_DELETE_MODE);

  const deleteModeOptions = computed<ConfirmDialogOption[]>(() => [
    {
      value: "record-only",
      label: i18n.t("home.delete_mode_record_only"),
      description: i18n.t("home.delete_mode_record_only_desc"),
    },
    {
      value: "record-and-files",
      label: i18n.t("home.delete_mode_with_files"),
      description: i18n.t("home.delete_mode_with_files_desc"),
    },
  ]);

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
      serversPageLoadedOnce = true;
    } finally {
      bootstrapping.value = false;
      refreshing.value = false;
    }
  }

  function selectServer(serverId: string): void {
    serverStore.setCurrentServer(serverId);
  }

  async function navigateToServerTarget(
    serverId: string,
    target: ServersPageTarget,
  ): Promise<void> {
    serverStore.setCurrentServer(serverId);

    if (target === "delete") {
      const server = serverStore.getServerById(serverId);
      if (!server) {
        return;
      }

      deletingServerId.value = server.id;
      if (shouldUseDeleteConfirmationItem(server.name)) {
        const item = pickDeleteConfirmationItem();
        deleteExpectedInput.value = item;
        deletePromptMessage.value = i18n.t("home.delete_confirm_item_message", { item });
        deleteInputPlaceholder.value = i18n.t("home.delete_input_placeholder_item");
      } else {
        deleteExpectedInput.value = server.name;
        deletePromptMessage.value = i18n.t("home.delete_confirm_message", { server: server.name });
        deleteInputPlaceholder.value = i18n.t("home.delete_input_placeholder");
      }
      deleteSelectedMode.value = DEFAULT_SERVER_DELETE_MODE;
      deleteDialogVisible.value = true;
      return;
    }

    if (target === "console") {
      await router.push({ name: NEXT_SERVER_INSTANCE_CONSOLE_ROUTE_NAME, params: { serverId } });
      return;
    }

    if (target === "config") {
      await router.push({ name: NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME, params: { serverId } });
      return;
    }

    if (target === "extensions") {
      await router.push({ name: NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME, params: { serverId } });
      return;
    }

    await router.push({ name: NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME, params: { serverId } });
  }

  async function confirmDeleteServer(): Promise<void> {
    if (!deletingServerId.value) {
      return;
    }

    deleteSubmitting.value = true;
    try {
      if (deleteSelectedMode.value === "record-only") {
        await serverApi.deleteServerRecordOnly(deletingServerId.value);
      } else {
        await serverApi.deleteServerWithFiles(deletingServerId.value);
      }
      deleteDialogVisible.value = false;
      deletingServerId.value = "";
      await loadData(true);
    } finally {
      deleteSubmitting.value = false;
    }
  }

  function closeDeleteDialog(): void {
    if (deleteSubmitting.value) {
      return;
    }

    deleteDialogVisible.value = false;
    deletingServerId.value = "";
    deleteExpectedInput.value = "";
    deletePromptMessage.value = "";
    deleteInputPlaceholder.value = i18n.t("home.delete_input_placeholder");
    deleteSelectedMode.value = DEFAULT_SERVER_DELETE_MODE;
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
    loadedOnce.value = serversPageLoadedOnce;
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
    deleteDialogVisible,
    deleteSubmitting,
    deleteExpectedInput,
    deletePromptMessage,
    deleteInputPlaceholder,
    deleteSelectedMode,
    deleteModeOptions,
    loadData,
    selectServer,
    navigateToServerTarget,
    confirmDeleteServer,
    closeDeleteDialog,
    navigateToCreate: () => navigateToCreate(router),
    navigateToImport: () => navigateToImport(router),
  };
}
