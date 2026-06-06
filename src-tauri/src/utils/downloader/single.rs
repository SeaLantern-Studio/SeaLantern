use reqwest::Client;
use std::time::Duration;

/// 单线程下载器
pub struct SingleThreadDownloader {
    client: Client,
}

/// 单线程下载实现
impl SingleThreadDownloader {
    /// 创建单线程下载器
    ///
    /// # Parameters
    ///
    /// - `user_agent`: 请求使用的浏览器标识
    pub fn new(user_agent: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent(user_agent)
                .build()
                .unwrap(),
        }
    }

    /// 读取远端文本内容
    ///
    /// # Parameters
    ///
    /// - `url`: 远端地址
    pub async fn read_to_string(&self, url: &str) -> Result<String, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("服务器返回错误状态码: {}", response.status()));
        }

        let content = response
            .text()
            .await
            .map_err(|e| format!("解析文本失败: {}", e))?;

        Ok(content)
    }
}
