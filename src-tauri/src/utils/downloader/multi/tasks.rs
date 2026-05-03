use super::chunk::download_chunk;
use super::super::{DownloadError, DownloadStatus};
use reqwest::Client;
use std::sync::Arc;

/// 生成全部下载分片任务
pub(super) fn spawn_download_tasks(
    client: Arc<Client>,
    url: &str,
    output_path: &str,
    thread_count: usize,
    total_size: u64,
    status: &Arc<DownloadStatus>,
) -> Vec<tokio::task::JoinHandle<Result<(), DownloadError>>> {
    let mut tasks = Vec::new();
    let chunk_size = total_size / thread_count as u64;

    for index in 0..thread_count {
        let start = index as u64 * chunk_size;
        let end = if index == thread_count - 1 {
            total_size - 1
        } else {
            start + chunk_size - 1
        };

        let url = url.to_string();
        let path = output_path.to_string();
        let client_ptr = Arc::clone(&client);
        let status_ptr = Arc::clone(status);

        tasks.push(tokio::spawn(async move {
            download_chunk(client_ptr, url, path, start, end, status_ptr).await
        }));
    }

    tasks
}

/// 启动后台监控，汇总全部分片结果
pub(super) fn spawn_task_monitor(
    tasks: Vec<tokio::task::JoinHandle<Result<(), DownloadError>>>,
    status: &Arc<DownloadStatus>,
) {
    let status_for_monitor = Arc::clone(status);
    tokio::spawn(async move {
        for task in tasks {
            match task.await {
                Ok(Ok(_)) => {}
                Ok(Err(err)) => {
                    if !status_for_monitor.cancelled() {
                        status_for_monitor.set_error(err.to_string()).await;
                    }
                }
                Err(err) => {
                    status_for_monitor
                        .set_error(format!("线程崩溃: {}", err))
                        .await;
                }
            }
        }
    });
}
