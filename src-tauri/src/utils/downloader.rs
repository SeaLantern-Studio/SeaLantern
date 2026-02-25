use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};
use tokio::sync::RwLock;

///一个基本的User-agent
pub const USER_AGENT_EXAMPLE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0";

/// 实时进度快照
#[derive(Debug, Clone, serde::Serialize)] // 如果需要返回给前端，可以加 Serialize
pub struct DownloadSnapshot {
    pub downloaded: u64,
    pub total_size: u64,
    pub progress_percentage: f64,
    pub is_finished: bool,
    pub error: Option<String>,
}

/// 状态管理
pub struct DownloadStatus {
    pub total_size: u64,
    pub downloaded: AtomicU64,
    // 使用 tokio 的 RwLock 存储错误信息
    pub error_message: RwLock<Option<String>>,
}

impl DownloadStatus {
    pub fn new(total_size: u64) -> Self {
        Self {
            total_size,
            downloaded: AtomicU64::new(0),
            error_message: RwLock::new(None),
        }
    }

    /// 设置错误信息
    pub async fn set_error(&self, msg: String) {
        let mut lock = self.error_message.write().await;
        *lock = Some(msg);
    }

    /// 获取当前快照，用于传递给前端
    pub async fn snapshot(&self) -> DownloadSnapshot {
        let downloaded = self.downloaded.load(Ordering::Relaxed);
        let error = self.error_message.read().await.clone();

        DownloadSnapshot {
            downloaded,
            total_size: self.total_size,
            progress_percentage: if self.total_size > 0 {
                (downloaded as f64 / self.total_size as f64) * 100.0
            } else {
                0.0
            },
            is_finished: downloaded >= self.total_size || error.is_some(),
            error,
        }
    }
}

///多线程下载
pub struct MultiThreadDownloader {
    client: Client,
}

impl MultiThreadDownloader {
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: Client::builder()
                // 调优超时：连接 15s，读取数据块 30s
                .connect_timeout(Duration::from_secs(15))
                .read_timeout(Duration::from_secs(30))
                .user_agent(user_agent)
                .build()
                .unwrap(),
        }
    }

    pub async fn download(
        &self,
        url: &str,
        output_path: &str,
        thread_count: usize,
    ) -> Result<Arc<DownloadStatus>, String> {
        if thread_count == 0 {
            return Err("Thread count must be positive".to_string());
        }
        let res = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| format!("HEAD 请求失败: {}", e))?;

        let total_size = res
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| ct.parse::<u64>().ok())
            .ok_or("服务器未返回 Content-Length")?;

        let file = tokio::fs::File::create(output_path)
            .await
            .map_err(|e| e.to_string())?;
        file.set_len(total_size).await.map_err(|e| e.to_string())?;

        let status = Arc::new(DownloadStatus::new(total_size));
        let chunk_size = total_size / thread_count as u64;
        let client = Arc::new(self.client.clone());

        let mut tasks = Vec::new();

        for i in 0..thread_count {
            let start = i as u64 * chunk_size;
            let end = if i == thread_count - 1 {
                total_size - 1
            } else {
                start + chunk_size - 1
            };

            let url = url.to_string();
            let path = output_path.to_string();
            let client_ptr = Arc::clone(&client);
            let status_ptr = Arc::clone(&status);

            // 移除 unwrap()，让子任务返回 Result
            tasks.push(tokio::spawn(async move {
                Self::_worker(client_ptr, url, path, start, end, status_ptr).await
            }));
        }

        // 异步监控线程结果
        let status_for_monitor = Arc::clone(&status);
        tokio::spawn(async move {
            for task in tasks {
                match task.await {
                    Ok(Ok(_)) => {} // 线程执行成功
                    Ok(Err(e)) => {
                        // 子任务逻辑错误 (如 TimedOut)
                        status_for_monitor.set_error(e.to_string()).await;
                    }
                    Err(e) => {
                        // 线程 Panic 或被取消
                        status_for_monitor
                            .set_error(format!("线程崩溃: {}", e))
                            .await;
                    }
                }
            }
        });

        Ok(status)
    }

    async fn _worker(
        client: Arc<Client>,
        url: String,
        path: String,
        start: u64,
        end: u64,
        status: Arc<DownloadStatus>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let range = format!("bytes={}-{}", start, end);
        // 使用 map_err 包装可能的请求错误
        let mut response = client.get(url).header("Range", range).send().await?;

        let file = OpenOptions::new().write(true).open(path).await?;
        let mut writer = BufWriter::with_capacity(128 * 1024, file);
        writer.seek(SeekFrom::Start(start)).await?;

        let mut local_downloaded = 0u64;

        // 移除 chunk 后的 unwrap，使用 ? 自动传播错误
        while let Some(chunk) = response.chunk().await? {
            let len = chunk.len() as u64;
            writer.write_all(&chunk).await?;

            local_downloaded += len;
            // 降低状态更新频率以减轻原子操作压力
            if local_downloaded > 512 * 1024 {
                status
                    .downloaded
                    .fetch_add(local_downloaded, Ordering::Relaxed);
                local_downloaded = 0;
            }
        }

        writer.flush().await?;
        status
            .downloaded
            .fetch_add(local_downloaded, Ordering::Relaxed);

        Ok(())
    }
}
//
// ///单线程下载
// pub struct SingleThreadDownloader {
//     client: Client,
// }
//
// ///单线程下载实现
// impl SingleThreadDownloader {
//     pub fn new(user_agent: &str) -> Self {
//         Self {
//             client: Client::builder()
//                 .timeout(Duration::from_secs(30))
//                 .user_agent(user_agent)
//                 .build()
//                 .unwrap(),
//         }
//     }
//
//     pub async fn download(&self, url: &str, output_path: &str) -> Result<(), String> {
//         let response = self
//             .client
//             .get(url)
//             .send()
//             .await
//             .map_err(|e| e.to_string())?;
//
//         let total_size = response
//             .content_length()
//             .ok_or("Failed to get content length")?;
//
//         let pb = ProgressBar::new(total_size);
//         pb.set_style(
//             ProgressStyle::with_template(
//                 "[{elapsed_precise}] {bar:40.green/white} {bytes}/{total_bytes} ({eta}) {msg}",
//             )
//             .unwrap()
//             .progress_chars("##-"),
//         );
//         pb.set_message("Downloading...");
//
//         let mut file = OpenOptions::new()
//             .create(true)
//             .write(true)
//             .truncate(true)
//             .open(output_path)
//             .await
//             .map_err(|e| e.to_string())?;
//
//         let mut downloaded: u64 = 0;
//         let mut stream = response.bytes_stream();
//
//         while let Some(item) = stream.next().await {
//             let chunk = item.map_err(|e| e.to_string())?;
//             file.write_all(&chunk).await.map_err(|e| e.to_string())?;
//
//             let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
//             downloaded = new;
//             pb.set_position(new);
//         }
//
//         pb.finish_with_message("Download complete");
//         Ok(())
//     }
// }

///测试函数：多线程下载
#[tokio::test]
async fn test_multi_thread_download() -> Result<(), String> {
    let downloader = MultiThreadDownloader::new("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0"); //下载的线程数, User-agent

    let url = "https://files.mcjars.app/mohist/1.12.2/1.12.2-17e3fd09/server.jar"; // 一个大文件
    let save_path = "D:\\Projects\\MinecraftLuncher\\SeaLantern\\target\\multi_thread_download.bin";

    match downloader.download(url, save_path, 32).await {
        Ok(status_handle) => {
            println!("Downloaded to {:?}", save_path);
            loop {
                let info = status_handle.snapshot().await;
                println!(
                    "当前进度: {:.2}% ({} / {})",
                    info.progress_percentage, info.downloaded, info.total_size
                );

                if info.is_finished {
                    println!("下载完成！");
                    break;
                }

                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("\n 下载中止: {}", e);

            if std::path::Path::new(save_path).exists() {
                let _ = std::fs::remove_file(save_path);
                println!("已清理不完整的文件。");
            }
            Err(e.to_string())
        }
    }
}

// ///测试函数：单线程下载
// #[tokio::test]
// async fn test_simple_thread_download() -> Result<(), String> {
//     let downloader = SingleThreadDownloader::new("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0"); //下载的线程数, User-agent
//
//     let url = "https://files.mcjars.app/mohist/1.12.2/1.12.2-17e3fd09/server.jar.CHECKSUMS.txt"; // 一个大文件
//     let save_path =
//         "D:\\Projects\\MinecraftLuncher\\SeaLantern\\target\\simple_thread_download.bin";
//
//     match downloader.download(url, save_path).await {
//         Ok(_) => {
//             println!("\n 下载成功！文件已保存至: {}", save_path);
//             Ok(())
//         }
//         Err(e) => {
//             // 这里会捕获到具体的错误原因，比如 404、网络断开或权限不足
//             eprintln!("\n 下载中止: {}", e);
//
//             // 可以在这里清理未下载完的残留文件
//             if std::path::Path::new(save_path).exists() {
//                 let _ = std::fs::remove_file(save_path);
//                 println!("已清理不完整的文件。");
//             }
//             Err(e.to_string())
//         }
//     }
// }
