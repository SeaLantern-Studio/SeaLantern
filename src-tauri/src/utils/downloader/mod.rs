//! 下载工具
//!
//! 这里提供多线程文件下载、单线程文本读取和下载进度快照

mod multi;
mod single;
mod status;

pub use multi::MultiThreadDownloader;
pub use single::SingleThreadDownloader;
pub use status::{DownloadError, DownloadStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardcode_data::dev_samples::SAMPLE_SERVER_CORE_LFS_URL;
    use crate::hardcode_data::external_services::COMMON_HTTP_BROWSER_USER_AGENT;
    use std::time::Duration;

    /// 测试多线程下载
    #[tokio::test]
    async fn test_multi_thread_download() -> Result<(), String> {
        let downloader = MultiThreadDownloader::new(COMMON_HTTP_BROWSER_USER_AGENT);

        let url = SAMPLE_SERVER_CORE_LFS_URL;
        let save_path = "./target/multi_thread_download_test.bin";

        std::fs::create_dir_all("./target").map_err(|e| e.to_string())?;

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
}
