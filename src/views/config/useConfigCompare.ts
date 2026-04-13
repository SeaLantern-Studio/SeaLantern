import { computed, ref, type Ref } from "vue";
import { configApi } from "@api/config";
import type { ConfigEntry as ConfigEntryType } from "@api/config";
import { i18n } from "@language";
import type { ServerInstance } from "@type/server";

export interface CompareEntry {
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

export interface ComparePanelControlState {
  key: string;
  value: string;
  valueType: string;
  defaultValue: string;
  numericError?: string;
}

export interface ComparePanelRow {
  key: string;
  description: string;
  different: boolean;
  onlyInSource: boolean;
  onlyInTarget: boolean;
  source: ComparePanelControlState;
  target: ComparePanelControlState;
}

interface UseConfigCompareOptions {
  currentServerId: Ref<string | null>;
  servers: Ref<ServerInstance[]>;
  sourceEntries: Ref<ConfigEntryType[]>;
  sourceValues: Ref<Record<string, string>>;
  sourceNumericFieldErrors: Ref<Record<string, string>>;
  activeCategory: Ref<string>;
  searchQuery: Ref<string>;
  getTranslatedPropertyDescription: (key: string) => string;
  setError: (message: string | null) => void;
}

function buildServerPropertiesPath(path: string) {
  const basePath = path.replace(/[/\\]$/, "");
  if (!basePath) {
    return "server.properties";
  }

  const separator = basePath.includes("\\") ? "\\" : "/";
  return `${basePath}${separator}server.properties`;
}

export function useConfigCompare(options: UseConfigCompareOptions) {
  const compareMode = ref(false);
  const compareTargetServerId = ref("");
  const compareTargetEntries = ref<ConfigEntryType[]>([]);
  const compareTargetDraftValues = ref<Record<string, string>>({});
  const compareTargetLoadedValues = ref<Record<string, string>>({});
  const compareLoading = ref(false);

  const compareTargetServer = computed(
    () => options.servers.value.find((s) => s.id === compareTargetServerId.value) || null,
  );
  const compareTargetPath = computed(() => compareTargetServer.value?.path || "");
  const compareTargetServerPropertiesPath = computed(() =>
    buildServerPropertiesPath(compareTargetPath.value),
  );
  const compareServerOptions = computed(() =>
    options.servers.value
      .filter((server) => server.id !== options.currentServerId.value)
      .map((server) => ({
        label: server.name,
        value: server.id,
      })),
  );
  const hasCompareTargets = computed(() => compareServerOptions.value.length > 0);

  const compareEntries = computed(() =>
    buildCompareEntries(
      options.sourceEntries.value,
      options.sourceValues.value,
      compareTargetEntries.value,
      compareTargetDraftValues.value,
    ),
  );

  const filteredCompareEntries = computed(() => {
    return compareEntries.value.filter((entry) => {
      const matchCat =
        options.activeCategory.value === "all" || entry.category === options.activeCategory.value;
      const matchSearch =
        !options.searchQuery.value ||
        entry.key.toLowerCase().includes(options.searchQuery.value.toLowerCase()) ||
        entry.description.toLowerCase().includes(options.searchQuery.value.toLowerCase());
      return matchCat && matchSearch;
    });
  });

  const compareDifferenceCount = computed(
    () => compareEntries.value.filter((entry) => entry.different).length,
  );
  const compareDifferenceBadgeText = computed(() =>
    i18n.t("config.compare.difference_badge", { count: compareDifferenceCount.value }),
  );

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
      description: options.getTranslatedPropertyDescription(entry.key),
      different: entry.different,
      onlyInSource: entry.onlyInSource,
      onlyInTarget: entry.onlyInTarget,
      source: {
        key: entry.key,
        value: entry.sourceValue,
        valueType: getCompareValueType(entry, "source"),
        defaultValue: getCompareDefaultValue(entry, "source"),
        numericError: options.sourceNumericFieldErrors.value[entry.key],
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

  async function applyParsedCompareTargetState(sourceText: string) {
    const parsed = await configApi.parseServerPropertiesSource(sourceText);
    compareTargetEntries.value = parsed.entries as ConfigEntryType[];
    compareTargetDraftValues.value = { ...parsed.raw };
    compareTargetLoadedValues.value = { ...parsed.raw };
  }

  async function loadCompareProperties() {
    if (!compareMode.value || !compareTargetPath.value) {
      resetCompareState(false);
      return;
    }

    compareLoading.value = true;
    options.setError(null);
    try {
      const targetResult = await configApi.readServerProperties(compareTargetPath.value);
      compareTargetEntries.value = targetResult.entries as ConfigEntryType[];
      compareTargetDraftValues.value = { ...targetResult.raw };
      compareTargetLoadedValues.value = { ...targetResult.raw };
    } catch (e) {
      options.setError(String(e));
      compareTargetEntries.value = [];
      compareTargetDraftValues.value = {};
      compareTargetLoadedValues.value = {};
    } finally {
      compareLoading.value = false;
    }
  }

  function updateCompareTargetValue(key: string, value: string | boolean | number) {
    compareTargetDraftValues.value[key] = String(value);
  }

  function handleCompareModeChange(value: boolean) {
    compareMode.value = value;
    if (!value) {
      // 这是刻意的设计：关闭对照模式时直接丢弃对照侧草稿，回到单列编辑体验。
      resetCompareState(false);
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

  function resetCompareState(includeMode = true) {
    if (includeMode) {
      compareMode.value = false;
    }
    compareTargetServerId.value = "";
    compareTargetEntries.value = [];
    compareTargetDraftValues.value = {};
    compareTargetLoadedValues.value = {};
  }

  return {
    compareMode,
    compareTargetServerId,
    compareTargetEntries,
    compareTargetDraftValues,
    compareTargetLoadedValues,
    compareLoading,
    compareTargetServer,
    compareTargetPath,
    compareTargetServerPropertiesPath,
    compareServerOptions,
    hasCompareTargets,
    compareDifferenceBadgeText,
    compareTargetNumericFieldErrors,
    comparePanelRows,
    loadCompareProperties,
    applyParsedCompareTargetState,
    updateCompareTargetValue,
    handleCompareModeChange,
    handleCompareTargetServerChange,
    resetCompareState,
  };
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
