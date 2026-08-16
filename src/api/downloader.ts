import { reactive, onUnmounted, computed } from "vue";
import { invoke } from "./invoke";
import { i18n } from "@language";

export type TaskStatus = "Pending" | "Downloading" | "Completed" | { Error: string };

export interface DownloadTaskInfo {
  id: string;
  total_size: number;
  downloaded: number;
  progress: number;
  status: TaskStatus;
  is_finished: boolean;
}

export interface DownloadOptions {
  url: string;
  save_path: string;
  thread_count?: number;
}

export interface DownloadLink {
  version: string; // 版本号
  fileName: string; // 文件名
  url: string; // 下载URL
}

// 类型下载链接集合
export interface TypeDownloadLinks {
  server_type: string; // 服务器类型名称
  versions: string[]; // 可用版本列表
  links: DownloadLink[]; // 下载链接列表
}

// 基础下载链接数据
export interface BaseDownloadLinks {
  server_types: string[]; // 所有服务器类型
  links: TypeDownloadLinks[]; // 各类型的详细链接
}

export const downloadApi = {
  /**
   * 基础 API：创建下载任务
   */
  async downloadFile(options: DownloadOptions): Promise<string> {
    // 后端 download_create 接收 request 结构体参数（snake_case 字段）
    const task = await invoke<DownloadTaskInfo>("download_create", {
      request: {
        url: options.url,
        save_path: options.save_path,
        thread_count: options.thread_count || 32,
      },
    });
    return task.id;
  },

  /**
   * 基础 API：单次查询
   */
  async pollTask(id: string): Promise<DownloadTaskInfo> {
    return invoke<DownloadTaskInfo>("download_query", { id });
  },

  /**
   * 删除/取消下载任务
   */
  async cancelDownloadTask(id: string): Promise<void> {
    return invoke<void>("download_cancel", { id });
  },

  /**
   * 启动并自动轮询
   */
  useDownload() {
    const taskInfo = reactive<DownloadTaskInfo>({
      id: "",
      total_size: 0,
      downloaded: 0,
      progress: 0,
      status: "Pending",
      is_finished: false,
    });

    const errorMessage = computed(() => {
      if (typeof taskInfo.status === "object" && "Error" in taskInfo.status) {
        return taskInfo.status.Error;
      }
      return null;
    });

    const isSuccess = computed(() => taskInfo.status === "Completed");

    let timer: number | null = null;
    let activeSession = 0;

    const start = async (options: DownloadOptions) => {
      stop();

      const session = ++activeSession;
      taskInfo.is_finished = false;
      taskInfo.progress = 0;
      taskInfo.status = "Pending";

      try {
        const id = await this.downloadFile(options);
        if (session !== activeSession) return;

        taskInfo.id = id;
        let pollingInFlight = false;

        const intervalId = window.setInterval(async () => {
          if (session !== activeSession) {
            clearInterval(intervalId);
            if (timer === intervalId) timer = null;
            return;
          }

          if (pollingInFlight || taskInfo.id !== id || taskInfo.is_finished) {
            if (taskInfo.id !== id || taskInfo.is_finished) {
              clearInterval(intervalId);
              if (timer === intervalId) timer = null;
            }
            return;
          }

          pollingInFlight = true;
          try {
            const data = await this.pollTask(id);
            if (session !== activeSession || taskInfo.id !== id) {
              return;
            }

            Object.assign(taskInfo, data);
            if (data.is_finished) {
              taskInfo.progress = 100;
              clearInterval(intervalId);
              if (timer === intervalId) timer = null;
            }
          } catch (err) {
            if (session === activeSession && taskInfo.id === id && !taskInfo.is_finished) {
              taskInfo.status = { Error: i18n.t("downloader.connection_lost") };
            }
            clearInterval(intervalId);
            if (timer === intervalId) timer = null;
          } finally {
            pollingInFlight = false;
          }
        }, 800);
        timer = intervalId;
      } catch (err: any) {
        taskInfo.status = { Error: err.toString() };
        taskInfo.is_finished = true;
      }
    };

    const stop = () => {
      activeSession += 1;
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    };

    const reset = () => {
      stop();
      taskInfo.id = "";
      taskInfo.total_size = 0;
      taskInfo.downloaded = 0;
      taskInfo.progress = 0;
      taskInfo.status = "Pending";
      taskInfo.is_finished = false;
    };

    onUnmounted(stop);

    return { taskInfo, start, stop, reset, errorMessage, isSuccess };
  },
};

export const downloadServerApi = {
  async getServerTypes(): Promise<string[]> {
    return invoke<string[]>("catalog_server_types");
  },

  async getVersionsByType(serverType: string): Promise<string[]> {
    return invoke<string[]>("catalog_versions", { server_type: serverType });
  },

  async getDownloadInfo(serverType: string, version: string): Promise<DownloadLink> {
    return invoke<DownloadLink>("catalog_details", {
      server_type: serverType,
      server_version: version,
    });
  },
};
