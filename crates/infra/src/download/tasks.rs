//! 分片任务管理。
//!
//! 负责将文件按线程数分割为多个分片、生成下载任务并启动后台监控。

use std::sync::Arc;

use crate::download::chunk::download_chunk;
use crate::download::status::{DownloadError, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// 将文件分割为多个分片范围。
///
/// # Parameters
///
/// - `total_size`: 文件总大小
/// - `thread_count`: 分片数量
///
/// # Returns
///
/// 返回 `(start, end)` 元组的列表，每个元组表示一个分片的起止位置。
pub(super) fn split_ranges(total_size: u64, thread_count: usize) -> Vec<(u64, u64)> {
    if total_size == 0 {
        return Vec::new();
    }

    let actual_count = (thread_count as u64).min(total_size) as usize;
    let chunk_size = total_size / actual_count as u64;
    let mut ranges = Vec::with_capacity(actual_count);

    for i in 0..actual_count {
        let start = i as u64 * chunk_size;
        let end = if i == actual_count - 1 {
            total_size - 1
        } else {
            start + chunk_size - 1
        };
        ranges.push((start, end));
    }

    ranges
}

/// 生成全部分片下载任务。
///
/// 为每个分片 `spawn` 一个 tokio task，所有任务共享同一个 `DownloadStatus`。
///
/// # Parameters
///
/// - `client`: 已配置的 HTTP 客户端（Clone 后在任务间共享）
/// - `url`: 下载地址
/// - `path`: 本地保存路径
/// - `thread_count`: 分片数量
/// - `total_size`: 文件总大小
/// - `status`: 共享下载状态
///
/// # Returns
///
/// 返回所有分片任务的 `JoinHandle` 列表。
pub(super) fn spawn_download_tasks(
    client: NetClient,
    url: String,
    path: String,
    thread_count: usize,
    total_size: u64,
    status: &Arc<DownloadStatus>,
) -> Vec<tokio::task::JoinHandle<Result<(), DownloadError>>> {
    let ranges = split_ranges(total_size, thread_count);
    let mut tasks = Vec::with_capacity(ranges.len());

    for (start, end) in ranges {
        let client = client.clone();
        let url = url.clone();
        let path = path.clone();
        let status = Arc::clone(status);

        tasks.push(tokio::spawn(async move {
            download_chunk(&client, &url, &path, start, end, status).await
        }));
    }

    tasks
}

/// 启动后台监控任务。
///
/// 在后台等待所有分片任务完成，汇总错误信息到 `DownloadStatus`。
/// 所有分片成功后调用 `download_completed()` 记录完成事件。
///
/// # Parameters
///
/// - `tasks`: 分片任务的 `JoinHandle` 列表
/// - `status`: 共享下载状态
/// - `url`: 下载地址（用于完成事件记录）
/// - `total_size`: 文件总大小（用于完成事件记录）
pub(super) fn spawn_task_monitor(
    tasks: Vec<tokio::task::JoinHandle<Result<(), DownloadError>>>,
    status: &Arc<DownloadStatus>,
    url: String,
    total_size: u64,
) {
    let status = Arc::clone(status);
    tokio::spawn(async move {
        let start = std::time::Instant::now();
        let mut has_error = false;

        for task in tasks {
            match task.await {
                Ok(Ok(_)) => {}
                Ok(Err(err)) => {
                    has_error = true;
                    // chunk_failed() already logged this at error level;
                    // here we only propagate to DownloadStatus without duplicate logging.
                    if !status.cancelled() {
                        status.set_error(err.to_string()).await;
                    }
                }
                Err(err) => {
                    has_error = true;
                    tracing::error!(
                        target: observability::DOWNLOAD_TARGET,
                        event_name = "chunk_thread_crashed",
                        error = %err,
                        "chunk thread crashed"
                    );
                    status
                        .set_error(format!("chunk thread crashed: {}", err))
                        .await;
                }
            }
        }

        if !has_error && !status.cancelled() {
            let elapsed = start.elapsed().as_millis() as u64;
            observability::download_completed(&url, total_size, elapsed);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ranges_normal() {
        let ranges = split_ranges(1000, 4);
        assert_eq!(ranges.len(), 4);
        assert_eq!(ranges[0], (0, 249));
        assert_eq!(ranges[1], (250, 499));
        assert_eq!(ranges[2], (500, 749));
        assert_eq!(ranges[3], (750, 999));
    }

    #[test]
    fn split_ranges_exact_division() {
        let ranges = split_ranges(8, 4);
        assert_eq!(ranges.len(), 4);
        assert_eq!(ranges[0], (0, 1));
        assert_eq!(ranges[3], (6, 7));
    }

    #[test]
    fn split_ranges_empty() {
        let ranges = split_ranges(0, 8);
        assert!(ranges.is_empty());
    }

    #[test]
    fn split_ranges_small_file() {
        let ranges = split_ranges(3, 8);
        assert_eq!(ranges.len(), 3);
    }

    #[test]
    fn split_ranges_single_thread() {
        let ranges = split_ranges(100, 1);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], (0, 99));
    }
}
