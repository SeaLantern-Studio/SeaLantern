use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

//由于本代码可能暂时未能投入使用，故加上#[allow(dead_code)]以消除提示
//在以后的开发中，如果使用到该下载器，请删去#[allow(dead_code)]
//示例：
//     let downloader = MultiPartDownloader::new(8); //下载的线程数，不建议过少/过多
//
//     let url = "https://download-cdn.jetbrains.com/rustrover/RustRover-2025.3.3.exe"; // 一个大文件
//     let save_path = "path\to\your\file";
//
//     match downloader.download(url, save_path).await {
//         Ok(_) => {
//             println!("\n 下载成功！文件已保存至: {}", save_path);
//         },
//         Err(e) => {
//             // 这里会捕获到具体的错误原因，比如 404、网络断开或权限不足
//             eprintln!("\n 下载中止: {}", e);
//
//             // 可以在这里清理未下载完的残留文件
//             if std::path::Path::new(save_path).exists() {
//                 let _ = std::fs::remove_file(save_path);
//                 println!("已清理不完整的文件。");
//             }
//         }
//     }

#[allow(dead_code)]
pub struct MultiThreadDownloader {
    client: Client,
    thread_count: usize,
}

#[allow(dead_code)]
impl MultiThreadDownloader {
    pub fn new(thread_count: usize) -> Self {
        Self { client: Client::new(), thread_count }
    }

    pub async fn download(
        &self,
        url: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let res = self.client.head(url).send().await?;
        let total_size = res
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| ct.parse::<u64>().ok())
            .ok_or("Server did not return Content-Length")?;

        let file = tokio::fs::File::create(output_path).await?;
        file.set_len(total_size).await?;

        // 初始化进度条管理器
        let m = MultiProgress::new();
        let main_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )?
        .progress_chars(">>-");

        let chunk_size = total_size / self.thread_count as u64;
        let mut tasks = Vec::new();
        let client = Arc::new(self.client.clone());

        for i in 0..self.thread_count {
            let start = i as u64 * chunk_size;
            let end = if i == self.thread_count - 1 {
                total_size - 1
            } else {
                start + chunk_size - 1
            };

            // 为每个分块创建一个进度条
            let pb = m.add(ProgressBar::new(end - start + 1));
            pb.set_style(main_style.clone());
            pb.set_message(format!("Chunk #{}", i));

            let url = url.to_string();
            let path = output_path.to_string();
            let client_ptr = Arc::clone(&client);

            tasks.push(tokio::spawn(async move {
                Self::download_range(client_ptr, url, path, start, end, pb).await
            }));
        }

        join_all(tasks).await;
        println!("\n所有分块下载完成！");
        Ok(())
    }

    async fn download_range(
        client: Arc<Client>,
        url: String,
        path: String,
        start: u64,
        end: u64,
        pb: ProgressBar, // 传入进度条句柄
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let range = format!("bytes={}-{}", start, end);
        let mut response = client.get(url).header("Range", range).send().await?;

        let mut file = OpenOptions::new().write(true).open(path).await?;
        file.seek(SeekFrom::Start(start)).await?;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            // 更新进度条位置
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Done");
        Ok(())
    }
}
