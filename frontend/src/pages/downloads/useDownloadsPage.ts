import { computed, onMounted, reactive, shallowRef, watch } from "vue";
import { useRouter } from "vue-router";
import { systemApi } from "@api/system";
import { downloadApi, downloadServerApi, type DownloadLink } from "@api/downloader";
import { useCreateServerDraftStore } from "@stores/createServerDraft";
import { useGlobalMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { NEXT_SERVER_CREATE_ROUTE_NAME } from "@src/router/pageMeta";
import type { WorkbenchFactItem } from "@src/components/workbench/WorkbenchFactGrid.vue";

let downloadsPageLoadedOnce = false;

export type DownloadsSectionId = "tasks" | "server" | "file";
export type DownloadTaskKind = "server" | "file";
export type DownloadTaskStateTone = "primary" | "success" | "warning" | "error";

export interface DownloadsSectionItem {
  id: DownloadsSectionId;
  label: string;
  description: string;
}

export interface DownloadTaskEntry {
  id: string;
  kind: DownloadTaskKind;
  title: string;
  detail: string;
  savePath: string;
  createdAt: number;
  progress: number;
  downloaded: number;
  totalSize: number;
  statusText: string;
  tone: DownloadTaskStateTone;
  isFinished: boolean;
  errorMessage: string | null;
  canCreateInstance: boolean;
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

function buildSavePath(dir: string, filename: string): string {
  const normalizedDir = normalizePath(dir).replace(/[\\/]+$/, "");
  const normalizedFile = filename.replace(/^[\\/]+/, "");
  return normalizedDir && normalizedFile
    ? `${normalizedDir}/${normalizedFile}`
    : normalizedDir || normalizedFile;
}

function createTaskStatusSnapshot(
  taskInfo: { status: unknown; isFinished: boolean },
  errorMessage: string | null,
): Pick<DownloadTaskEntry, "statusText" | "tone" | "isFinished" | "errorMessage"> {
  if (errorMessage) {
    return {
      statusText: i18n.t("downloads.next.tasks.status_failed"),
      tone: "error",
      isFinished: true,
      errorMessage,
    };
  }

  if (taskInfo.isFinished) {
    return {
      statusText: i18n.t("downloads.next.tasks.status_finished"),
      tone: "success",
      isFinished: true,
      errorMessage: null,
    };
  }

  if (taskInfo.status === "Pending") {
    return {
      statusText: i18n.t("downloads.next.tasks.status_pending"),
      tone: "warning",
      isFinished: false,
      errorMessage: null,
    };
  }

  return {
    statusText: i18n.t("downloads.next.tasks.status_downloading"),
    tone: "primary",
    isFinished: false,
    errorMessage: null,
  };
}

function validateThreadCount(value: string): string | null {
  if (!value.trim()) {
    return i18n.t("download-file.thread_count_empty");
  }
  if (!/^-?\d+$/.test(value.trim())) {
    return i18n.t("download-file.thread_count_invalid");
  }
  if (!/^[1-9]\d*$/.test(value.trim())) {
    return i18n.t("download-file.thread_count_positive");
  }
  if (Number.parseInt(value, 10) > 256) {
    return i18n.t("download-file.thread_count_too_big");
  }
  return null;
}

function ensureValidUrl(url: string): boolean {
  try {
    const parsed = new URL(url.trim());
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export function useDownloadsPage() {
  const router = useRouter();
  const globalMessage = useGlobalMessage();
  const createServerDraftStore = useCreateServerDraftStore();

  const sectionItems = computed<readonly DownloadsSectionItem[]>(() => [
    {
      id: "tasks",
      label: i18n.t("downloads.next.sections.tasks.label"),
      description: i18n.t("downloads.next.sections.tasks.description"),
    },
    {
      id: "server",
      label: i18n.t("downloads.next.sections.server.label"),
      description: i18n.t("downloads.next.sections.server.description"),
    },
    {
      id: "file",
      label: i18n.t("downloads.next.sections.file.label"),
      description: i18n.t("downloads.next.sections.file.description"),
    },
  ]);

  const activeSectionId = shallowRef<DownloadsSectionId>("tasks");
  const bootstrapping = shallowRef(!downloadsPageLoadedOnce);
  const loadingServerTypes = shallowRef(false);
  const loadingVersions = shallowRef(false);
  const loadingDownloadInfo = shallowRef(false);
  const submittingServer = shallowRef(false);
  const submittingFile = shallowRef(false);
  const pageError = shallowRef<string | null>(null);

  const savePaths = reactive({
    serverDir: "",
    fileDir: "",
  });

  const fileForm = reactive({
    url: "",
    filename: "",
    threadCount: "32",
  });

  const serverForm = reactive({
    selectedType: "",
    selectedVersion: "",
    filename: "server.jar",
    threadCount: "32",
  });

  const serverTypes = shallowRef<string[]>([]);
  const versions = shallowRef<string[]>([]);
  const downloadInfo = shallowRef<DownloadLink | null>(null);
  const activeTaskId = shallowRef<string | null>(null);
  const taskHistory = shallowRef<DownloadTaskEntry[]>([]);

  const {
    taskInfo,
    start: startTask,
    reset: resetTask,
    errorMessage: taskError,
  } = downloadApi.useDownload();

  const currentSection = computed(() => {
    return (
      sectionItems.value.find((item) => item.id === activeSectionId.value) ?? sectionItems.value[0]
    );
  });

  const serverTypeOptions = computed(() =>
    serverTypes.value.map((type) => ({ label: type, value: type })),
  );

  const versionOptions = computed(() =>
    [...versions.value]
      .toSorted((a, b) => {
        const aParts = a.split(".").map(Number);
        const bParts = b.split(".").map(Number);
        for (let index = 0; index < Math.max(aParts.length, bParts.length); index += 1) {
          const diff = (bParts[index] || 0) - (aParts[index] || 0);
          if (diff !== 0) {
            return diff;
          }
        }
        return 0;
      })
      .map((version) => ({ label: version, value: version })),
  );

  const currentTask = computed(() => {
    if (!activeTaskId.value) {
      return null;
    }
    return taskHistory.value.find((entry) => entry.id === activeTaskId.value) ?? null;
  });

  const recentTasks = computed(() => {
    if (!activeTaskId.value) {
      return taskHistory.value;
    }
    return taskHistory.value.filter((entry) => entry.id !== activeTaskId.value);
  });

  const hasAnyTask = computed(() => taskHistory.value.length > 0);
  const hasActiveTask = computed(() => Boolean(currentTask.value));
  const isTaskRunning = computed(() => Boolean(currentTask.value && !currentTask.value.isFinished));
  const fileSavePath = computed(() => buildSavePath(savePaths.fileDir, fileForm.filename));
  const serverSavePath = computed(() => buildSavePath(savePaths.serverDir, serverForm.filename));

  const summaryFacts = computed<WorkbenchFactItem[]>(() => [
    {
      label: i18n.t("downloads.next.summary.total_tasks"),
      value: String(taskHistory.value.length),
    },
    {
      label: i18n.t("downloads.next.summary.active_tasks"),
      value: hasActiveTask.value && !currentTask.value?.isFinished ? "1" : "0",
      tone: hasActiveTask.value && !currentTask.value?.isFinished ? "warning" : "default",
    },
    {
      label: i18n.t("downloads.next.summary.completed_tasks"),
      value: String(
        taskHistory.value.filter((entry) => entry.isFinished && !entry.errorMessage).length,
      ),
      tone: taskHistory.value.some((entry) => entry.isFinished && !entry.errorMessage)
        ? "success"
        : "default",
    },
    {
      label: i18n.t("downloads.next.summary.failed_tasks"),
      value: String(taskHistory.value.filter((entry) => Boolean(entry.errorMessage)).length),
      tone: taskHistory.value.some((entry) => Boolean(entry.errorMessage)) ? "danger" : "default",
    },
  ]);

  const canSubmitServer = computed(() => {
    return Boolean(
      !submittingServer.value &&
      serverForm.selectedType &&
      serverForm.selectedVersion &&
      savePaths.serverDir.trim() &&
      serverForm.filename.trim() &&
      downloadInfo.value?.url &&
      !validateThreadCount(serverForm.threadCount),
    );
  });

  const canSubmitFile = computed(() => {
    return Boolean(
      !submittingFile.value &&
      ensureValidUrl(fileForm.url) &&
      savePaths.fileDir.trim() &&
      fileForm.filename.trim() &&
      !validateThreadCount(fileForm.threadCount),
    );
  });

  function selectSection(sectionId: string): void {
    if (sectionId === "tasks" || sectionId === "server" || sectionId === "file") {
      activeSectionId.value = sectionId;
    }
  }

  function syncTaskSnapshot(): void {
    if (!activeTaskId.value) {
      return;
    }

    const snapshot = createTaskStatusSnapshot(taskInfo, taskError.value);
    taskHistory.value = taskHistory.value.map((entry) => {
      if (entry.id !== activeTaskId.value) {
        return entry;
      }

      return {
        ...entry,
        progress: taskInfo.progress,
        downloaded: taskInfo.downloaded,
        totalSize: taskInfo.totalSize,
        statusText: snapshot.statusText,
        tone: snapshot.tone,
        isFinished: snapshot.isFinished,
        errorMessage: snapshot.errorMessage,
      };
    });
  }

  function pushTask(entry: DownloadTaskEntry): void {
    activeTaskId.value = entry.id;
    taskHistory.value = [entry, ...taskHistory.value.filter((item) => item.id !== entry.id)].slice(
      0,
      8,
    );
    syncTaskSnapshot();
  }

  function fillFilenameFromUrl(): void {
    if (!ensureValidUrl(fileForm.url)) {
      return;
    }
    const parsed = new URL(fileForm.url.trim());
    const segments = parsed.pathname.split("/");
    const nextFilename = segments[segments.length - 1] || "";
    if (nextFilename) {
      fileForm.filename = nextFilename;
    }
  }

  async function loadDefaultPaths(): Promise<void> {
    try {
      const defaultPath = normalizePath(await systemApi.getDefaultRunPath());
      if (!savePaths.serverDir) {
        savePaths.serverDir = defaultPath;
      }
      if (!savePaths.fileDir) {
        savePaths.fileDir = defaultPath;
      }
    } catch {
      // keep manual path selection available
    }
  }

  async function loadServerTypes(): Promise<void> {
    loadingServerTypes.value = true;
    pageError.value = null;
    try {
      const nextTypes = await downloadServerApi.getServerTypes();
      serverTypes.value = nextTypes;
      if (!serverForm.selectedType && nextTypes.length > 0) {
        await updateSelectedType(nextTypes[0]);
      }
    } catch (error) {
      pageError.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingServerTypes.value = false;
    }
  }

  async function loadVersionsByType(serverType: string): Promise<void> {
    if (!serverType) {
      return;
    }
    loadingVersions.value = true;
    pageError.value = null;
    versions.value = [];
    downloadInfo.value = null;
    serverForm.selectedVersion = "";
    try {
      const nextVersions = await downloadServerApi.getVersionsByType(serverType);
      versions.value = nextVersions;
      if (nextVersions.length > 0) {
        await updateSelectedVersion(nextVersions[nextVersions.length - 1]);
      }
    } catch (error) {
      pageError.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingVersions.value = false;
    }
  }

  async function loadDownloadInfo(serverType: string, version: string): Promise<void> {
    if (!serverType || !version) {
      return;
    }
    loadingDownloadInfo.value = true;
    pageError.value = null;
    downloadInfo.value = null;
    try {
      const nextInfo = await downloadServerApi.getDownloadInfo(serverType, version);
      downloadInfo.value = nextInfo;
      serverForm.filename = nextInfo.fileName;
    } catch (error) {
      pageError.value = error instanceof Error ? error.message : String(error);
    } finally {
      loadingDownloadInfo.value = false;
    }
  }

  async function updateSelectedType(value: string): Promise<void> {
    serverForm.selectedType = value;
    await loadVersionsByType(value);
  }

  async function updateSelectedVersion(value: string): Promise<void> {
    serverForm.selectedVersion = value;
    await loadDownloadInfo(serverForm.selectedType, value);
  }

  async function pickServerDirectory(): Promise<void> {
    try {
      const selected = await systemApi.pickFolder();
      if (selected) {
        savePaths.serverDir = normalizePath(selected);
      }
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function pickFileDirectory(): Promise<void> {
    try {
      const selected = await systemApi.pickFolder();
      if (selected) {
        savePaths.fileDir = normalizePath(selected);
      }
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function submitServerDownload(): Promise<void> {
    if (!canSubmitServer.value || !downloadInfo.value) {
      return;
    }
    const validationError = validateThreadCount(serverForm.threadCount);
    if (validationError) {
      globalMessage.error(validationError);
      return;
    }
    submittingServer.value = true;
    resetTask();
    try {
      const savePath = serverSavePath.value;
      await startTask({
        url: downloadInfo.value.url,
        savePath,
        threadCount: Number.parseInt(serverForm.threadCount, 10),
      });
      pushTask({
        id: taskInfo.id,
        kind: "server",
        title: `${serverForm.selectedType} ${serverForm.selectedVersion}`,
        detail: serverForm.filename,
        savePath,
        createdAt: Date.now(),
        progress: taskInfo.progress,
        downloaded: taskInfo.downloaded,
        totalSize: taskInfo.totalSize,
        ...createTaskStatusSnapshot(taskInfo, taskError.value),
        canCreateInstance: true,
      });
      activeSectionId.value = "tasks";
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    } finally {
      submittingServer.value = false;
      syncTaskSnapshot();
    }
  }

  async function submitFileDownload(): Promise<void> {
    if (!canSubmitFile.value) {
      return;
    }
    const validationError = validateThreadCount(fileForm.threadCount);
    if (validationError) {
      globalMessage.error(validationError);
      return;
    }
    submittingFile.value = true;
    resetTask();
    try {
      const savePath = fileSavePath.value;
      await startTask({
        url: fileForm.url.trim(),
        savePath,
        threadCount: Number.parseInt(fileForm.threadCount, 10),
      });
      pushTask({
        id: taskInfo.id,
        kind: "file",
        title: fileForm.filename,
        detail: fileForm.url.trim(),
        savePath,
        createdAt: Date.now(),
        progress: taskInfo.progress,
        downloaded: taskInfo.downloaded,
        totalSize: taskInfo.totalSize,
        ...createTaskStatusSnapshot(taskInfo, taskError.value),
        canCreateInstance: false,
      });
      activeSectionId.value = "tasks";
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    } finally {
      submittingFile.value = false;
      syncTaskSnapshot();
    }
  }

  async function cancelActiveTask(): Promise<void> {
    if (!taskInfo.id || !activeTaskId.value) {
      return;
    }
    try {
      await downloadApi.cancelDownloadTask(taskInfo.id);
      taskHistory.value = taskHistory.value.map((entry) => {
        if (entry.id !== activeTaskId.value) {
          return entry;
        }
        return {
          ...entry,
          statusText: i18n.t("downloads.next.tasks.status_cancelled"),
          tone: "warning",
          isFinished: true,
          errorMessage: i18n.t("downloads.next.tasks.cancelled_detail"),
        };
      });
      resetTask();
    } catch (error) {
      globalMessage.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function goToCreateInstance(task: DownloadTaskEntry): Promise<void> {
    if (!task.canCreateInstance || !task.isFinished || task.errorMessage) {
      return;
    }
    createServerDraftStore.setDraft({
      sourcePath: task.savePath,
      sourceType: "archive",
    });
    await router.push({ name: NEXT_SERVER_CREATE_ROUTE_NAME });
  }

  watch(
    () =>
      [
        taskInfo.id,
        taskInfo.progress,
        taskInfo.downloaded,
        taskInfo.totalSize,
        taskInfo.isFinished,
        taskError.value,
      ] as const,
    () => {
      syncTaskSnapshot();
    },
  );

  onMounted(async () => {
    if (!downloadsPageLoadedOnce) {
      bootstrapping.value = true;
    }

    try {
      await Promise.all([loadDefaultPaths(), loadServerTypes()]);
      downloadsPageLoadedOnce = true;
    } finally {
      bootstrapping.value = false;
    }
  });

  return {
    bootstrapping,
    loadingServerTypes,
    loadingVersions,
    loadingDownloadInfo,
    submittingServer,
    submittingFile,
    pageError,
    activeSectionId,
    currentSection,
    sectionItems,
    savePaths,
    fileForm,
    serverForm,
    serverTypeOptions,
    versionOptions,
    downloadInfo,
    currentTask,
    recentTasks,
    hasAnyTask,
    hasActiveTask,
    isTaskRunning,
    summaryFacts,
    fileSavePath,
    serverSavePath,
    canSubmitServer,
    canSubmitFile,
    selectSection,
    fillFilenameFromUrl,
    updateSelectedType,
    updateSelectedVersion,
    pickServerDirectory,
    pickFileDirectory,
    submitServerDownload,
    submitFileDownload,
    cancelActiveTask,
    goToCreateInstance,
  };
}
