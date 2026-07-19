//! HTTP 请求构建与发送。
//!
//! 提供链式调用的请求构建器 `RequestBuilder`，支持自动重试。
//! 每次重试时使用内部存储的元数据重新构建 `reqwest` 请求，
//! 避免 `reqwest::Request` 不实现 `Clone` 的问题。

use reqwest::{IntoUrl, Method, Response};
use serde::Serialize;

use crate::net::client::NetClient;
use crate::net::error::NetError;
use crate::net::RetryPolicy;
use crate::observability;

/// HTTP 请求构建器。
///
/// 通过 `NetClient::get()` / `NetClient::post()` 创建，
/// 支持链式配置 header、body、重试策略，最后调用 `.send()` 发起请求。
///
/// # 重试行为
///
/// | 条件 | 行为 |
/// |------|------|
/// | 超时、连接失败、DNS 错误 | 自动重试 |
/// | 5xx 服务端错误 | 自动重试 |
/// | 4xx 客户端错误 | 不重试，直接返回 |
/// | 请求被取消 | 不重试 |
pub struct RequestBuilder<'a> {
    client: &'a NetClient,
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    retry_policy: RetryPolicy,
    // content-type 单独存，因为设置 body 时会用到
    content_type: Option<String>,
}

impl<'a> RequestBuilder<'a> {
    /// 创建新的请求构建器。
    pub(super) fn new(
        client: &'a NetClient,
        method: Method,
        url: impl IntoUrl,
    ) -> Result<Self, NetError> {
        let url_str = url
            .into_url()
            .map_err(|e| NetError::Parse(format!("URL 格式无效: {}", e)))?
            .to_string();

        Ok(Self {
            client,
            method,
            url: url_str,
            headers: Vec::new(),
            body: None,
            retry_policy: *client.retry_policy(),
            content_type: None,
        })
    }

    /// 添加请求头。
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// 设置请求体（文本格式）。
    ///
    /// 自动添加 `Content-Type: text/plain`，除非已通过 `.header()` 手动设置。
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self.content_type.get_or_insert_with(|| "text/plain".into());
        self
    }

    /// 设置 JSON 请求体。
    ///
    /// 自动添加 `Content-Type: application/json`，除非已通过 `.header()` 手动设置。
    pub fn json<T: Serialize>(mut self, value: &T) -> Result<Self, NetError> {
        let body = serde_json::to_string(value)
            .map_err(|e| NetError::Parse(format!("序列化 JSON 失败: {}", e)))?;
        self.body = Some(body);
        self.content_type
            .get_or_insert_with(|| "application/json".into());
        Ok(self)
    }

    /// 覆盖本次请求的重试策略。
    pub fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// 发起请求，自动应用重试策略。
    ///
    /// # 重试逻辑
    ///
    /// 1. 用当前 attempt 计算退避延迟
    /// 2. 通过 `client.get_reqwest_client()` 构建并发送请求
    /// 3. 成功 → 返回 `Ok(Response)`
    /// 4. 失败 → 判断是否可重试
    ///    - 可重试且未达到上限 → 记录事件、等待、重试
    ///    - 不可重试或已达上限 → 返回 `Err(NetError)`
    pub async fn send(self) -> Result<Response, NetError> {
        let max_attempts = self.retry_policy.max_retries + 1; // 首次 + 重试次数

        observability::request_started(&self.url);

        for attempt in 1..=max_attempts {
            // 构建 reqwest 请求
            let mut req = self
                .client
                .get_reqwest_client()
                .request(self.method.clone(), &self.url);

            // 添加请求头
            for (key, value) in &self.headers {
                req = req.header(key.as_str(), value.as_str());
            }

            // 添加 body 和 Content-Type
            if let Some(ref body) = self.body {
                if let Some(ref ct) = self.content_type {
                    req = req.header("Content-Type", ct.as_str());
                }
                req = req.body(body.clone());
            }

            // 发起请求
            match req.send().await {
                Ok(response) => {
                    // 5xx 服务端错误可重试
                    if response.status().is_server_error() && attempt < max_attempts {
                        let status = response.status().as_u16();
                        observability::request_retry(
                            &self.url,
                            attempt,
                            &format!("服务端返回 {status}"),
                        );
                        self.sleep(attempt).await;
                        continue;
                    }

                    // 重试用完后的最终响应：成功（2xx/3xx）正常返回，否则转为错误
                    if response.status().is_success() || response.status().is_redirection() {
                        observability::request_completed(&self.url);
                        return Ok(response);
                    }
                    let status = response.status().as_u16();
                    let error = format!("HTTP {status}");
                    observability::request_failed(&self.url, &error);
                    return Err(NetError::Response(status, error));
                }
                Err(err) => {
                    // 判断是否可重试
                    if attempt < max_attempts && is_retryable(&err) {
                        observability::request_retry(&self.url, attempt, &err);
                        self.sleep(attempt).await;
                        continue;
                    }

                    let error = convert_reqwest_error(err);
                    observability::request_failed(&self.url, &error);
                    return Err(error);
                }
            }
        }

        unreachable!("循环逻辑保证至少执行一次")
    }

    /// 指数退避等待。
    async fn sleep(&self, attempt: u32) {
        let exponent = (attempt - 1).min(63); // 防止移位溢出
        let delay = self
            .retry_policy
            .base_delay
            .saturating_mul(2u32.saturating_pow(exponent));

        let delay = delay.min(self.retry_policy.max_delay);

        tokio::time::sleep(delay).await;
    }
}

/// 判断 `reqwest` 错误是否可重试。
fn is_retryable(err: &reqwest::Error) -> bool {
    if err.is_timeout() {
        return true;
    }
    if err.is_connect() {
        return true;
    }
    false
}

/// 将 `reqwest::Error` 转为 `NetError`。
fn convert_reqwest_error(err: reqwest::Error) -> NetError {
    if err.is_timeout() {
        NetError::Timeout
    } else if err.is_connect() {
        NetError::Request(format!("连接失败: {}", err))
    } else if err.is_status() {
        if let Some(status) = err.status() {
            NetError::Response(status.as_u16(), err.to_string())
        } else {
            NetError::Request(err.to_string())
        }
    } else {
        NetError::Request(err.to_string())
    }
}
