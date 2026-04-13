<script setup lang="ts">
import { ref, computed, watch, onMounted, onActivated, nextTick } from "vue";
import { useRoute } from "vue-router";
import SLSpinner from "@components/common/SLSpinner.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLButton from "@components/common/SLButton.vue";
import SLModal from "@components/common/SLModal.vue";
import SLConfirmDialog from "@components/common/SLConfirmDialog.vue";
import SLTooltip from "@components/common/SLTooltip.vue";
import { SLTabBar } from "@components/common";
import { configApi } from "@api/config";
import { m_pluginApi, type m_PluginInfo, type m_PluginConfigFile } from "@api/mcs_plugins";
import type { ConfigEntry as ConfigEntryType } from "@api/config";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import {
  Trash2,
  RefreshCw,
  Save,
  Settings,
  FileText,
  RotateCcw,
  FolderOpen,
  Edit,
  FileDiff,
} from "lucide-vue-next";

import ConfigCategories from "@components/config/ConfigCategories.vue";
import ConfigSourceDiffView from "@components/config/ConfigSourceDiffView.vue";
import ConfigSourceEditor from "@components/config/ConfigSourceEditor.vue";
import ConfigPropertyEditorControl from "@components/config/ConfigPropertyEditorControl.vue";
import ConfigComparePanel from "@components/config/ConfigComparePanel.vue";
import { systemApi } from "@api/system";
import { buildDiffLines } from "@utils/configDiff";
import "@styles/plugin-list.css";
import "@styles/views/ConfigView.css";

const route = useRoute();
const store = useServerStore();

interface CompareEntry {
  key: string;
  description: string;
  category: string;
  sourceEntry: ConfigEntryType | null;
  targetEntry: ConfigEntryType | null;
  sourceValue: string;
  targetValue: string;
  different: boolean;
  onlyInSource: boolean;
  onlyInTarget: boolean;
}

interface PendingSaveItem {
  serverId: string;
  serverName: string;
  filePath: string;
  originalText: string;
  modifiedText: string;
}

interface ComparePanelControlState {
  key: string;
  value: string;
  valueType: string;
  defaultValue: string;
  numericError?: string;
}

interface ComparePanelRow {
  key: string;
  description: string;
  different: boolean;
  onlyInSource: boolean;
  onlyInTarget: boolean;
  source: ComparePanelControlState;
  target: ComparePanelControlState;
}

const entries = ref<ConfigEntryType[]>([]);
const editValues = ref<Record<string, string>>({});
const loadedValues = ref<Record<string, string>>({});
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);
const successMsg = ref<string | null>(null);
const searchQuery = ref("");
const activeCategory = ref("all");
const editorMode = ref<"visual" | "source">("visual");
const sourceDraftText = ref("");
const loadedSourceText = ref("");
const visualModeBaseValues = ref<Record<string, string>>({});
const visualDraftDirty = ref(false);
const modeSwitching = ref(false);
const sourceParseError = ref<string | null>(null);
const showDiscardConfirm = ref(false);
const pendingReloadSide = ref<"current" | "compare" | null>(null);
const showSaveDiffModal = ref(false);
const pendingSaveItems = ref<PendingSaveItem[]>([]);
const compareMode = ref(false);
const compareTargetServerId = ref("");
const compareTargetEntries = ref<ConfigEntryType[]>([]);
const compareTargetDraftValues = ref<Record<string, string>>({});
const compareTargetLoadedValues = ref<Record<string, string>>({});
const compareLoading = ref(false);
const serverPath = computed(() => {
  const server = store.servers.find((s) => s.id === store.currentServerId);
  return server?.path || "";
});
const compareTargetServer = computed(
  () => store.servers.find((s) => s.id === compareTargetServerId.value) || null,
);
const compareTargetPath = computed(() => compareTargetServer.value?.path || "");
const compareServerOptions = computed(() =>
  store.servers
    .filter((server) => server.id !== currentServerId.value)
    .map((server) => ({
      label: server.name,
      value: server.id,
    })),
);

function buildServerPropertiesPath(path: string) {
  const basePath = path.replace(/[/\\]$/, "");
  if (!basePath) {
    return "server.properties";
  }

  const separator = basePath.includes("\\") ? "\\" : "/";
  return `${basePath}${separator}server.properties`;
}

const serverPropertiesPath = computed(() => buildServerPropertiesPath(serverPath.value));
const compareTargetServerPropertiesPath = computed(() =>
  buildServerPropertiesPath(compareTargetPath.value),
);

const plugins = ref<m_PluginInfo[]>([]);
const pluginsLoading = ref(false);
const selectedPlugin = ref<m_PluginInfo | null>(null);
const loadedPlugins = ref<Set<string>>(new Set());
const observer = ref<IntersectionObserver | null>(null);
const activeTab = ref<"properties" | "plugins">("properties");

const configTabs = computed(() => [
  {
    key: "properties",
    label: i18n.t("config.server_properties"),
    count: "i",
    countTitle: serverPropertiesPath.value,
  },
  { key: "plugins", label: i18n.t("config.server_plugins") },
]);

const currentServerId = computed(() => store.currentServerId);

const editorModeTabs = computed(() => [
  { key: "visual", label: i18n.t("config.visual_mode") },
  { key: "source", label: i18n.t("config.source_mode") },
]);

const hasUnsavedChanges = computed(() => {
  const targetDirty =
    compareMode.value &&
    !areMapValuesEqual(compareTargetDraftValues.value, compareTargetLoadedValues.value);

  if (editorMode.value === "source") {
    return sourceDraftText.value !== loadedSourceText.value || targetDirty;
  }

  const sourceDirty = sourceDraftText.value !== loadedSourceText.value;
  const visualDirty = !areMapValuesEqual(editValues.value, loadedValues.value);
  if (!compareMode.value) {
    return sourceDirty || visualDirty;
  }

  return sourceDirty || visualDirty || targetDirty;
});

const saveStatusText = computed(() =>
  hasUnsavedChanges.value ? i18n.t("config.status_unsaved") : i18n.t("config.status_loaded"),
);
const reloadCurrentTooltipText = computed(
  () => `重新载入${currentServer.value?.name || i18n.t("config.current_server")}属性`,
);
const reloadCompareTooltipText = computed(
  () => `重新载入${compareTargetServer.value?.name || i18n.t("config.compare.target_server")}属性`,
);
const currentSideDirty = computed(
  () =>
    sourceDraftText.value !== loadedSourceText.value ||
    !areMapValuesEqual(editValues.value, loadedValues.value),
);
const compareSideDirty = computed(
  () => !areMapValuesEqual(compareTargetDraftValues.value, compareTargetLoadedValues.value),
);
const discardConfirmTitle = computed(() => {
  if (pendingReloadSide.value === "compare") {
    return "丢弃对照侧修改";
  }
  if (pendingReloadSide.value === "current") {
    return "丢弃当前侧修改";
  }
  return i18n.t("config.discard_title");
});
const discardConfirmMessage = computed(() => {
  if (pendingReloadSide.value === "compare") {
    return `重新载入将丢弃 ${compareTargetServer.value?.name || i18n.t("config.compare.target_server")} 的未保存属性修改。`;
  }
  if (pendingReloadSide.value === "current") {
    return `重新载入将丢弃 ${currentServer.value?.name || i18n.t("config.current_server")} 的未保存属性修改。`;
  }
  return i18n.t("config.discard_message");
});

const compareEntries = computed(() =>
  buildCompareEntries(
    entries.value,
    editValues.value,
    compareTargetEntries.value,
    compareTargetDraftValues.value,
  ),
);

const pendingSaveItemsWithStats = computed(() =>
  pendingSaveItems.value.map((item) => {
    const diffLines = buildDiffLines(item.originalText, item.modifiedText);
    let additions = 0;
    let deletions = 0;

    for (const line of diffLines) {
      if (line.type === "addition") additions += 1;
      if (line.type === "deletion") deletions += 1;
    }

    return {
      ...item,
      additions,
      deletions,
    };
  }),
);

const categories = computed(() => {
  const cats = new Set(entries.value.map((e) => e.category));
  return ["all", ...Array.from(cats)];
});

const gamemodeOptions = ref([
  { label: i18n.t("config.gamemode.survival"), value: "survival" },
  { label: i18n.t("config.gamemode.creative"), value: "creative" },
  { label: i18n.t("config.gamemode.adventure"), value: "adventure" },
  { label: i18n.t("config.gamemode.spectator"), value: "spectator" },
]);

const difficultyOptions = ref([
  { label: i18n.t("config.difficulty.peaceful"), value: "peaceful" },
  { label: i18n.t("config.difficulty.easy"), value: "easy" },
  { label: i18n.t("config.difficulty.normal"), value: "normal" },
  { label: i18n.t("config.difficulty.hard"), value: "hard" },
]);

const filteredEntries = computed(() => {
  return entries.value.filter((e: ConfigEntryType) => {
    const matchCat = activeCategory.value === "all" || e.category === activeCategory.value;
    const matchSearch =
      !searchQuery.value ||
      e.key.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (e.description ?? "").toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchCat && matchSearch;
  });
});

const filteredCompareEntries = computed(() => {
  return compareEntries.value.filter((entry) => {
    const matchCat = activeCategory.value === "all" || entry.category === activeCategory.value;
    const matchSearch =
      !searchQuery.value ||
      entry.key.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      entry.description.toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchCat && matchSearch;
  });
});

const compareDifferenceCount = computed(
  () => compareEntries.value.filter((entry) => entry.different).length,
);
const hasCompareTargets = computed(() => compareServerOptions.value.length > 0);
const compareDifferenceBadgeText = computed(() =>
  i18n.t("config.compare.difference_badge", { count: compareDifferenceCount.value }),
);
const configSaveDiffModalWidth = "1040px";

const numericFieldErrors = computed(() => {
  const errors: Record<string, string> = {};

  for (const entry of entries.value) {
    if (entry.value_type !== "number") {
      continue;
    }

    const value = editValues.value[entry.key]?.trim() ?? "";
    if (value.length === 0 || !/^-?\d+$/.test(value)) {
      errors[entry.key] = `${entry.key} 需要填写整数`;
    }
  }

  return errors;
});

const hasInvalidNumericValues = computed(() => Object.keys(numericFieldErrors.value).length > 0);

const compareTargetNumericFieldErrors = computed(() => {
  const errors: Record<string, string> = {};

  for (const entry of compareEntries.value) {
    if (getCompareValueType(entry, "target") !== "number") {
      continue;
    }

    const value = compareTargetDraftValues.value[entry.key]?.trim() ?? "";
    if (value.length === 0 || !/^-?\d+$/.test(value)) {
      errors[entry.key] = `${entry.key} 需要填写整数`;
    }
  }

  return errors;
});

const comparePanelRows = computed<ComparePanelRow[]>(() =>
  filteredCompareEntries.value.map((entry) => ({
    key: entry.key,
    description: getTranslatedPropertyDescription(entry.key),
    different: entry.different,
    onlyInSource: entry.onlyInSource,
    onlyInTarget: entry.onlyInTarget,
    source: {
      key: entry.key,
      value: entry.sourceValue,
      valueType: getCompareValueType(entry, "source"),
      defaultValue: getCompareDefaultValue(entry, "source"),
      numericError: numericFieldErrors.value[entry.key],
    },
    target: {
      key: entry.key,
      value: entry.targetValue,
      valueType: getCompareValueType(entry, "target"),
      defaultValue: getCompareDefaultValue(entry, "target"),
      numericError: compareTargetNumericFieldErrors.value[entry.key],
    },
  })),
);

function areMapValuesEqual(a: Record<string, string>, b: Record<string, string>) {
  const aKeys = Object.keys(a);
  const bKeys = Object.keys(b);

  if (aKeys.length !== bKeys.length) {
    return false;
  }

  for (const key of aKeys) {
    if (a[key] !== b[key]) {
      return false;
    }
  }

  return true;
}

function getTranslatedPropertyDescription(key: string) {
  const translationKey = `config.properties.${key}`;
  const translated = i18n.t(translationKey);
  return translated === translationKey ? "" : translated;
}

function getChangedPropertyValues() {
  const baseValues =
    sourceDraftText.value !== loadedSourceText.value
      ? visualModeBaseValues.value
      : loadedValues.value;
  return getChangedValues(editValues.value, baseValues);
}

function getChangedValues(
  draftValues: Record<string, string>,
  baseValues: Record<string, string>,
): Record<string, string> {
  const changedValues: Record<string, string> = {};

  for (const [key, value] of Object.entries(draftValues)) {
    if (baseValues[key] !== value) {
      changedValues[key] = value;
    }
  }

  return changedValues;
}

async function buildVisualPreviewSource() {
  const changedValues = getChangedPropertyValues();

  if (sourceDraftText.value !== loadedSourceText.value) {
    return configApi.previewServerPropertiesWriteFromSource(sourceDraftText.value, changedValues);
  }

  return configApi.previewServerPropertiesWrite(serverPath.value, changedValues);
}

function applyParsedSourceState(sourceText: string, targetMode: "visual" | "source" = "visual") {
  const parsed = configApi.parseServerPropertiesSource(sourceText);
  return parsed.then((result) => {
    entries.value = result.entries as ConfigEntryType[];
    editValues.value = { ...result.raw };
    loadedValues.value = { ...result.raw };
    visualModeBaseValues.value = { ...result.raw };
    sourceDraftText.value = sourceText;
    loadedSourceText.value = sourceText;
    sourceParseError.value = null;
    visualDraftDirty.value = false;
    editorMode.value = targetMode;

    const port = result.raw["server-port"];
    if (port) {
      updateCurrentServerPort(port);
    }

    return result;
  });
}

async function applyParsedCompareTargetState(sourceText: string) {
  const parsed = await configApi.parseServerPropertiesSource(sourceText);
  compareTargetEntries.value = parsed.entries as ConfigEntryType[];
  compareTargetDraftValues.value = { ...parsed.raw };
  compareTargetLoadedValues.value = { ...parsed.raw };
}

function getCompareValueType(entry: CompareEntry, side: "source" | "target") {
  const primary = side === "source" ? entry.sourceEntry : entry.targetEntry;
  const fallback = side === "source" ? entry.targetEntry : entry.sourceEntry;
  return primary?.value_type ?? fallback?.value_type ?? "string";
}

function getCompareDefaultValue(entry: CompareEntry, side: "source" | "target") {
  const primary = side === "source" ? entry.sourceEntry : entry.targetEntry;
  const fallback = side === "source" ? entry.targetEntry : entry.sourceEntry;
  return primary?.default_value ?? fallback?.default_value ?? "";
}

function buildCompareEntries(
  sourceEntries: ConfigEntryType[],
  sourceValues: Record<string, string>,
  targetEntries: ConfigEntryType[],
  targetValues: Record<string, string>,
): CompareEntry[] {
  const sourceEntryMap = new Map(sourceEntries.map((entry) => [entry.key, entry]));
  const targetEntryMap = new Map(targetEntries.map((entry) => [entry.key, entry]));
  const orderedKeys: string[] = [];
  const seenKeys = new Set<string>();

  const pushKey = (key: string) => {
    if (seenKeys.has(key)) {
      return;
    }
    seenKeys.add(key);
    orderedKeys.push(key);
  };

  sourceEntries.forEach((entry) => pushKey(entry.key));
  targetEntries.forEach((entry) => pushKey(entry.key));
  Object.keys(sourceValues).forEach(pushKey);
  Object.keys(targetValues).forEach(pushKey);

  return orderedKeys.map((key) => {
    const sourceEntry = sourceEntryMap.get(key) ?? null;
    const targetEntry = targetEntryMap.get(key) ?? null;
    const sourceValue = sourceValues[key] ?? "";
    const targetValue = targetValues[key] ?? "";
    return {
      key,
      description: sourceEntry?.description ?? targetEntry?.description ?? "",
      category: sourceEntry?.category ?? targetEntry?.category ?? "other",
      sourceEntry,
      targetEntry,
      sourceValue,
      targetValue,
      different: sourceValue !== targetValue,
      onlyInSource: key in sourceValues && !(key in targetValues),
      onlyInTarget: !(key in sourceValues) && key in targetValues,
    };
  });
}

async function loadCompareProperties() {
  if (!compareMode.value || !compareTargetPath.value) {
    compareTargetEntries.value = [];
    compareTargetDraftValues.value = {};
    compareTargetLoadedValues.value = {};
    return;
  }

  compareLoading.value = true;
  error.value = null;
  try {
    const targetResult = await configApi.readServerProperties(compareTargetPath.value);
    compareTargetEntries.value = targetResult.entries as ConfigEntryType[];
    compareTargetDraftValues.value = { ...targetResult.raw };
    compareTargetLoadedValues.value = { ...targetResult.raw };
  } catch (e) {
    error.value = String(e);
    compareTargetEntries.value = [];
    compareTargetDraftValues.value = {};
    compareTargetLoadedValues.value = {};
  } finally {
    compareLoading.value = false;
  }
}

onMounted(async () => {
  await store.refreshList();
  const routeId = route.params.id as string;
  if (routeId) {
    store.setCurrentServer(routeId);
  } else if (!store.currentServerId && store.servers.length > 0) {
    store.setCurrentServer(store.servers[0].id);
  }
  await loadProperties();
  await loadPlugins();
});

watch(
  () => store.currentServerId,
  async () => {
    if (store.currentServerId) {
      // 这是刻意的设计：切换当前服务器时直接进入新的对照上下文，不保留旧的对照侧草稿。
      if (compareTargetServerId.value === store.currentServerId) {
        compareTargetServerId.value = compareServerOptions.value[0]?.value?.toString() || "";
      }
      await loadProperties();
      await loadPlugins();
    }
  },
);

watch(compareTargetServerId, async () => {
  if (compareMode.value && compareTargetServerId.value) {
    await loadCompareProperties();
  }
});

watch(hasCompareTargets, (hasTargets) => {
  if (hasTargets) {
    return;
  }

  // 这是刻意的设计：当没有可对照服务器时，直接退出对照模式并清空对照侧状态。
  compareMode.value = false;
  compareTargetServerId.value = "";
  compareTargetEntries.value = [];
  compareTargetDraftValues.value = {};
  compareTargetLoadedValues.value = {};
});

async function loadProperties() {
  if (!serverPath.value) return;

  loading.value = true;
  error.value = null;
  try {
    const sourceText = await configApi.readServerPropertiesSource(serverPath.value);
    await applyParsedSourceState(sourceText, "visual");
    if (compareMode.value && compareTargetServerId.value) {
      await loadCompareProperties();
    }
  } catch (e) {
    error.value = String(e);
    entries.value = [];
    editValues.value = {};
    loadedValues.value = {};
    sourceDraftText.value = "";
    loadedSourceText.value = "";
    compareTargetEntries.value = [];
    compareTargetDraftValues.value = {};
    compareTargetLoadedValues.value = {};
  } finally {
    loading.value = false;
  }
}

async function loadCurrentPropertiesOnly() {
  if (!serverPath.value) return;

  loading.value = true;
  error.value = null;
  try {
    const sourceText = await configApi.readServerPropertiesSource(serverPath.value);
    await applyParsedSourceState(sourceText, "visual");
  } catch (e) {
    error.value = String(e);
    entries.value = [];
    editValues.value = {};
    loadedValues.value = {};
    sourceDraftText.value = "";
    loadedSourceText.value = "";
  } finally {
    loading.value = false;
  }
}

/**
 * 更新当前服务器的端口信息
 * @param port 端口号字符串
 */
function updateCurrentServerPort(port: string) {
  if (!port) return;

  const currentServer = store.servers.find((s) => s.id === store.currentServerId);
  if (currentServer) {
    currentServer.port = parseInt(port) || 25565;
  }
}

async function saveProperties() {
  if (!serverPath.value || !hasUnsavedChanges.value || saving.value) return;

  error.value = null;
  pendingSaveItems.value = [];

  try {
    if (editorMode.value === "visual" && hasInvalidNumericValues.value) {
      const invalidKeys = Object.keys(numericFieldErrors.value);
      error.value = `以下字段需要填写整数：${invalidKeys.join("、")}`;
      return;
    }

    if (compareMode.value && Object.keys(compareTargetNumericFieldErrors.value).length > 0) {
      const invalidKeys = Object.keys(compareTargetNumericFieldErrors.value);
      error.value = `以下字段需要填写整数：${invalidKeys.join("、")}`;
      return;
    }

    const pendingItems: PendingSaveItem[] = [];

    const sourceChanged =
      sourceDraftText.value !== loadedSourceText.value ||
      !areMapValuesEqual(editValues.value, loadedValues.value);
    if (sourceChanged) {
      const latestSourceText = await configApi.readServerPropertiesSource(serverPath.value);
      const nextSourceText =
        editorMode.value === "visual" ? await buildVisualPreviewSource() : sourceDraftText.value;

      if (nextSourceText !== latestSourceText) {
        pendingItems.push({
          serverId: currentServerId.value || "",
          serverName: currentServer.value?.name || i18n.t("config.compare.source_server"),
          filePath: serverPropertiesPath.value,
          originalText: latestSourceText,
          modifiedText: nextSourceText,
        });
      } else {
        await applyParsedSourceState(latestSourceText, editorMode.value);
      }
    }

    const targetDirty =
      compareMode.value &&
      !areMapValuesEqual(compareTargetDraftValues.value, compareTargetLoadedValues.value);
    if (targetDirty && compareTargetPath.value) {
      const latestTargetText = await configApi.readServerPropertiesSource(compareTargetPath.value);
      const targetChangedValues = getChangedValues(
        compareTargetDraftValues.value,
        compareTargetLoadedValues.value,
      );
      const nextTargetText = await configApi.previewServerPropertiesWrite(
        compareTargetPath.value,
        targetChangedValues,
      );

      if (nextTargetText !== latestTargetText) {
        pendingItems.push({
          serverId: compareTargetServerId.value,
          serverName: compareTargetServer.value?.name || i18n.t("config.compare.target_server"),
          filePath: compareTargetServerPropertiesPath.value,
          originalText: latestTargetText,
          modifiedText: nextTargetText,
        });
      } else {
        await applyParsedCompareTargetState(latestTargetText);
      }
    }

    pendingSaveItems.value = pendingItems;

    if (pendingSaveItems.value.length === 0) {
      successMsg.value = i18n.t("config.no_changes_to_save");
      setTimeout(() => (successMsg.value = null), 3000);
      return;
    }

    showSaveDiffModal.value = true;
  } catch (e) {
    error.value = String(e);
  }
}

function updateValue(key: string, value: string | boolean | number) {
  editValues.value[key] = String(value);
  visualDraftDirty.value = true;
}

function updateCompareTargetValue(key: string, value: string | boolean | number) {
  compareTargetDraftValues.value[key] = String(value);
}

function updateSourceDraft(value: string) {
  sourceDraftText.value = value;
  sourceParseError.value = null;
}

async function handleEditorModeChange(mode: string | null) {
  const targetMode = mode === "source" ? "source" : "visual";
  if (targetMode === editorMode.value || modeSwitching.value || !serverPath.value) return;

  modeSwitching.value = true;
  error.value = null;

  try {
    if (targetMode === "source") {
      if (visualDraftDirty.value) {
        sourceDraftText.value = await buildVisualPreviewSource();
        visualDraftDirty.value = false;
      }
      sourceParseError.value = null;
      editorMode.value = "source";
      return;
    }

    const parsed = await configApi.parseServerPropertiesSource(sourceDraftText.value);
    entries.value = parsed.entries as ConfigEntryType[];
    editValues.value = { ...parsed.raw };
    visualModeBaseValues.value = { ...parsed.raw };
    visualDraftDirty.value = false;
    sourceParseError.value = null;
    editorMode.value = "visual";
  } catch (e) {
    sourceParseError.value = i18n.t("config.source_parse_failed");
    error.value = String(e);
  } finally {
    modeSwitching.value = false;
  }
}

async function confirmSaveProperties() {
  if (pendingSaveItems.value.length === 0 || saving.value) return;

  saving.value = true;
  error.value = null;
  successMsg.value = null;

  try {
    for (const item of pendingSaveItems.value) {
      await configApi.writeServerPropertiesSource(item.filePath, item.modifiedText);
    }

    const savedCurrent = pendingSaveItems.value.find(
      (item) => item.serverId === currentServerId.value,
    );
    if (savedCurrent) {
      await applyParsedSourceState(savedCurrent.modifiedText, editorMode.value);
    }

    const savedTarget = pendingSaveItems.value.find(
      (item) => item.serverId === compareTargetServerId.value,
    );
    if (savedTarget) {
      await applyParsedCompareTargetState(savedTarget.modifiedText);
    }

    pendingSaveItems.value = [];
    successMsg.value = i18n.t("config.saved");
    showSaveDiffModal.value = false;
    setTimeout(() => (successMsg.value = null), 3000);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

function closeSaveDiffModal() {
  if (saving.value) return;
  pendingSaveItems.value = [];
  showSaveDiffModal.value = false;
}

async function reloadPropertiesWithGuard() {
  pendingReloadSide.value = "current";
  if (currentSideDirty.value) {
    showDiscardConfirm.value = true;
    return;
  }

  await loadCurrentPropertiesOnly();
  pendingReloadSide.value = null;
}

async function reloadComparePropertiesWithGuard() {
  if (!compareMode.value) {
    return;
  }

  pendingReloadSide.value = "compare";
  if (compareSideDirty.value) {
    showDiscardConfirm.value = true;
    return;
  }

  await loadCompareProperties();
  pendingReloadSide.value = null;
}

async function confirmReloadDiscard() {
  showDiscardConfirm.value = false;
  if (pendingReloadSide.value === "compare") {
    await loadCompareProperties();
  } else {
    await loadCurrentPropertiesOnly();
  }
  pendingReloadSide.value = null;
}

function handleCategoryChange(category: string) {
  activeCategory.value = category;
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function handleSearchUpdate(value: string) {
  searchQuery.value = value;
}

function handleCompareModeChange(value: boolean) {
  compareMode.value = value;
  if (!value) {
    // 这是刻意的设计：关闭对照模式时直接丢弃对照侧草稿，回到单列编辑体验。
    compareTargetServerId.value = "";
    compareTargetEntries.value = [];
    compareTargetDraftValues.value = {};
    compareTargetLoadedValues.value = {};
    return;
  }

  if (!compareTargetServerId.value && compareServerOptions.value.length > 0) {
    compareTargetServerId.value = String(compareServerOptions.value[0].value);
  }
  void loadCompareProperties();
}

function handleCompareTargetServerChange(value: string | number) {
  // 这是刻意的设计：切换对照服务器时直接加载新的对照侧内容，不保留旧的对照侧草稿。
  compareTargetServerId.value = String(value);
}

async function loadPlugins() {
  if (!store.currentServerId) return;

  pluginsLoading.value = true;
  error.value = null;
  try {
    plugins.value = await m_pluginApi.m_getPlugins(store.currentServerId);
    loadedPlugins.value = new Set();
    nextTick(() => {
      setupIntersectionObserver();
    });
  } catch (e) {
    error.value = String(e);
    plugins.value = [];
  } finally {
    pluginsLoading.value = false;
  }
}

function setupIntersectionObserver() {
  if (observer.value) {
    observer.value.disconnect();
  }

  observer.value = new IntersectionObserver(
    (intersectionEntries) => {
      intersectionEntries.forEach((entry) => {
        if (entry.isIntersecting) {
          const pluginElement = entry.target as HTMLElement;
          const pluginFileName = pluginElement.getAttribute("data-plugin-file-name");
          if (pluginFileName) {
            loadPluginDetails(pluginFileName);
          }
        }
      });
    },
    {
      rootMargin: "200px 0px",
      threshold: 0.1,
    },
  );

  nextTick(() => {
    const pluginElements = document.querySelectorAll<HTMLElement>(".plugin-list-item");
    pluginElements.forEach((element) => {
      observer.value?.observe(element);
    });
  });
}

async function loadPluginDetails(pluginFileName: string) {
  if (loadedPlugins.value.has(pluginFileName)) {
    return;
  }

  loadedPlugins.value.add(pluginFileName);
}

async function togglePlugin(plugin: m_PluginInfo) {
  if (!store.currentServerId) return;

  if (!plugin.file_name.endsWith(".jar") && !plugin.file_name.endsWith(".jar.disabled")) {
    alert(i18n.t("config.not_jar_file", { file: plugin.file_name }));
    return;
  }

  try {
    await m_pluginApi.m_togglePlugin(store.currentServerId, plugin.file_name, !plugin.enabled);
    plugin.enabled = !plugin.enabled;
  } catch (e) {
    error.value = String(e);
  }
}

async function deletePlugin(plugin: m_PluginInfo) {
  const activeServerId = store.currentServerId;
  if (!activeServerId) return;
  try {
    const pluginElement = document.querySelector<HTMLElement>(
      `.plugin-list-item[data-plugin-file-name="${plugin.file_name}"]`,
    );
    if (pluginElement) {
      const originalHeight = pluginElement.offsetHeight;
      pluginElement.style.height = `${originalHeight}px`;
      pluginElement.style.flexShrink = "0";

      pluginElement.classList.add("deleting");

      setTimeout(async () => {
        await m_pluginApi.m_deletePlugin(activeServerId, plugin.file_name);
        plugins.value = plugins.value.filter((p) => p.file_name !== plugin.file_name);
        if (selectedPlugin.value?.file_name === plugin.file_name) {
          selectedPlugin.value = null;
        }
      }, 500);
    } else {
      // 如果找不到元素，直接删除
      await m_pluginApi.m_deletePlugin(activeServerId, plugin.file_name);
      plugins.value = plugins.value.filter((p) => p.file_name !== plugin.file_name);
      if (selectedPlugin.value?.file_name === plugin.file_name) {
        selectedPlugin.value = null;
      }
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function reloadPlugins() {
  if (!store.currentServerId) return;
  try {
    await m_pluginApi.m_reloadPlugins(store.currentServerId);
    await loadPlugins();
  } catch (e) {
    error.value = String(e);
  }
}

async function handlePluginClick(plugin: m_PluginInfo) {
  const activeServerId = store.currentServerId;
  if (!activeServerId) return;

  if (selectedPlugin.value?.file_name === plugin.file_name) {
    selectedPlugin.value = null;
  } else {
    if (!plugin.config_files || (plugin.config_files.length === 0 && plugin.has_config_folder)) {
      try {
        const configFiles = await m_pluginApi.m_getPluginConfigFiles(
          activeServerId,
          plugin.file_name,
          plugin.name,
        );
        const updatedPlugin = {
          ...plugin,
          config_files: configFiles,
        };
        selectedPlugin.value = updatedPlugin;
        const pluginIndex = plugins.value.findIndex((p) => p.file_name === plugin.file_name);
        if (pluginIndex !== -1) {
          plugins.value[pluginIndex] = updatedPlugin;
        }
      } catch (e) {
        console.error("Failed to load plugin config files:", e);
        selectedPlugin.value = plugin;
      }
    } else {
      selectedPlugin.value = plugin;
    }
  }
}

async function openPluginFolder(plugin: m_PluginInfo) {
  if (!store.currentServerId) return;
  const server = store.servers.find((s) => s.id === store.currentServerId);
  if (!server) return;

  const basePath = server.path.replace(/[/\\]$/, "");
  const pluginConfigPath = `${basePath}${basePath.includes("\\") ? "\\" : "/"}plugins${basePath.includes("\\") ? "\\" : "/"}${plugin.name}`;

  try {
    await systemApi.openFolder(pluginConfigPath);
  } catch (e) {
    error.value = String(e);
  }
}

async function openConfigFile(config: m_PluginConfigFile) {
  try {
    await systemApi.openFile(config.file_path);
  } catch (e) {
    error.value = String(e);
  }
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + " KB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}

const currentServer = computed(() => store.servers.find((s) => s.id === store.currentServerId));

onActivated(async () => {
  await loadProperties();
  await loadPlugins();
  if (compareMode.value && compareTargetServerId.value) {
    await loadCompareProperties();
  }
});
</script>

<template>
  <div class="config-view animate-fade-in">
    <div class="config-header">
      <div class="config-tabs-row">
        <SLTabBar v-model="activeTab" :tabs="configTabs" :level="1" />
        <div v-if="activeTab === 'properties'" class="config-properties-header-actions">
          <SLButton
            v-if="hasCompareTargets"
            size="sm"
            :variant="compareMode ? 'primary' : 'secondary'"
            class="config-compare-toggle"
            @click="handleCompareModeChange(!compareMode)"
          >
            <FileDiff :size="16" />
            {{ i18n.t("config.compare.toggle") }}
          </SLButton>
          <SLTabBar
            class="config-editor-mode-bar"
            :modelValue="editorMode"
            :tabs="editorModeTabs"
            :level="2"
            @update:modelValue="handleEditorModeChange"
          />
        </div>
      </div>
    </div>

    <div v-if="!currentServerId" class="empty-state">
      <p class="text-body">{{ i18n.t("config.no_server") }}</p>
    </div>

    <template v-else>
      <div v-if="error" class="error-banner">
        <span>{{ error }}</span>
        <button class="banner-close" @click="error = null">x</button>
      </div>
      <div v-if="successMsg" class="success-banner">
        <span>{{ i18n.t("config.saved") }}</span>
      </div>

      <template v-if="activeTab === 'properties'">
        <div v-show="editorMode === 'visual'">
          <ConfigCategories
            :categories="categories"
            :activeCategory="activeCategory"
            :searchQuery="searchQuery"
            @updateCategory="handleCategoryChange"
            @updateSearch="handleSearchUpdate"
          />

          <div v-if="loading || compareLoading" class="loading-state">
            <SLSpinner size="lg" />
            <span>{{ i18n.t("config.loading") }}</span>
          </div>

          <div v-else-if="compareMode && !hasCompareTargets" class="empty-state glass-card">
            <p class="text-caption">{{ i18n.t("config.compare.no_target_servers") }}</p>
          </div>

          <ConfigComparePanel
            v-else-if="compareMode && compareTargetServerId"
            :compareTargetServerId="compareTargetServerId"
            :compareServerOptions="compareServerOptions"
            :hasCompareTargets="hasCompareTargets"
            :compareLoading="compareLoading"
            :inlineLabel="i18n.t('config.compare.inline_label')"
            :sourceServerName="currentServer?.name || i18n.t('config.compare.source_server')"
            :targetServerName="compareTargetServer?.name || i18n.t('config.compare.target_server')"
            :differenceBadgeText="compareDifferenceBadgeText"
            :differentLabel="i18n.t('config.compare.different')"
            :noDifferencesText="i18n.t('config.compare.no_differences')"
            :rows="comparePanelRows"
            :gamemodeOptions="gamemodeOptions"
            :difficultyOptions="difficultyOptions"
            @updateCompareTargetServer="handleCompareTargetServerChange"
            @updateSourceValue="updateValue($event.key, $event.value)"
            @updateTargetValue="updateCompareTargetValue($event.key, $event.value)"
          />

          <div v-else class="config-entries">
            <div v-for="entry in filteredEntries" :key="entry.key" class="config-entry glass-card">
              <div class="entry-header">
                <div class="entry-key-row">
                  <span class="entry-key text-mono">{{ entry.key }}</span>
                </div>
                <p
                  v-if="getTranslatedPropertyDescription(entry.key)"
                  class="entry-desc text-caption"
                >
                  {{ getTranslatedPropertyDescription(entry.key) }}
                </p>
              </div>
              <div class="entry-control">
                <ConfigPropertyEditorControl
                  :propertyKey="entry.key"
                  :modelValue="editValues[entry.key]"
                  :valueType="entry.value_type"
                  :defaultValue="entry.default_value"
                  :numericError="numericFieldErrors[entry.key]"
                  :gamemodeOptions="gamemodeOptions"
                  :difficultyOptions="difficultyOptions"
                  @update:modelValue="updateValue(entry.key, $event)"
                />
              </div>
            </div>
            <div v-if="filteredEntries.length === 0 && !loading" class="empty-state">
              <p class="text-caption">{{ i18n.t("config.no_config") }}</p>
            </div>
          </div>
        </div>

        <div v-show="editorMode === 'source'">
          <div class="source-editor-wrap">
            <ConfigSourceEditor
              :modelValue="sourceDraftText"
              @update:modelValue="updateSourceDraft"
            />
            <p v-if="sourceParseError" class="source-parse-error">
              {{ sourceParseError }}
            </p>
          </div>
        </div>

        <div
          class="config-floating-actions glass-strong"
          :class="{ 'config-floating-actions--unsaved': hasUnsavedChanges }"
        >
          <div class="floating-status-wrap">
            <div class="floating-status text-caption">{{ saveStatusText }}</div>
          </div>
          <div class="floating-actions-group">
            <SLTooltip :content="reloadCurrentTooltipText">
              <SLButton
                variant="secondary"
                size="sm"
                iconOnly
                class="config-floating-icon-btn"
                @click="reloadPropertiesWithGuard"
              >
                <RefreshCw :size="16" />
              </SLButton>
            </SLTooltip>
            <SLTooltip v-if="compareMode" :content="reloadCompareTooltipText">
              <SLButton
                variant="secondary"
                size="sm"
                iconOnly
                class="config-floating-icon-btn"
                :loading="compareLoading"
                :disabled="!compareTargetServerId"
                @click="reloadComparePropertiesWithGuard"
              >
                <RefreshCw :size="16" />
              </SLButton>
            </SLTooltip>
            <SLButton
              variant="primary"
              size="sm"
              iconOnly
              class="config-floating-icon-btn"
              :class="
                hasUnsavedChanges
                  ? 'config-floating-icon-btn--unsaved'
                  : 'config-floating-icon-btn--idle'
              "
              :disabled="!hasUnsavedChanges"
              :loading="saving"
              @click="saveProperties"
            >
              <span
                class="save-icon-wrap"
                :class="{ 'save-icon-wrap--unsaved': hasUnsavedChanges && !saving }"
              >
                <Save :size="16" />
              </span>
            </SLButton>
          </div>
        </div>
      </template>

      <template v-if="activeTab === 'plugins'">
        <div class="plugins-header">
          <h3>{{ i18n.t("config.server_plugins") }}</h3>
          <div class="plugins-header-actions">
            <SLButton @click="loadPlugins" :loading="pluginsLoading" variant="secondary" size="sm">
              <RotateCcw :size="16" />
              {{ i18n.t("config.refresh_list") }}
            </SLButton>
            <SLButton
              @click="reloadPlugins"
              :loading="pluginsLoading"
              variant="danger"
              size="sm"
              class="reload-btn"
              :title="i18n.t('config.reload_plugins_warning')"
            >
              <RefreshCw :size="14" />
              {{ i18n.t("config.reload_plugins") }}
            </SLButton>
          </div>
        </div>

        <div v-if="pluginsLoading" class="loading-state">
          <SLSpinner size="lg" />
          <span>{{ i18n.t("config.loading_plugins") }}</span>
        </div>

        <div v-else class="plugins-container">
          <div v-if="plugins.length === 0" class="empty-state">
            <p class="text-caption">{{ i18n.t("config.no_plugins") }}</p>
          </div>

          <div v-else class="plugin-list-view">
            <div
              v-for="plugin in plugins"
              :key="plugin.file_name"
              class="plugin-list-item"
              :class="{
                disabled: !plugin.enabled,
                expanded: selectedPlugin?.file_name === plugin.file_name,
              }"
              :data-plugin-file-name="plugin.file_name"
              @click="handlePluginClick(plugin)"
            >
              <div class="plugin-list-icon">
                {{ plugin.name.charAt(0).toUpperCase() }}
              </div>
              <div class="plugin-list-info">
                <div class="plugin-list-header">
                  <h4>{{ plugin.name }}</h4>
                  <span class="plugin-list-version">{{ plugin.version }}</span>
                  <div v-if="plugin.has_config_folder" class="config-badge">
                    <Settings :size="14" />
                    <template v-if="selectedPlugin?.file_name === plugin.file_name">
                      {{ plugin.config_files.length }} {{ i18n.t("config.config_files_count") }}
                    </template>
                    <template v-else>
                      {{ i18n.t("config.has_config_folder") }}
                    </template>
                  </div>
                  <div v-else class="no-config-badge">
                    <FileText :size="14" />
                    {{ i18n.t("config.no_config_files") }}
                  </div>
                </div>
                <div
                  v-if="selectedPlugin?.file_name === plugin.file_name"
                  class="plugin-list-details"
                >
                  <p>{{ i18n.t("config.author") }}: {{ plugin.author }}</p>
                  <p v-if="plugin.description">{{ plugin.description }}</p>
                  <p>{{ formatFileSize(plugin.file_size) }}</p>

                  <div v-if="plugin.has_config_folder" class="plugin-config-section">
                    <div class="plugin-config-section-header">
                      <h5>{{ i18n.t("config.config_files") }}</h5>
                      <SLButton
                        size="sm"
                        variant="secondary"
                        @click.stop="openPluginFolder(plugin)"
                      >
                        <FolderOpen :size="14" />
                        {{ i18n.t("common.open_folder") }}
                      </SLButton>
                    </div>
                    <div v-if="plugin.config_files.length > 0" class="plugin-config-files-list">
                      <div
                        v-for="config in plugin.config_files"
                        :key="config.file_name"
                        class="plugin-config-file-item"
                        @click.stop="openConfigFile(config)"
                      >
                        <div class="plugin-config-file-name">{{ config.file_name }}</div>
                        <div class="plugin-config-file-type">{{ config.file_type }}</div>
                        <div class="plugin-config-file-actions">
                          <SLButton size="sm" variant="secondary">
                            <Edit :size="14" />
                            {{ i18n.t("config.open") }}
                          </SLButton>
                        </div>
                      </div>
                    </div>
                    <div v-else class="empty-state">
                      <p class="text-caption">{{ i18n.t("config.empty_config_folder") }}</p>
                    </div>
                  </div>
                </div>
              </div>
              <div class="plugin-list-actions">
                <SLSwitch
                  :modelValue="plugin.enabled"
                  @update:modelValue="togglePlugin(plugin)"
                  :title="plugin.enabled ? i18n.t('config.disable') : i18n.t('config.enable')"
                />
                <button
                  @click.stop="deletePlugin(plugin)"
                  class="icon-btn"
                  :title="i18n.t('config.delete')"
                >
                  <Trash2 :size="16" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </template>

      <SLConfirmDialog
        :visible="showDiscardConfirm"
        :title="discardConfirmTitle"
        :message="discardConfirmMessage"
        :confirmText="i18n.t('config.discard_confirm')"
        :cancelText="i18n.t('common.cancel')"
        confirmVariant="danger"
        @confirm="confirmReloadDiscard"
        @close="
          showDiscardConfirm = false;
          pendingReloadSide = null;
        "
      />

      <SLModal
        :visible="showSaveDiffModal"
        :title="i18n.t('config.diff_modal_title')"
        :width="configSaveDiffModalWidth"
        :close-on-overlay="!saving"
        @close="closeSaveDiffModal"
      >
        <div
          v-for="diffItem in pendingSaveItemsWithStats"
          :key="`${diffItem.serverId}-${diffItem.filePath}`"
          class="source-diff-block"
        >
          <div class="source-diff-title-row text-caption">
            <span class="source-diff-server">{{ diffItem.serverName }}</span>
            <SLTooltip :content="diffItem.filePath">
              <span class="source-diff-path-hint">i</span>
            </SLTooltip>
            <span
              >{{ i18n.t("config.diff_original") }} → {{ i18n.t("config.diff_after_save") }}</span
            >
            <span class="diff-count diff-count-add">+{{ diffItem.additions }}</span>
            <span class="diff-count diff-count-del">-{{ diffItem.deletions }}</span>
          </div>
          <ConfigSourceDiffView
            :original="diffItem.originalText"
            :modified="diffItem.modifiedText"
          />
        </div>
        <template #footer>
          <div class="diff-modal-actions">
            <SLButton variant="secondary" :disabled="saving" @click="closeSaveDiffModal">
              {{ i18n.t("common.cancel") }}
            </SLButton>
            <SLButton variant="primary" :loading="saving" @click="confirmSaveProperties">
              {{ i18n.t("config.confirm_save") }}
            </SLButton>
          </div>
        </template>
      </SLModal>
    </template>
  </div>
</template>
