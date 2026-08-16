import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { downloadApi, type DownloadTaskInfo, type DownloadOptions } from "@api/downloader";
import { i18n } from "@language";

/**
 * 下载任务来源标识
 * - server: 下载服务端
 * - file: 自定义下载
 */
export type TaskOrigin = "server" | "file";

/** 完成态自动消失延时，用户没查看的情况 */
const AUTO_DISMISS_MS = 30_000;
/** 用户看过了再关面板，8s 后撅掉 */
const VIEWED_DISMISS_MS = 8_000;

/** 轮询间隔 */
const POLL_INTERVAL_MS = 800;

/**
 * 全局下载任务 store
 * 把原来 downloader.ts 里 useDownload() 的局部 taskInfo 提升成全局状态，
 * 这样任意页面都能读下载进度，顶部任务球就靠这个
 */
export const useDownloadStore = defineStore("download", () => {
  // 当前任务，单任务模型，预留多任务扩展，null 表示没活儿
  const currentTask = ref<DownloadTaskInfo | null>(null);
  const taskOriginTab = ref<TaskOrigin | null>(null);
  // 展示用元信息，详情面板要用的
  const filename = ref("");
  const savePath = ref("");
  // 用户看没看过完成结果，控制任务球自动消失
  const viewed = ref(false);
  // 详情面板开没开
  const panelOpen = ref(false);
  // 看过之后关面板的宽限期，8s 内还显示，然后撅掉
  const viewedGracePeriod = ref(false);
  // 瞬时速度，字节/秒
  const speed = ref(0);

  let pollTimer: number | null = null;
  let dismissTimer: number | null = null;
  // 看过之后关面板的延时消失计时器
  let viewedDismissTimer: number | null = null;
  // 会话计数，防止旧轮询往新任务里写状态
  let activeSession = 0;
  // 上次轮询的已下载量和时间戳，算瞬时速度用
  let lastDownloaded = 0;
  let lastTimestamp = 0;

  // ========== Getters ==========
  /** 正在下载中，有任务 ID 且没完成 */
  const isDownloading = computed(
    () => !!currentTask.value && currentTask.value.id !== "" && !currentTask.value.is_finished,
  );

  /** 任务结束了没，完成或出错都算 */
  const isFinished = computed(() => !!currentTask.value?.is_finished);

  /** 错误信息字符串，没错误返回 null */
  const taskError = computed(() => {
    if (!currentTask.value) return null;
    const s = currentTask.value.status;
    if (typeof s === "object" && "Error" in s) return s.Error;
    return null;
  });

  /** 出错终止，isFinished 且有错误 */
  const isError = computed(() => isFinished.value && !!taskError.value);

  /** 正常完成，isFinished 且没错误 */
  const isCompleted = computed(() => isFinished.value && !taskError.value);

  /** 下载进度百分比 0-100 */
  const progress = computed(() => currentTask.value?.progress ?? 0);

  /** 状态文本，下载中/已完成/失败，给按钮等场景用 */
  const statusLabel = computed(() => {
    if (taskError.value) return i18n.t("download-file.failed");
    if (isCompleted.value) return i18n.t("downloadServerView.status.finished");
    return i18n.t("download-file.downloading");
  });

  /**
   * 任务球显不显示：
   * - 没任务 → 撅掉
   * - 完成/出错且看过了、面板关了、不在宽限期 → 撅掉
   * 其他情况都显示
   */
  const shouldShowPill = computed(() => {
    if (!currentTask.value || currentTask.value.id === "") return false;
    if (isFinished.value && viewed.value && !panelOpen.value && !viewedGracePeriod.value)
      return false;
    return true;
  });

  // ========== 内部工具 ==========
  /** 撅掉进度轮询 */
  function stopPolling() {
    if (pollTimer !== null) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  /** 清掉所有消失计时器，自动消失和看过后延时消失一起撅了，顺便退出宽限期 */
  function clearDismissTimer() {
    if (dismissTimer !== null) {
      clearTimeout(dismissTimer);
      dismissTimer = null;
    }
    if (viewedDismissTimer !== null) {
      clearTimeout(viewedDismissTimer);
      viewedDismissTimer = null;
    }
    viewedGracePeriod.value = false;
  }

  /** 启动看过后的延时消失计时 */
  function startViewedDismissTimer() {
    if (viewedDismissTimer !== null) {
      clearTimeout(viewedDismissTimer);
    }
    viewedGracePeriod.value = true;
    viewedDismissTimer = window.setTimeout(() => {
      clearTask();
    }, VIEWED_DISMISS_MS);
  }

  /** 启动完成态自动消失计时 */
  function startAutoDismissTimer() {
    clearDismissTimer();
    dismissTimer = window.setTimeout(() => {
      markViewed();
    }, AUTO_DISMISS_MS);
  }

  /** 清空任务，留给 resetTask/cancelTask 调 */
  function clearTask() {
    currentTask.value = null;
    taskOriginTab.value = null;
    filename.value = "";
    savePath.value = "";
    viewed.value = false;
    panelOpen.value = false;
    viewedGracePeriod.value = false;
    speed.value = 0;
    lastDownloaded = 0;
    lastTimestamp = 0;
  }

  // ========== Actions ==========
  /**
   * 启动下载任务，替代原来 useDownload().start
   * @param opts 下载参数
   * @param meta 展示元信息，文件名、保存路径、来源
   */
  async function startTask(
    opts: DownloadOptions,
    meta: { filename: string; savePath: string; origin: TaskOrigin },
  ): Promise<void> {
    stopPolling();
    clearDismissTimer();
    activeSession += 1;
    const session = activeSession;

    // 重置为新任务状态
    currentTask.value = {
      id: "",
      total_size: 0,
      downloaded: 0,
      progress: 0,
      status: "Pending",
      is_finished: false,
    };
    taskOriginTab.value = meta.origin;
    filename.value = meta.filename;
    savePath.value = meta.savePath;
    viewed.value = false;
    panelOpen.value = false;
    viewedGracePeriod.value = false;
    speed.value = 0;
    lastDownloaded = 0;
    lastTimestamp = 0;

    try {
      const id = await downloadApi.downloadFile(opts);
      if (session !== activeSession) return;
      if (!currentTask.value) return;
      currentTask.value.id = id;

      let pollingInFlight = false;
      pollTimer = window.setInterval(async () => {
        if (session !== activeSession) {
          stopPolling();
          return;
        }
        const task = currentTask.value;
        if (!task || pollingInFlight || task.id !== id || task.is_finished) {
          if (!task || task.id !== id || task.is_finished) stopPolling();
          return;
        }

        pollingInFlight = true;
        try {
          const data = await downloadApi.pollTask(id);
          if (session !== activeSession || !currentTask.value || currentTask.value.id !== id)
            return;

          // 算瞬时速度
          const now = Date.now();
          if (lastTimestamp > 0 && now > lastTimestamp) {
            const deltaBytes = data.downloaded - lastDownloaded;
            const deltaSec = (now - lastTimestamp) / 1000;
            if (deltaSec > 0) speed.value = Math.max(0, deltaBytes / deltaSec);
          }
          lastDownloaded = data.downloaded;
          lastTimestamp = now;

          Object.assign(currentTask.value, data);
          if (data.is_finished) {
            currentTask.value.progress = 100;
            speed.value = 0;
            stopPolling();
            startAutoDismissTimer();
          }
        } catch (err) {
          if (
            session === activeSession &&
            currentTask.value &&
            currentTask.value.id === id &&
            !currentTask.value.is_finished
          ) {
            currentTask.value.status = { Error: i18n.t("downloader.connection_lost") };
            currentTask.value.is_finished = true;
            speed.value = 0;
            startAutoDismissTimer();
          }
          stopPolling();
        } finally {
          pollingInFlight = false;
        }
      }, POLL_INTERVAL_MS);
    } catch (err: any) {
      if (currentTask.value) {
        currentTask.value.status = { Error: err.toString() };
        currentTask.value.is_finished = true;
        startAutoDismissTimer();
      }
    }
  }

  /** 撅掉下载任务 */
  async function cancelTask(): Promise<void> {
    if (currentTask.value?.id) {
      try {
        await downloadApi.cancelDownloadTask(currentTask.value.id);
      } catch (e) {
        console.error("Failed to cancel download task:", e);
      }
    }
    resetTask();
  }

  /** 重置任务状态，全部清零 */
  function resetTask(): void {
    stopPolling();
    clearDismissTimer();
    clearTask();
  }

  /**
   * 标记用户看过完成结果了
   * - 取消自动消失计时
   * - 不清空 currentTask，留着完成态给下载页按钮态判断，只靠 shouldShowPill 把任务球藏起来
   */
  function markViewed(): void {
    viewed.value = true;
    clearDismissTimer();
  }

  /**
   * 开/关详情面板
   * - 开面板：取消看过后的延时消失计时，用户又来看了
   * - 关面板：完成了且看过了，启动短延时 8s 后真正撅掉
   */
  function setPanelOpen(open: boolean): void {
    panelOpen.value = open;
    if (open) {
      // 重新开面板，取消看过后的消失计时和宽限期
      if (viewedDismissTimer !== null) {
        clearTimeout(viewedDismissTimer);
        viewedDismissTimer = null;
      }
      viewedGracePeriod.value = false;
    } else {
      // 关面板时完成了且看过了，启动短延时消失
      if (isFinished.value && viewed.value) {
        startViewedDismissTimer();
      }
    }
  }

  return {
    // 状态
    currentTask,
    taskOriginTab,
    filename,
    savePath,
    viewed,
    panelOpen,
    speed,
    // 计算属性
    isDownloading,
    isFinished,
    isCompleted,
    isError,
    taskError,
    progress,
    statusLabel,
    shouldShowPill,
    // 操作
    startTask,
    cancelTask,
    resetTask,
    markViewed,
    setPanelOpen,
  };
});
