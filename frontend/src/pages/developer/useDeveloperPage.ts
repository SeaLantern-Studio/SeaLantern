import { computed, reactive, shallowRef, watch } from "vue";
import { useRouter } from "vue-router";
import { useDeveloperTools } from "@composables/useDeveloperTools";
import { useSettingsStore } from "@stores/settingsStore";
import { i18n } from "@language";
import { NEXT_SETTINGS_ROUTE_NAME } from "@src/router/pageMeta";
import type { WorkbenchFactItem } from "@src/components/workbench/WorkbenchFactGrid.vue";
import type { DeveloperBannerTone } from "@composables/useDeveloperTools";

export type DeveloperSectionId = "overview" | "logs" | "tools" | "other";

export interface DeveloperSectionItem {
  id: DeveloperSectionId;
  label: string;
  description: string;
}

export function useDeveloperPage() {
  const router = useRouter();
  const settingsStore = useSettingsStore();

  const activeSectionId = shallowRef<DeveloperSectionId>("overview");
  const developerEnabled = computed(() => settingsStore.settings.developer_mode);

  const sectionItems = computed<readonly DeveloperSectionItem[]>(() => [
    {
      id: "overview",
      label: i18n.t("developer.next.sections.overview.label"),
      description: i18n.t("developer.next.sections.overview.description"),
    },
    {
      id: "logs",
      label: i18n.t("developer.next.sections.logs.label"),
      description: i18n.t("developer.next.sections.logs.description"),
    },
    {
      id: "tools",
      label: i18n.t("developer.next.sections.tools.label"),
      description: i18n.t("developer.next.sections.tools.description"),
    },
    {
      id: "other",
      label: i18n.t("developer.next.sections.other.label"),
      description: i18n.t("developer.next.sections.other.description"),
    },
  ]);

  const {
    logLines,
    filteredLogCount,
    totalLogCount,
    hasLogEntries,
    memoryDisplayPrecision,
    selectedLogLevel,
    selectedLogModule,
    logLevelOptions,
    logModuleOptions,
    systemInfo,
    version,
    loadingLogs,
    loadingSystem,
    exportingLogs,
    clearingLogs,
    downloadingUpdate,
    triggeringCrash,
    updateUrl,
    activeBanner,
    logError,
    systemError,
    isBrowserMode,
    canTriggerCrash,
    refreshLogs,
    refreshSystemInfo,
    copyLogs,
    exportLogsToFile,
    clearAllLogs,
    copySystemSummary,
    downloadUpdateFromUrl,
    triggerCrashTest,
    showTestSuccessToast,
    showTestErrorToast,
    showTestWarningToast,
    showTestInfoToast,
    showTestBanner,
    clearTestBanner,
  } = useDeveloperTools({
    enabled: () => developerEnabled.value,
  });

  const consoleDisplay = reactive({
    fontSize: computed(() => settingsStore.settings.console_font_size || 12),
    fontFamily: computed(() => settingsStore.settings.console_font_family || ""),
    letterSpacing: computed(() => settingsStore.settings.console_letter_spacing || 0),
    maxLogLines: computed(() => Math.max(100, settingsStore.settings.max_log_lines || 1000)),
  });

  const currentSection = computed(() => {
    return sectionItems.value.find((item) => item.id === activeSectionId.value) ?? sectionItems.value[0];
  });

  const summaryFacts = computed<WorkbenchFactItem[]>(() => {
    const environment = isBrowserMode.value
      ? i18n.t("developer.next.summary.browser")
      : i18n.t("developer.next.summary.desktop");

    return [
      {
        label: i18n.t("developer.app_version"),
        value: version.value,
      },
      {
        label: i18n.t("developer.next.summary.environment"),
        value: environment,
      },
      {
        label: i18n.t("developer.next.summary.log_count"),
        value: String(totalLogCount.value),
        tone: totalLogCount.value > 0 ? "warning" : "default",
      },
      {
        label: i18n.t("developer.next.summary.system_status"),
        value: systemInfo.value ? i18n.t("developer.next.summary.ready") : i18n.t("developer.next.summary.waiting"),
        tone: systemInfo.value ? "success" : "default",
      },
    ];
  });

  function selectSection(sectionId: string): void {
    if (sectionId === "overview" || sectionId === "logs" || sectionId === "tools" || sectionId === "other") {
      activeSectionId.value = sectionId;
    }
  }

  function setSelectedLogLevel(value: string): void {
    selectedLogLevel.value = value;
  }

  function setSelectedLogModule(value: string): void {
    selectedLogModule.value = value;
  }

  function setUpdateUrl(value: string): void {
    updateUrl.value = value;
  }

  function triggerToastTest(kind: "success" | "error" | "warning" | "info"): void {
    switch (kind) {
      case "success":
        showTestSuccessToast();
        break;
      case "error":
        showTestErrorToast();
        break;
      case "warning":
        showTestWarningToast();
        break;
      case "info":
        showTestInfoToast();
        break;
    }
  }

  function triggerBannerTest(tone: DeveloperBannerTone): void {
    showTestBanner(tone);
  }

  watch(
    developerEnabled,
    (enabled) => {
      if (!enabled) {
        void router.replace({ name: NEXT_SETTINGS_ROUTE_NAME, hash: "#developer-management" });
      }
    },
    { flush: "sync" },
  );

  return {
    activeSectionId,
    currentSection,
    sectionItems,
    developerEnabled,
    summaryFacts,
    logLines,
    filteredLogCount,
    totalLogCount,
    hasLogEntries,
    memoryDisplayPrecision,
    selectedLogLevel,
    selectedLogModule,
    logLevelOptions,
    logModuleOptions,
    systemInfo,
    version,
    loadingLogs,
    loadingSystem,
    exportingLogs,
    clearingLogs,
    downloadingUpdate,
    triggeringCrash,
    updateUrl,
    activeBanner,
    logError,
    systemError,
    isBrowserMode,
    canTriggerCrash,
    refreshLogs,
    refreshSystemInfo,
    copyLogs,
    exportLogsToFile,
    clearAllLogs,
    copySystemSummary,
    downloadUpdateFromUrl,
    triggerCrashTest,
    selectSection,
    setSelectedLogLevel,
    setSelectedLogModule,
    setUpdateUrl,
    triggerToastTest,
    triggerBannerTest,
    clearTestBanner,
    consoleDisplay,
  };
}
