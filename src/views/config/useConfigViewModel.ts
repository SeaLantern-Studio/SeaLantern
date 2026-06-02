import { computed, ref } from "vue";
import type { RouteLocationNormalizedLoaded } from "vue-router";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import { useConfigPlugins } from "@views/config/useConfigPlugins";
import { useConfigCompare } from "@views/config/useConfigCompare";
import { useConfigPropertiesDialogs } from "@views/config/useConfigPropertiesDialogs";
import { useConfigPageLifecycle } from "@views/config/useConfigPageLifecycle";
import { useConfigPropertiesSectionBindings } from "@views/config/useConfigPropertiesSectionBindings";
import { useConfigPropertiesEditor } from "@views/config/useConfigPropertiesEditor";

type ConfigTabKey = "properties" | "plugins" | "startup";

function resolveActiveTab(route: RouteLocationNormalizedLoaded): ConfigTabKey {
  const tab = route.query.tab as string;
  if (tab === "startup") {
    return "startup";
  }
  if (tab === "plugins") {
    return "plugins";
  }
  return "properties";
}

function buildServerPropertiesPath(path: string) {
  const basePath = path.replace(/[/\\]$/, "");
  if (!basePath) {
    return "server.properties";
  }

  const separator = basePath.includes("\\") ? "\\" : "/";
  return `${basePath}${separator}server.properties`;
}

export interface UseConfigViewModelOptions {
  route: RouteLocationNormalizedLoaded;
}

export function useConfigViewModel(options: UseConfigViewModelOptions) {
  const store = useServerStore();

  const error = ref<string | null>(null);
  const successMsg = ref<string | null>(null);
  const activeTab = ref<ConfigTabKey>(resolveActiveTab(options.route));
  const configSaveDiffModalWidth = "1040px";

  const routeId = computed(() => (options.route.params.id as string) || "");
  const currentServerId = computed(() => store.currentServerId);
  const currentServer = computed(
    () => store.servers.find((server) => server.id === store.currentServerId) || null,
  );
  const serverPath = computed(() => currentServer.value?.path || "");
  const serverPropertiesPath = computed(() => buildServerPropertiesPath(serverPath.value));

  function setError(message: string | null) {
    error.value = message;
  }

  function setSuccess(message: string | null) {
    successMsg.value = message;
  }

  function updateCurrentServerPort(port: string) {
    if (!port) {
      return;
    }

    const activeServer = store.servers.find((server) => server.id === store.currentServerId);
    if (activeServer) {
      activeServer.port = parseInt(port) || 25565;
    }
  }

  function handleStartupConfigSaved(maxMemory: number, minMemory: number) {
    const activeServer = store.servers.find((server) => server.id === store.currentServerId);
    if (activeServer) {
      activeServer.max_memory = maxMemory;
      activeServer.min_memory = minMemory;
    }
  }

  const propertiesEditor = useConfigPropertiesEditor({
    serverPath,
    serverPropertiesPath,
    currentServerId,
    currentServerName: computed(() => currentServer.value?.name || ""),
    setError,
    setSuccess,
    updateCurrentServerPort,
  });

  const compare = useConfigCompare({
    currentServerId,
    servers: computed(() => store.servers),
    sourceEntries: propertiesEditor.entries,
    sourceValues: propertiesEditor.editValues,
    sourceNumericFieldErrors: propertiesEditor.numericFieldErrors,
    activeCategory: propertiesEditor.activeCategory,
    searchQuery: propertiesEditor.searchQuery,
    getTranslatedPropertyDescription: propertiesEditor.getTranslatedPropertyDescription,
    setError,
  });

  propertiesEditor.bindCompareContext({
    compareMode: compare.compareMode,
    compareTargetServerId: compare.compareTargetServerId,
    compareTargetEntries: compare.compareTargetEntries,
    compareTargetPath: compare.compareTargetPath,
    compareTargetServerName: computed(
      () => compare.compareTargetServer.value?.name || i18n.t("config.compare.target_server"),
    ),
    compareTargetServerPropertiesPath: compare.compareTargetServerPropertiesPath,
    compareTargetDraftValues: compare.compareTargetDraftValues,
    compareTargetLoadedValues: compare.compareTargetLoadedValues,
    compareTargetSourceDraftText: compare.compareTargetSourceDraftText,
    compareTargetLoadedSourceText: compare.compareTargetLoadedSourceText,
    compareTargetNumericFieldErrors: compare.compareTargetNumericFieldErrors,
    loadCompareProperties: compare.loadCompareProperties,
    applyParsedCompareTargetState: compare.applyParsedCompareTargetState,
    applyCompareTargetSourceDraftToVisualState: compare.applyCompareTargetSourceDraftToVisualState,
    buildCompareTargetPreviewSource: compare.buildCompareTargetPreviewSource,
    prepareCompareTargetSourceDraftForSourceMode:
      compare.prepareCompareTargetSourceDraftForSourceMode,
    updateCompareTargetSourceDraft: compare.updateCompareTargetSourceDraft,
    captureDifferenceCategorySnapshot: compare.captureDifferenceCategorySnapshot,
  });

  const pluginsState = useConfigPlugins({
    currentServerId,
    getCurrentServer: () => currentServer.value,
    setError,
  });

  const configTabs = computed(() => [
    {
      key: "properties",
      label: i18n.t("config.server_properties"),
      count: "i",
      countTitle: serverPropertiesPath.value,
    },
    { key: "startup", label: i18n.t("config.startup_properties") },
    { key: "plugins", label: i18n.t("config.server_plugins") },
  ]);

  const editorModeTabs = computed(() => [
    { key: "visual", label: i18n.t("config.visual_mode") },
    { key: "source", label: i18n.t("config.source_mode") },
  ]);

  const gamemodeOptions = computed(() => [
    { label: i18n.t("config.gamemode.survival"), value: "survival" },
    { label: i18n.t("config.gamemode.creative"), value: "creative" },
    { label: i18n.t("config.gamemode.adventure"), value: "adventure" },
    { label: i18n.t("config.gamemode.spectator"), value: "spectator" },
  ]);

  const difficultyOptions = computed(() => [
    { label: i18n.t("config.difficulty.peaceful"), value: "peaceful" },
    { label: i18n.t("config.difficulty.easy"), value: "easy" },
    { label: i18n.t("config.difficulty.normal"), value: "normal" },
    { label: i18n.t("config.difficulty.hard"), value: "hard" },
  ]);

  const translatedDescriptionByKey = computed(() => {
    const result: Record<string, string> = {};
    propertiesEditor.filteredEntries.value.forEach((entry) => {
      result[entry.key] = propertiesEditor.getTranslatedPropertyDescription(entry.key);
    });
    return result;
  });

  const currentServerName = computed(
    () => currentServer.value?.name || i18n.t("config.compare.source_server"),
  );
  const compareTargetServerName = computed(
    () => compare.compareTargetServer.value?.name || i18n.t("config.compare.target_server"),
  );
  const propertiesSectionBindings = useConfigPropertiesSectionBindings({
    propertiesEditor,
    compare,
    currentServerName,
    compareTargetServerName,
    translatedDescriptionByKey,
    gamemodeOptions,
    difficultyOptions,
  });
  const propertiesDialogs = useConfigPropertiesDialogs({
    propertiesEditor,
    modalWidth: configSaveDiffModalWidth,
  });

  useConfigPageLifecycle({
    routeId,
    currentServerId,
    serverCount: computed(() => store.servers.length),
    setCurrentServer: (id) => {
      if (id) {
        store.setCurrentServer(id);
        return;
      }

      if (!store.currentServerId && store.servers.length > 0) {
        store.setCurrentServer(store.servers[0].id);
      }
    },
    refreshList: () => store.refreshList(),
    loadProperties: () => propertiesEditor.loadProperties(),
    loadPlugins: () => pluginsState.loadPlugins(),
    compareTargetServerId: compare.compareTargetServerId,
    compareServerOptions: compare.compareServerOptions,
    hasCompareTargets: compare.hasCompareTargets,
    compareMode: compare.compareMode,
    loadCompareProperties: () => compare.loadCompareProperties(),
    resetCompareState: (clearTarget) => compare.resetCompareState(clearTarget),
  });

  return {
    store,
    error,
    successMsg,
    activeTab,
    configSaveDiffModalWidth,
    routeId,
    currentServerId,
    currentServer,
    currentServerName,
    compareTargetServerName,
    serverPath,
    setError,
    propertiesEditor,
    compare,
    propertiesDialogs,
    propertiesSectionBindings,
    pluginsState,
    configTabs,
    editorModeTabs,
    gamemodeOptions,
    difficultyOptions,
    translatedDescriptionByKey,
    handleStartupConfigSaved,
  };
}
