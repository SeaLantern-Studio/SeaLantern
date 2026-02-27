use crate::models::download::{TaskProgressResponse, TaskStatus};
use crate::utils::downloader::MultiThreadDownloader;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct DownloadManager {
    // 使用 RwLock 保证多线程下对任务 Map 的读写安全
    tasks: Arc<RwLock<HashMap<Uuid, Arc<DownloadTaskState>>>>,
    downloader: Arc<MultiThreadDownloader>,
}

struct DownloadTaskState {
    _url: String,
    _file_path: String,
    status_handle: tokio::sync::Mutex<Option<Arc<crate::utils::downloader::DownloadStatus>>>,
    internal_status: RwLock<TaskStatus>,
    join_handle: tokio::sync::Mutex<Option<JoinHandle<()>>>, // 添加 JoinHandle 来追踪后台任务
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            downloader: Arc::new(MultiThreadDownloader::new(
                crate::utils::downloader::USER_AGENT_EXAMPLE,
            )),
        }
    }

    /// 创建下载任务
    pub async fn create_task(&self, url: &str, path: &str, thread_count: usize) -> Uuid {
        let id = Uuid::new_v4();
        let state = Arc::new(DownloadTaskState {
            _url: url.to_string(),
            _file_path: path.to_string(),
            status_handle: tokio::sync::Mutex::new(None),
            internal_status: RwLock::new(TaskStatus::Pending),
            join_handle: tokio::sync::Mutex::new(None), // 初始化为空的 JoinHandle
        });

        // 将任务存入管理 Map
        self.tasks.write().await.insert(id, state.clone());

        let downloader = self.downloader.clone();
        let url_str = url.to_string();
        let path_str = path.to_string();

        // 在外部创建 state_clone 以在闭包内外使用
        let state_clone = state.clone();

        // 这里的 state_clone 是 Arc 的克隆，move 进来后可以在后台线程持续更新该任务的具体状态
        let handle = tokio::spawn(async move {
            match downloader.download(&url_str, &path_str, thread_count).await {
                Ok(handle) => {
                    // 1. 关联下载句柄
                    {
                        let mut h = state_clone.status_handle.lock().await;
                        *h = Some(handle);
                        let mut s = state_clone.internal_status.write().await;
                        *s = TaskStatus::Downloading;
                    }

                    // 2. 轮询检测是否完成（utils 层的下载器完成后，handle.snapshot().is_finished 会变为 true）
                    loop {
                        // 1. 在一个独立的作用域内获取状态句柄，确保锁能及时释放
                        let status_handle_opt = {
                            let h = state_clone.status_handle.lock().await;
                            h.as_ref().cloned() // 克隆 Arc 指针，增加引用计数，释放互斥锁
                        };

                        let mut is_done = false;

                        // 2. 如果句柄存在，再进行异步 snapshot 调用
                        if let Some(status_handle) = status_handle_opt {
                            let snap = status_handle.snapshot().await;

                            // 如果出错，也标记为结束，并更新内部状态
                            if let Some(err_msg) = snap.error {
                                let mut s = state_clone.internal_status.write().await;
                                *s = TaskStatus::Error(err_msg);
                                break;
                            }

                            if snap.is_finished {
                                is_done = true;
                            }
                        }

                        if is_done {
                            let mut s = state_clone.internal_status.write().await;
                            // 只有在没报错的情况下才标记为 Completed
                            if let TaskStatus::Downloading = *s {
                                *s = TaskStatus::Completed;
                            }
                            break;
                        }

                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
                Err(e) => {
                    let mut s = state_clone.internal_status.write().await;
                    *s = TaskStatus::Error(e);
                }
            }
        });

        // 将 JoinHandle 存储到状态中
        {
            let mut join_handle_lock = state.join_handle.lock().await;
            *join_handle_lock = Some(handle);
        }

        id
    }

    /// 查询进度并尝试清理
    pub async fn get_progress(&self, id: Uuid) -> Option<TaskProgressResponse> {
        let tasks = self.tasks.read().await;
        let state = tasks.get(&id)?;

        let mut status = state.internal_status.read().await.clone();
        let mut progress = 0.0;
        let mut is_finished = false;
        let mut total_size: u64 = 0;
        let mut downloaded: u64 = 0;

        // 如果在下载中，从 utils 的 Atomic 变量取实时数据
        if let Some(handle) = &*state.status_handle.lock().await {
            let snap = handle.snapshot().await;
            progress = snap.progress_percentage;
            total_size = snap.total_size;
            downloaded = snap.downloaded;
            if snap.is_finished {
                status = TaskStatus::Completed;
                is_finished = true;
            }
        }

        if matches!(status, TaskStatus::Completed | TaskStatus::Error(_)) {
            is_finished = true;
        }

        let resp = TaskProgressResponse {
            id,
            total_size,
            downloaded,
            progress,
            status,
            is_finished,
        };

        Some(resp)
    }

    /// 显式清理任务
    // pub async fn remove_task(&self, id: Uuid) {
    //     self.tasks.write().await.remove(&id);
    // }
    /// 批量获取所有任务进度，并清理已完成的任务
    pub async fn get_all_progress(&self) -> Vec<TaskProgressResponse> {
        let mut results = Vec::new();
        let mut to_remove = Vec::new();

        // 1. 读取所有任务状态
        let tasks = self.tasks.read().await;
        for (id, state) in tasks.iter() {
            let mut status = state.internal_status.read().await.clone();
            let mut progress = 0.0;
            let mut total_size: u64 = 0;
            let mut downloaded: u64 = 0;

            if let Some(handle) = &*state.status_handle.lock().await {
                let snap = handle.snapshot().await;
                progress = snap.progress_percentage;
                total_size = snap.total_size;
                downloaded = snap.downloaded;
                if snap.is_finished {
                    status = TaskStatus::Completed;
                }
            }

            let is_finished = matches!(status, TaskStatus::Completed | TaskStatus::Error(_));
            results.push(TaskProgressResponse {
                id: *id,
                total_size,
                downloaded,
                progress,
                status,
                is_finished,
            });

            if is_finished {
                to_remove.push(*id);
            }
        }
        drop(tasks); // 释放读锁

        // 2. 批量清理已结束的任务 (阅后即焚)
        if !to_remove.is_empty() {
            let mut tasks_write = self.tasks.write().await;
            for id in to_remove {
                tasks_write.remove(&id);
            }
        }

        results
    }

    pub async fn get_progress_and_remove(&self, id: Uuid) -> Option<TaskProgressResponse> {
        let resp = self.get_progress(id).await?;

        if resp.is_finished {
            let mut tasks = self.tasks.write().await;
            tasks.remove(&id);
        }

        Some(resp)
    }

    pub async fn cancel_task(&self, id: Uuid) -> Result<(), String> {
        let tasks = self.tasks.read().await;
        if let Some(state) = tasks.get(&id) {
            // 获取文件路径以便在取消后删除
            let file_path = state._file_path.clone();

            // 调用取消方法，中断下载
            {
                let handle = state.status_handle.lock().await;
                if let Some(ref status_handle) = *handle {
                    status_handle.cancel(); // 发送取消信号到实际的下载工作
                }
            }

            // 取消 JoinHandle 中的任务
            {
                let mut join_handle_guard = state.join_handle.lock().await;
                if let Some(handle) = join_handle_guard.take() {
                    // take 会取出并移除原始值
                    handle.abort(); // 中止后台任务
                }
            }

            drop(tasks);

            // 从管理器中移除任务
            self.tasks.write().await.remove(&id);

            // 删除临时下载的文件
            tokio::fs::remove_file(&file_path).await.map_err(|e| format!("删除临时文件失败: {}", e))?;

            Ok(())
        } else {
            // 任务不存在（可能已完成或被清理），不返回错误
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio;

    #[tokio::test]
    async fn test_download_manager() {
        let manager = DownloadManager::new();

        let url = "https://files.mcjars.app/mohist/1.12.2/1.12.2-17e3fd09/server.jar";
        let save_path = "test_manager_output.txt";

        let task_id = manager.create_task(url, save_path, 32).await;
        println!("任务已创建, ID: {}", task_id);

        let mut completed = false;
        let mut timeout_counter = 0;

        while timeout_counter < 30 {
            if let Some(resp) = manager.get_progress(task_id).await {
                println!(
                    "进度: {:.2}% | 状态: {:?} | 是否完成: {} | 已下载：{} | 总大小：{}",
                    resp.progress, resp.status, resp.is_finished, resp.downloaded, resp.total_size
                );

                if resp.is_finished {
                    if let TaskStatus::Completed = resp.status {
                        println!("测试通过：文件下载完成！");
                        completed = true;
                    } else if let TaskStatus::Error(e) = resp.status {
                        panic!("测试失败：下载过程中出现错误: {}", e);
                    }
                    break;
                }
            } else {
                panic!("测试失败：无法获取任务状态");
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
            timeout_counter += 1;
        }

        assert!(completed, "测试超时：任务未在规定时间内完成");

        match manager.cancel_task(task_id).await {
            Ok(_) => println!("任务取消成功"),
            Err(e) => println!("任务取消失败: {}", e),
        }

        let final_check = manager.get_progress(task_id).await;
        assert!(final_check.is_none(), "测试失败：任务在清理后依然存在");
        println!("任务已成功从管理器中移除。");

        if std::path::Path::new(save_path).exists() {
            let _ = std::fs::remove_file(save_path);
            println!("测试残留文件已清理。");
        }
    }
}
