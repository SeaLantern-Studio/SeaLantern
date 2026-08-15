use sealantern_infra::net::global_client;
use serde::{Deserialize, Serialize};

use super::payload::prepare_payload;
use crate::observability;

const MCLOGS_API_URL: &str = "https://api.mclo.gs/1/log";
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

/// 将日志文本上传到 mclo.gs 并返回分享链接。
///
/// - `content` 为空时直接返回错误，由调用方决定提示文案；
/// - 超过 mclo.gs 行数限制时仅保留最后部分日志；
/// - 超过 mclo.gs 大小限制时直接报错，避免被服务端拒绝；
/// - 通过 `sealantern_infra` 的进程级全局 `NetClient` 发送请求，
///   该客户端已按用户配置的代理设置构建，避免绕过代理导致上传失败。
pub async fn share_logs(content: String) -> Result<String, String> {
    let payload_content = prepare_payload(&content)?;

    // 在发起请求前获取当前进程级全局客户端，确保使用最新的代理设置。
    let client = global_client().map_err(|e| {
        observability::mclogs_netclient_init_failed(&e);
        format!("初始化网络客户端失败: {}", e)
    })?;

    let response = client
        .post(MCLOGS_API_URL)
        .map_err(|e| {
            observability::mclogs_request_build_failed(&e);
            format!("构建上传请求失败: {}", e)
        })?
        .header("User-Agent", "SeaLantern")
        .json(&MclogsUploadRequest {
            content: &payload_content,
            source: SOURCE_NAME,
        })
        .map_err(|e| {
            observability::mclogs_serialize_failed(&e);
            format!("序列化上传内容失败: {}", e)
        })?
        .send()
        .await
        .map_err(|e| {
            observability::mclogs_upload_failed(&e);
            format!("上传日志到 mclo.gs 失败: {}", e)
        })?;

    if !response.status().is_success() {
        observability::mclogs_status_error(response.status().as_u16());
        return Err(format!("mclo.gs 返回错误状态码: {}", response.status()));
    }

    let body: MclogsUploadResponse = response.json().await.map_err(|e| {
        observability::mclogs_parse_failed(&e);
        format!("解析 mclo.gs 响应失败: {}", e)
    })?;

    if !body.success {
        let reason = body.error.clone().unwrap_or_else(|| "unknown".to_string());
        observability::mclogs_rejected(&reason);
        return Err(body
            .error
            .unwrap_or_else(|| "mclo.gs 拒绝上传，未知原因".to_string()));
    }

    let url = body.url.ok_or_else(|| {
        observability::mclogs_url_missing();
        "mclo.gs 响应缺少 url 字段".to_string()
    })?;

    observability::mclogs_shared(&url);
    Ok(url)
}
