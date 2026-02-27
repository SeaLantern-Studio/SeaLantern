import { computed, onMounted, ref, watch, type Ref } from "vue";
import { downloadApi } from "@api/downloader";
import { serverApi } from "@api/server";
import { systemApi } from "@api/system";
import { STARTER_SERVER_JAR_NAME } from "@components/views/create/constants";
import { joinPath, normalizePathForMatch } from "@components/views/create/startupUtils";
import { i18n } from "@language";

type SourceType = "archive" | "folder" | "";

interface UseStarterCoreDownloadOptions {
  sourcePath: Ref<string>;
  sourceType: Ref<SourceType>;
  clearError: () => void;
  showError: (message: string) => void;
}

export function useStarterCoreDownload(options: UseStarterCoreDownloadOptions) {
  const { sourcePath, sourceType, clearError, showError } = options;

  const coreDownloadTargetPath = ref("");
  const coreDownloadCoreType = ref("");
  const coreDownloadMcVersion = ref("");
  const coreDownloadCoreOptions = ref<string[]>([]);
  const coreDownloadMcVersionOptions = ref<string[]>([]);
  const starterOptionsLoading = ref(false);
  const starterOptionsLoadFailed = ref(false);
  const coreDownloadLaunching = ref(false);
  const coreDownloadResolvedSourcePath = ref("");
  const coreDownloadResolvedSavePath = ref("");
  const lastCoreDownloadTargetPath = ref("");
  const lastCoreDownloadSavePath = ref("");
  const lastCoreDownloadCoreType = ref("");
  const lastCoreDownloadMcVersion = ref("");

  const {
    taskInfo: coreDownloadTaskInfo,
    start: startCoreDownload,
    reset: resetCoreDownload,
    errorMessage: coreDownloadError,
  } = downloadApi.useDownload();

  const coreTypeSelectOptions = computed(() =>
    coreDownloadCoreOptions.value.map((value) => ({
      value,
      label: value,
    })),
  );
  const mcVersionSelectOptions = computed(() =>
    coreDownloadMcVersionOptions.value.map((value) => ({
      value,
      label: value,
    })),
  );
  const isCoreDownloading = computed(
    () =>
      coreDownloadLaunching.value ||
      (coreDownloadTaskInfo.id !== "" && !coreDownloadTaskInfo.isFinished),
  );
  const isCoreDownloadCompleted = computed(
    () => coreDownloadTaskInfo.isFinished && coreDownloadTaskInfo.status === "Completed",
  );
  const hasStarterDownloadOptions = computed(
    () => coreDownloadCoreOptions.value.length > 0 && coreDownloadMcVersionOptions.value.length > 0,
  );
  const isStarterOptionsUnavailable = computed(
    () =>
      !starterOptionsLoading.value &&
      (starterOptionsLoadFailed.value || !hasStarterDownloadOptions.value),
  );
  const starterOptionsUnavailableMessage = computed(() =>
    isStarterOptionsUnavailable.value ? i18n.t("create.source_method_two_unavailable") : "",
  );
  const isStarterDownloadControlDisabled = computed(
    () =>
      isCoreDownloading.value || starterOptionsLoading.value || isStarterOptionsUnavailable.value,
  );

  function markStarterOptionsUnavailable(error?: unknown) {
    starterOptionsLoadFailed.value = true;
    coreDownloadCoreOptions.value = [];
    coreDownloadMcVersionOptions.value = [];

    if (error !== undefined) {
      console.error("Failed to load starter download options:", error);
    }
    showError(i18n.t("create.source_method_two_unavailable"));
  }

  onMounted(async () => {
    starterOptionsLoading.value = true;
    starterOptionsLoadFailed.value = false;
    try {
      const starterOptions = await serverApi.getStarterDownloadOptions();
      const coreTypeOptions = starterOptions.coreTypeOptions ?? [];
      const mcVersionOptions = starterOptions.mcVersionOptions ?? [];
      coreDownloadCoreOptions.value = coreTypeOptions;
      coreDownloadMcVersionOptions.value = mcVersionOptions;

      if (coreTypeOptions.length === 0 || mcVersionOptions.length === 0) {
        markStarterOptionsUnavailable();
      }
    } catch (error) {
      markStarterOptionsUnavailable(error);
    } finally {
      starterOptionsLoading.value = false;
    }
  });

  watch(coreDownloadError, (value) => {
    if (value) {
      showError(value);
    }
  });

  watch(
    [coreDownloadTargetPath, coreDownloadCoreType, coreDownloadMcVersion],
    ([target, coreType, mcVersion], previous) => {
      if (!previous) {
        return;
      }
      const [prevTarget, prevCoreType, prevMcVersion] = previous;
      if (isCoreDownloading.value) {
        return;
      }
      if (target === prevTarget && coreType === prevCoreType && mcVersion === prevMcVersion) {
        return;
      }
      if (coreDownloadTaskInfo.id === "" && !isCoreDownloadCompleted.value) {
        return;
      }

      resetCoreDownload();
      coreDownloadResolvedSourcePath.value = "";
      coreDownloadResolvedSavePath.value = "";
    },
  );

  watch(isCoreDownloadCompleted, (completed) => {
    if (!completed) {
      return;
    }
    const resolvedPath = coreDownloadResolvedSourcePath.value.trim();
    const resolvedSavePath = coreDownloadResolvedSavePath.value.trim();
    if (!resolvedPath || !resolvedSavePath) {
      return;
    }

    lastCoreDownloadTargetPath.value = resolvedPath;
    lastCoreDownloadSavePath.value = resolvedSavePath;
    lastCoreDownloadCoreType.value = coreDownloadCoreType.value;
    lastCoreDownloadMcVersion.value = coreDownloadMcVersion.value;

    // Download mode uses a folder source for the rest of the create flow.
    sourcePath.value = resolvedPath;
    sourceType.value = "folder";
  });

  async function pickCoreDownloadTargetPath() {
    const selected = await systemApi.pickFolder();
    if (selected) {
      coreDownloadTargetPath.value = selected;
    }
  }

  async function handleCoreDownload() {
    clearError();
    if (starterOptionsLoading.value || isStarterOptionsUnavailable.value) {
      showError(i18n.t("create.source_method_two_unavailable"));
      return;
    }
    if (
      !coreDownloadTargetPath.value.trim() ||
      !coreDownloadCoreType.value ||
      !coreDownloadMcVersion.value
    ) {
      showError(i18n.t("create.source_method_two_required"));
      return;
    }

    try {
      coreDownloadLaunching.value = true;
      const downloadUrl = await serverApi.resolveStarterDownloadUrl(
        coreDownloadCoreType.value,
        coreDownloadMcVersion.value,
      );

      const normalizedTargetPath = coreDownloadTargetPath.value.trim().replace(/[\\/]+$/, "");
      const savePath = joinPath(normalizedTargetPath, STARTER_SERVER_JAR_NAME);

      const sameDownloadFolder =
        normalizePathForMatch(normalizedTargetPath) ===
        normalizePathForMatch(lastCoreDownloadTargetPath.value);
      const sameCoreSelection =
        coreDownloadCoreType.value === lastCoreDownloadCoreType.value &&
        coreDownloadMcVersion.value === lastCoreDownloadMcVersion.value;
      const previousCorePath = lastCoreDownloadSavePath.value.trim();

      if (sameDownloadFolder && !sameCoreSelection && previousCorePath) {
        await systemApi.removeFileIfExists(previousCorePath);
      }

      resetCoreDownload();
      coreDownloadResolvedSourcePath.value = normalizedTargetPath;
      coreDownloadResolvedSavePath.value = savePath;
      await startCoreDownload({
        url: downloadUrl,
        savePath,
      });
    } catch (error) {
      showError(String(error));
    } finally {
      coreDownloadLaunching.value = false;
    }
  }

  return {
    coreDownloadTargetPath,
    coreDownloadCoreType,
    coreDownloadMcVersion,
    coreDownloadTaskInfo,
    coreTypeSelectOptions,
    mcVersionSelectOptions,
    isCoreDownloading,
    isCoreDownloadCompleted,
    isStarterDownloadControlDisabled,
    starterOptionsUnavailableMessage,
    pickCoreDownloadTargetPath,
    handleCoreDownload,
  };
}
