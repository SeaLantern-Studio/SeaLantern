import { reactive, onUnmounted, computed } from "vue";
import { tauriInvoke } from "./tauri";
import { i18n } from "@language";

export type TaskStatus = "Pending" | "Downloading" | "Completed" | { Error: string };

export interface DownloadTaskInfo {
  id: string;
  totalSize: number;
  downloaded: number;
  progress: number;
  status: TaskStatus;
  isFinished: boolean;
}

export interface DownloadOptions {
  url: string;
  savePath: string;
  threadCount?: number;
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
    return tauriInvoke<string>("download_file", {
      url: options.url,
      savePath: options.savePath,
      threadCount: options.threadCount || 32,
    });
  },

  /**
   * 基础 API：单次查询
   */
  async pollTask(id: string): Promise<DownloadTaskInfo> {
    return tauriInvoke<DownloadTaskInfo>("poll_task", { idStr: id });
  },

  /**
   * 删除/取消下载任务
   */
  async cancelDownloadTask(id: string): Promise<void> {
    return tauriInvoke<void>("cancel_download_task", {
      idStr: id,
    });
  },

  /**
   * 启动并自动轮询
   */
  useDownload() {
    const taskInfo = reactive<DownloadTaskInfo>({
      id: "",
      totalSize: 0,
      downloaded: 0,
      progress: 0,
      status: "Pending",
      isFinished: false,
    });

    const errorMessage = computed(() => {
      if (typeof taskInfo.status === "object" && "Error" in taskInfo.status) {
        return taskInfo.status.Error;
      }
      return null;
    });

    const isSuccess = computed(() => taskInfo.status === "Completed");

    let timer: number | null = null;

    const start = async (options: DownloadOptions) => {
      taskInfo.isFinished = false;
      taskInfo.progress = 0;

      try {
        const id = await this.downloadFile(options);
        taskInfo.id = id;

        timer = window.setInterval(async () => {
          try {
            const data = await this.pollTask(id);
            Object.assign(taskInfo, data);
            if (data.isFinished) {
              data.progress = 100;
              stop();
            }
          } catch (err) {
            if (!taskInfo.isFinished) {
              taskInfo.status = { Error: i18n.t("downloader.connection_lost") };
            }
            stop();
          }
        }, 800);
      } catch (err: any) {
        taskInfo.status = { Error: err.toString() };
        taskInfo.isFinished = true;
      }
    };

    const stop = () => {
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    };

    const reset = () => {
      stop();
      taskInfo.id = "";
      taskInfo.totalSize = 0;
      taskInfo.downloaded = 0;
      taskInfo.progress = 0;
      taskInfo.status = "Pending";
      taskInfo.isFinished = false;
    };

    onUnmounted(stop);

    return { taskInfo, start, stop, reset, errorMessage, isSuccess };
  },
};

export const downloadServerApi = {
  async getServerTypes(): Promise<string[]> {
    return tauriInvoke<string[]>("get_server_types");
  },

  async getVersionsByType(serverType: string): Promise<string[]> {
    return tauriInvoke<string[]>("get_versions_by_type", { serverType });
  },

  async getDownloadInfo(serverType: string, version: string): Promise<DownloadLink> {
    return tauriInvoke<DownloadLink>("get_download_info", {
      serverType,
      version,
    });
  },
};
