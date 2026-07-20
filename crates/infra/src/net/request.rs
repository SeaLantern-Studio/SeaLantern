//! HTTP request building and sending.
//!
//! Provides a chainable request builder `RequestBuilder` with automatic retry support.
//! On each retry, the request is reconstructed from internally stored metadata,
//! avoiding the issue that `reqwest::Request` does not implement `Clone`.

use reqwest::{IntoUrl, Method, Response};
use serde::Serialize;

use crate::net::client::NetClient;
use crate::net::error::NetError;
use crate::net::RetryPolicy;
use crate::observability;

/// HTTP request builder.
///
/// Created via `NetClient::get()` / `NetClient::post()`,
/// supports chainable configuration of headers, body, retry policy,
/// and finally calls `.send()` to dispatch the request.
///
/// # Retry Behavior
///
/// | Condition | Behavior |
/// |-----------|----------|
/// | Timeout, connection failure, DNS error | Automatic retry |
/// | 5xx server error | Automatic retry |
/// | 4xx client error | No retry, return immediately |
/// | Request cancelled | No retry |
pub struct RequestBuilder<'a> {
    client: &'a NetClient,
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    retry_policy: RetryPolicy,
    // content-type is stored separately because it is needed when setting the body
    content_type: Option<String>,
}

impl<'a> RequestBuilder<'a> {
    /// Creates a new request builder.
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

    /// Adds a request header.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Sets the request body (text format).
    ///
    /// Automatically adds `Content-Type: text/plain` unless already set via `.header()`.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self.content_type.get_or_insert_with(|| "text/plain".into());
        self
    }

    /// Sets a JSON request body.
    ///
    /// Automatically adds `Content-Type: application/json` unless already set via `.header()`.
    pub fn json<T: Serialize>(mut self, value: &T) -> Result<Self, NetError> {
        let body = serde_json::to_string(value)
            .map_err(|e| NetError::Parse(format!("序列化 JSON 失败: {}", e)))?;
        self.body = Some(body);
        self.content_type
            .get_or_insert_with(|| "application/json".into());
        Ok(self)
    }

    /// Overrides the retry policy for this request.
    pub fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Dispatches the request with automatic retry policy.
    ///
    /// # Retry Logic
    ///
    /// 1. Compute backoff delay based on current attempt
    /// 2. Build and send the request via `client.get_reqwest_client()`
    /// 3. On success -> return `Ok(Response)`
    /// 4. On failure -> determine if retryable
    ///    - Retryable and under limit -> log event, wait, retry
    ///    - Not retryable or limit reached -> return `Err(NetError)`
    pub async fn send(self) -> Result<Response, NetError> {
        let max_attempts = self.retry_policy.max_retries + 1; // initial attempt + retries

        observability::request_started(&self.url);

        for attempt in 1..=max_attempts {
            // Build the reqwest request
            let mut req = self
                .client
                .get_reqwest_client()
                .request(self.method.clone(), &self.url);

            // Add request headers
            for (key, value) in &self.headers {
                req = req.header(key.as_str(), value.as_str());
            }

            // Add body and Content-Type
            if let Some(ref body) = self.body {
                if let Some(ref ct) = self.content_type {
                    req = req.header("Content-Type", ct.as_str());
                }
                req = req.body(body.clone());
            }

            // Dispatch the request
            match req.send().await {
                Ok(response) => {
                    // 5xx server errors are retryable
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

                    // Final response after retries exhausted: success (2xx/3xx) returns Ok, otherwise convert to error
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
                    // Determine if retryable
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

    /// Exponential backoff wait.
    async fn sleep(&self, attempt: u32) {
        let exponent = (attempt - 1).min(63); // prevent shift overflow
        let delay = self
            .retry_policy
            .base_delay
            .saturating_mul(2u32.saturating_pow(exponent));

        let delay = delay.min(self.retry_policy.max_delay);

        tokio::time::sleep(delay).await;
    }
}

/// Determines whether a `reqwest` error is retryable.
fn is_retryable(err: &reqwest::Error) -> bool {
    if err.is_timeout() {
        return true;
    }
    if err.is_connect() {
        return true;
    }
    false
}

/// Converts a `reqwest::Error` into `NetError`.
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
