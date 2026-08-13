//! mclo.gs 日志分享服务。
//!
//! 将控制台/服务器日志上传到 mclo.gs 换取一个可分享的只读链接，
//! 便于在社区、Issue、群里贴日志求助，而无需直接粘贴大段文本。
//!
//! 参考：<https://mclo.gs/doc/api>

use sealantern_infra::net::global_client;
use serde::{Deserialize, Serialize};

const MCLOGS_API_URL: &str = "https://api.mclo.gs/1/log";
/// mclo.gs 限制单条日志最多 25,000 行，超出会上传失败。
const MAX_LOG_LINE_COUNT: usize = 25_000;
/// mclo.gs 限制单条日志最多约 10 MiB（UTF-8 字节长度），超出会上传失败。
const MAX_LOG_SIZE_BYTES: usize = 10 * 1024 * 1024;
/// 标识日志来源，会展示在 mclo.gs 页面上。
const SOURCE_NAME: &str = "SeaLantern";

#[derive(Debug, Deserialize)]
struct MclogsUploadResponse {
    success: bool,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct MclogsUploadRequest<'a> {
    content: &'a str,
    source: &'a str,
}

/// 超出行数限制时仅保留最后 `MAX_LOG_LINE_COUNT` 行。
///
/// 先统计总行数，再用 `skip` 顺序取末尾 N 行，
/// 避免双向反转与多余的中间分配。
fn truncate_to_last_lines(text: &str) -> String {
    let total_lines = text.lines().count();
    if total_lines > MAX_LOG_LINE_COUNT {
        text.lines()
            .skip(total_lines - MAX_LOG_LINE_COUNT)
            .collect::<Vec<&str>>()
            .join("\n")
    } else {
        text.to_string()
    }
}

/// 将日志文本上传到 mclo.gs 并返回分享链接。
///
/// - `content` 为空时直接返回错误，由调用方决定提示文案；
/// - 超过 `MAX_LOG_LINE_COUNT` 行时仅保留最后 N 行；
/// - 超过 `MAX_LOG_SIZE_BYTES` 时直接报错，避免被服务端拒绝；
/// - 通过 `sealantern_infra` 的进程级全局 `NetClient` 发送请求，
///   该客户端已按用户配置的代理设置构建，避免绕过代理导致上传失败。
pub async fn share_logs(content: String) -> Result<String, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("日志内容为空，无法分享".to_string());
    }

    // 行数截断（保留末尾 N 行）。
    let payload_content = truncate_to_last_lines(trimmed);

    // 大小限制：mclo.gs 单条上限约 10 MiB，超出会被拒绝，
    // 先本地拦截以给出明确错误，而不是等到服务端返回失败。
    if payload_content.len() > MAX_LOG_SIZE_BYTES {
        return Err(format!(
            "日志大小 {} 字节已超过 mclo.gs 上限 {} 字节，无法分享",
            payload_content.len(),
            MAX_LOG_SIZE_BYTES
        ));
    }

    // 复用进程级全局客户端（已应用代理设置），避免每次新建连接池，
    // 并保证请求经由用户配置的代理。
    let client = global_client().map_err(|e| format!("初始化网络客户端失败: {}", e))?;

    let response = client
        .post(MCLOGS_API_URL)
        .map_err(|e| format!("构建上传请求失败: {}", e))?
        .header("User-Agent", "SeaLantern")
        .json(&MclogsUploadRequest {
            content: &payload_content,
            source: SOURCE_NAME,
        })
        .map_err(|e| format!("序列化上传内容失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("上传日志到 mclo.gs 失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("mclo.gs 返回错误状态码: {}", response.status()));
    }

    let body: MclogsUploadResponse = response
        .json()
        .await
        .map_err(|e| format!("解析 mclo.gs 响应失败: {}", e))?;

    if !body.success {
        return Err(body.error.unwrap_or_else(|| "mclo.gs 拒绝上传，未知原因".to_string()));
    }

    body.url
        .ok_or_else(|| "mclo.gs 响应缺少 url 字段".to_string())
}
