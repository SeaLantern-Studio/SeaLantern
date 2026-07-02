import { computed, ref, watch } from "vue";
import type { RouteLocationNormalizedLoaded } from "vue-router";
import { configApi } from "@api/config";
import type { ServerConfigDiscoveryOptions, ServerConfigJsonMode } from "@api/config";
import { systemApi } from "@api/system";
import { i18n } from "@language";
import { useServerStore } from "@stores/serverStore";
import { useConfigCompare } from "@src/features/config-editor/useConfigCompare";
import { useConfigPageLifecycle } from "@src/features/config-editor/useConfigPageLifecycle";
import { useConfigPlugins } from "@src/features/config-editor/useConfigPlugins";
import { useConfigPropertiesDialogs } from "@src/features/config-editor/useConfigPropertiesDialogs";
import { useConfigPropertiesEditor } from "@src/features/config-editor/useConfigPropertiesEditor";
import { useConfigPropertiesSectionBindings } from "@src/features/config-editor/useConfigPropertiesSectionBindings";

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

function createDefaultDiscoveryOptions(): ServerConfigDiscoveryOptions {
  return {
    manual_import_dirs: [],
    manual_import_files: [],
    json_mode: "filtered",
  };
}

function dedupePaths(paths: string[]) {
  return Array.from(new Set(paths.map((path) => path.trim()).filter(Boolean)));
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

  const configFiles = ref<Awaited<ReturnType<typeof configApi.listServerConfigFiles>>>([]);
  const configSearchResults = ref<Awaited<ReturnType<typeof configApi.searchServerConfigFiles>>>(
    [],
  );
  const configDiscoveryOptions = ref<ServerConfigDiscoveryOptions>(createDefaultDiscoveryOptions());
  const configSearchQuery = ref("");
  const configSearchMode = ref<"keyword" | "regex" | "similarity">("keyword");
  const configSearchScope = ref<"path" | "content" | "all">("path");
  const configSearchLoading = ref(false);
  const configSearchError = ref<string | null>(null);
  const selectedConfigLocator = ref("");
  const selectedConfigRelativePath = ref("");
  let configSearchRequestId = 0;

  const currentConfigFile = computed(() => {
    if (selectedConfigLocator.value) {
      const located =
        configFiles.value.find((file) => file.locator === selectedConfigLocator.value) || null;
      if (located) {
        return located;
      }
    }

    if (!selectedConfigRelativePath.value) {
      return null;
    }

    return (
      configFiles.value.find((file) => file.relative_path === selectedConfigRelativePath.value) ||
      null
    );
  });

  const visibleConfigFiles = computed(() => {
    const baseFiles = configFiles.value;
    if (!configSearchQuery.value.trim()) {
      return baseFiles;
    }

    const mapped = configSearchResults.value
      .map((hit) => baseFiles.find((file) => file.locator === hit.locator) || null)
      .filter((file): file is NonNullable<(typeof baseFiles)[number]> => !!file);

    const selected = currentConfigFile.value;
    if (selected && !mapped.some((file) => file.locator === selected.locator)) {
      return [selected, ...mapped];
    }

    return mapped;
  });

  const currentConfigLabel = computed(
    () => currentConfigFile.value?.relative_path || i18n.t("config.config_files"),
  );
  const currentConfigFilePath = computed(() => currentConfigFile.value?.absolute_path || "");

  function syncSelectedConfig(file: (typeof configFiles.value)[number] | null) {
    selectedConfigLocator.value = file?.locator || "";
    selectedConfigRelativePath.value = file?.relative_path || "";
  }

  async function loadConfigFiles() {
    if (!serverPath.value) {
      configFiles.value = [];
      configSearchResults.value = [];
      configSearchError.value = null;
      syncSelectedConfig(null);
      return;
    }

    const files = await configApi.listServerConfigFiles(
      serverPath.value,
      configDiscoveryOptions.value,
    );
    configFiles.value = files;
    await refreshConfigSearch();

    const current = files.find((file) => file.locator === selectedConfigLocator.value) || null;
    if (current) {
      syncSelectedConfig(current);
      return;
    }

    const preferred =
      files.find((file) => file.known_role === "server_properties") ||
      files.find((file) => file.known_role === "startup_primary") ||
      files[0] ||
      null;
    syncSelectedConfig(preferred);
  }

  async function refreshConfigSearch() {
    const query = configSearchQuery.value.trim();
    configSearchError.value = null;

    if (!serverPath.value || !query) {
      configSearchResults.value = [];
      configSearchLoading.value = false;
      return;
    }

    const requestId = ++configSearchRequestId;
    configSearchLoading.value = true;
    try {
      const hits = await configApi.searchServerConfigFiles(
        serverPath.value,
        query,
        configSearchMode.value,
        configSearchScope.value,
        configDiscoveryOptions.value,
        100,
        false,
      );
      if (requestId !== configSearchRequestId) {
        return;
      }
      configSearchResults.value = hits;
    } catch (e) {
      if (requestId !== configSearchRequestId) {
        return;
      }
      configSearchResults.value = [];
      configSearchError.value = String(e);
    } finally {
      if (requestId === configSearchRequestId) {
        configSearchLoading.value = false;
      }
    }
  }

  function updateSelectedConfigFile(value: string | number) {
    const locator = String(value);
    const next = configFiles.value.find((file) => file.locator === locator) || null;
    syncSelectedConfig(next);
  }

  function updateConfigSearchQuery(value: string) {
    configSearchQuery.value = value;
  }

  function updateConfigSearchMode(value: string | number) {
    configSearchMode.value = String(value) as typeof configSearchMode.value;
  }

  function updateConfigSearchScope(value: string | number) {
    configSearchScope.value = String(value) as typeof configSearchScope.value;
  }

  async function updateConfigJsonMode(value: ServerConfigJsonMode) {
    configDiscoveryOptions.value = {
      ...configDiscoveryOptions.value,
      json_mode: value,
    };
    await loadConfigFiles();
  }

  async function importConfigDirectory() {
    const selected = await systemApi.pickFolder();
    if (!selected) {
      return;
    }

    configDiscoveryOptions.value = {
      ...configDiscoveryOptions.value,
      manual_import_dirs: dedupePaths([
        ...configDiscoveryOptions.value.manual_import_dirs,
        selected,
      ]),
    };
    await loadConfigFiles();
  }

  async function importConfigFile() {
    const selected = await systemApi.pickFile();
    if (!selected) {
      return;
    }

    configDiscoveryOptions.value = {
      ...configDiscoveryOptions.value,
      manual_import_files: dedupePaths([
        ...configDiscoveryOptions.value.manual_import_files,
        selected,
      ]),
    };
    await loadConfigFiles();
  }

  async function removeConfigImportDirectory(path: string) {
    configDiscoveryOptions.value = {
      ...configDiscoveryOptions.value,
      manual_import_dirs: configDiscoveryOptions.value.manual_import_dirs.filter(
        (item) => item !== path,
      ),
    };
    await loadConfigFiles();
  }

  async function removeConfigImportFile(path: string) {
    configDiscoveryOptions.value = {
      ...configDiscoveryOptions.value,
      manual_import_files: configDiscoveryOptions.value.manual_import_files.filter(
        (item) => item !== path,
      ),
    };
    await loadConfigFiles();
  }

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
    currentConfigFile,
    currentConfigLocator: computed(() => selectedConfigLocator.value),
    discoveryOptions: computed(() => configDiscoveryOptions.value),
    selectedConfigRelativePath,
    currentConfigFilePath,
    currentConfigLabel,
    currentServerId,
    currentServerName: computed(() => currentServer.value?.name || ""),
    setError,
    setSuccess,
    updateCurrentServerPort,
  });

  const compare = useConfigCompare({
    currentServerId,
    servers: computed(() => store.servers),
    sourceConfigRelativePath: computed(() => currentConfigFile.value?.relative_path || ""),
    sourceConfigKind: computed(() => currentConfigFile.value?.kind || null),
    sourceConfigSourceKind: computed(() => currentConfigFile.value?.source_kind || null),
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
      label: i18n.t("config.config_files"),
      count: "i",
      countTitle: currentConfigFilePath.value,
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
    allConfigFiles: computed(() => configFiles.value),
    configFiles: visibleConfigFiles,
    selectedConfigLocator: computed(() => selectedConfigLocator.value),
    discoveryOptions: computed(() => configDiscoveryOptions.value),
    configSearchQuery: computed(() => configSearchQuery.value),
    configSearchMode: computed(() => configSearchMode.value),
    configSearchScope: computed(() => configSearchScope.value),
    configSearchResults: computed(() => configSearchResults.value),
    configSearchLoading: computed(() => configSearchLoading.value),
    configSearchError: computed(() => configSearchError.value),
    updateSelectedConfigFile,
    updateConfigSearchQuery,
    updateConfigSearchMode,
    updateConfigSearchScope,
    updateConfigJsonMode,
    importConfigDirectory,
    importConfigFile,
    removeConfigImportDirectory,
    removeConfigImportFile,
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
    loadConfigFiles,
    loadProperties: () => propertiesEditor.loadProperties(),
    loadPlugins: () => pluginsState.loadPlugins(),
    compareTargetServerId: compare.compareTargetServerId,
    compareServerOptions: compare.compareServerOptions,
    hasCompareTargets: compare.hasCompareTargets,
    compareMode: compare.compareMode,
    loadCompareProperties: () => compare.loadCompareProperties(),
    resetCompareState: (clearTarget) => compare.resetCompareState(clearTarget),
  });

  watch(selectedConfigLocator, async (locator, previousLocator) => {
    if (!locator || locator === previousLocator || !currentServerId.value) {
      return;
    }

    compare.resetCompareState(true);
    await propertiesEditor.loadProperties();
  });

  watch([configSearchQuery, configSearchMode, configSearchScope, serverPath], async () => {
    await refreshConfigSearch();
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
    configFiles,
    visibleConfigFiles,
    currentConfigFile,
    currentConfigLabel,
    currentConfigFilePath,
    configDiscoveryOptions,
    selectedConfigLocator,
    selectedConfigRelativePath,
    configSearchQuery,
    configSearchMode,
    configSearchScope,
    configSearchResults,
    configSearchLoading,
    configSearchError,
    loadConfigFiles,
    updateSelectedConfigFile,
    updateConfigSearchQuery,
    updateConfigSearchMode,
    updateConfigSearchScope,
    updateConfigJsonMode,
    importConfigDirectory,
    importConfigFile,
    removeConfigImportDirectory,
    removeConfigImportFile,
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
