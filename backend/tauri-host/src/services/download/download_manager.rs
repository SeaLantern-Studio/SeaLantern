//! 下载任务管理器

mod create;
mod query;
mod state;

use crate::hardcode_data::external_services::COMMON_HTTP_BROWSER_USER_AGENT;
use crate::models::download::TaskProgressResponse;
use crate::utils::downloader::MultiThreadDownloader;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 下载任务总管理器
pub struct DownloadManager {
    // 使用 RwLock 保证多线程下对任务 Map 的读写安全
    tasks: Arc<RwLock<HashMap<Uuid, Arc<state::DownloadTaskState>>>>,
    downloader: Arc<MultiThreadDownloader>,
}

impl DownloadManager {
    /// 创建下载管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            downloader: Arc::new(MultiThreadDownloader::new(COMMON_HTTP_BROWSER_USER_AGENT)),
        }
    }

    /// 创建下载任务
    ///
    /// # Parameters
    ///
    /// - `url`: 下载地址
    /// - `path`: 保存路径
    /// - `thread_count`: 下载线程数
    pub async fn create_task(&self, url: &str, path: &str, thread_count: usize) -> Uuid {
        create::create_task(self, url, path, thread_count).await
    }

    /// 读取全部任务进度，并顺手清理已结束任务
    pub async fn get_all_progress(&self) -> Vec<TaskProgressResponse> {
        query::get_all_progress(self).await
    }

    /// 查询任务进度，并在结束后移除任务
    ///
    /// # Parameters
    ///
    /// - `id`: 任务 ID
    pub async fn get_progress_and_remove(&self, id: Uuid) -> Option<TaskProgressResponse> {
        query::get_progress_and_remove(self, id).await
    }

    /// 取消任务并删除临时文件
    ///
    /// # Parameters
    ///
    /// - `id`: 任务 ID
    pub async fn cancel_task(&self, id: Uuid) -> Result<(), String> {
        create::cancel_task(self, id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::download::TaskStatus;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::Router;
    use std::ops::RangeInclusive;
    use std::sync::Arc as StdArc;
    use std::time::Duration;
    use tokio;
    use tokio::net::TcpListener;

    #[derive(Clone)]
    struct TestDownloadPayload {
        bytes: StdArc<Vec<u8>>,
    }

    async fn serve_test_payload(
        State(payload): State<TestDownloadPayload>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        let total_len = payload.bytes.len();

        if let Some(range_header) = headers.get(header::RANGE) {
            let range_value = range_header
                .to_str()
                .expect("range header should be valid utf-8");
            let range = parse_range_header(range_value, total_len)
                .expect("range header should be parseable for test payload");

            let start = *range.start();
            let end = *range.end();
            let bytes = payload.bytes[start..=end].to_vec();
            let content_range = format!("bytes {}-{}/{}", start, end, total_len);

            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::CONTENT_RANGE,
                HeaderValue::from_str(&content_range).expect("content-range should be valid"),
            );
            response_headers.insert(
                header::CONTENT_LENGTH,
                HeaderValue::from_str(&bytes.len().to_string())
                    .expect("content-length should be valid"),
            );

            return (StatusCode::PARTIAL_CONTENT, response_headers, Body::from(bytes))
                .into_response();
        }

        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&total_len.to_string()).expect("content-length should be valid"),
        );

        (StatusCode::OK, response_headers, Body::from((*payload.bytes).clone())).into_response()
    }

    fn parse_range_header(value: &str, total_len: usize) -> Option<RangeInclusive<usize>> {
        let bytes = value.strip_prefix("bytes=")?;
        let (start, end) = bytes.split_once('-')?;
        let start = start.parse::<usize>().ok()?;
        let end = if end.is_empty() {
            total_len.checked_sub(1)?
        } else {
            end.parse::<usize>().ok()?
        };

        if start > end || end >= total_len {
            return None;
        }

        Some(start..=end)
    }

    #[tokio::test]
    async fn test_download_manager() {
        let manager = DownloadManager::new();

        let payload = TestDownloadPayload {
            bytes: StdArc::new(vec![b'x'; 512 * 1024]),
        };
        let app = Router::new()
            .route("/download.bin", get(serve_test_payload))
            .with_state(payload.clone());
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("test listener should bind");
        let addr = listener
            .local_addr()
            .expect("listener should report local addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test download server should run");
        });

        let url = format!("http://{}/download.bin", addr);
        let temp_dir = tempfile::tempdir().expect("tempdir should exist");
        let save_path = temp_dir.path().join("test_manager_output.bin");
        let save_path_str = save_path.to_string_lossy().to_string();

        let task_id = manager.create_task(&url, &save_path_str, 8).await;
        println!("任务已创建, ID: {}", task_id);

        let mut completed = false;
        let mut timeout_counter = 0;

        while timeout_counter < 30 {
            if let Some(resp) = query::get_progress(&manager, task_id).await {
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

        let final_check = query::get_progress(&manager, task_id).await;
        assert!(final_check.is_none(), "测试失败：任务在清理后依然存在");
        println!("任务已成功从管理器中移除。");

        if save_path.exists() {
            let _ = std::fs::remove_file(&save_path);
            println!("测试残留文件已清理。");
        }
    }
}
