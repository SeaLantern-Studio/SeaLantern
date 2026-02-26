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

  onMounted(async () => {
    try {
      const starterOptions = await serverApi.getStarterDownloadOptions();
      coreDownloadCoreOptions.value = starterOptions.coreTypeOptions;
      coreDownloadMcVersionOptions.value = starterOptions.mcVersionOptions;
    } catch (error) {
      showError(String(error));
    }
  });

  watch(coreDownloadError, (value) => {
    if (value) {
      showError(value);
    }
  });

  watch(
    [coreDownloadTargetPath, coreDownloadCoreType, coreDownloadMcVersion],
    ([target, coreType, mcVersion], previous = []) => {
      if (!previous.length) {
        return;
      }
      const [prevTarget, prevCoreType, prevMcVersion] = previous as [string, string, string];
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
    pickCoreDownloadTargetPath,
    handleCoreDownload,
  };
}
