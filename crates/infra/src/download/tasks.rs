//! Chunk task management.
//!
//! Responsible for splitting a file into multiple chunks by thread count, spawning download tasks, and starting background monitoring.

use std::sync::Arc;

use crate::download::chunk::download_chunk;
use crate::download::status::{DownloadError, DownloadStatus};
use crate::net::client::NetClient;
use crate::observability;

/// Splits the file into multiple chunk ranges.
///
/// # Parameters
///
/// - `total_size`: Total file size
/// - `thread_count`: Number of chunks
///
/// # Returns
///
/// Returns a list of `(start, end)` tuples, each representing the start and end positions of a chunk.
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

/// Spawns all chunk download tasks.
///
/// Spawns one tokio task per chunk, all tasks share the same `DownloadStatus`.
///
/// # Parameters
///
/// - `client`: Configured HTTP client (cloned and shared among tasks)
/// - `url`: Download URL
/// - `path`: Local save path
/// - `thread_count`: Number of chunks
/// - `total_size`: Total file size
/// - `status`: Shared download status
///
/// # Returns
///
/// Returns a list of `JoinHandle`s for all chunk tasks.
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

/// Starts the background monitor task.
///
/// Waits in the background for all chunk tasks to complete, aggregating errors into `DownloadStatus`.
/// Calls `download_completed()` to record the completion event when all chunks succeed.
///
/// # Parameters
///
/// - `tasks`: List of chunk task `JoinHandle`s
/// - `status`: Shared download status
/// - `url`: Download URL (for completion event logging)
/// - `total_size`: Total file size (for completion event logging)
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
