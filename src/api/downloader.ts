import {reactive, onUnmounted, computed} from "vue";
import { tauriInvoke } from "./tauri";

export type TaskStatus =
    | 'Pending'
    | 'Downloading'
    | 'Completed'
    | { Error: string };

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
      if (typeof taskInfo.status === 'object' && 'Error' in taskInfo.status) {
        return taskInfo.status.Error;
      }
      return null;
    });

    const isSuccess = computed(() => taskInfo.status === 'Completed');

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
              taskInfo.status = { Error: "任务连接丢失" };
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
  }
};