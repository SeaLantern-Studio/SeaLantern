use crate::models::plugin::{PluginSettingField, PluginSettingOption};
use crate::plugins::manager::PluginManager;
use crate::services::events::ServerEventEnvelope;
use serde_json::{json, Value};

pub(crate) const PLUGIN_ID: &str = "sea-lantern-obv11-client";

pub(crate) fn manifest_settings() -> Vec<PluginSettingField> {
    vec![
        PluginSettingField {
            key: "enabled".to_string(),
            label: "Enable Service".to_string(),
            field_type: "boolean".to_string(),
            display: None,
            default: Some(json!(true)),
            description: Some("启用 OneBot v11 服务端与事件分发。".to_string()),
            options: None,
            rows: None,
            maxlength: None,
        },
        PluginSettingField {
            key: "mode".to_string(),
            label: "Mode".to_string(),
            field_type: "select".to_string(),
            display: None,
            default: Some(json!("api_only")),
            description: Some(
                "api_only 仅开放 API；qq_http_forward 额外把事件转发到 QQ OneBot HTTP API。"
                    .to_string(),
            ),
            options: Some(vec![
                PluginSettingOption {
                    value: "api_only".to_string(),
                    label: "API Only".to_string(),
                },
                PluginSettingOption {
                    value: "qq_http_forward".to_string(),
                    label: "QQ HTTP Forward".to_string(),
                },
            ]),
            rows: None,
            maxlength: None,
        },
        PluginSettingField {
            key: "listen_addr".to_string(),
            label: "Listen Addr".to_string(),
            field_type: "string".to_string(),
            display: None,
            default: Some(json!("127.0.0.1:5710")),
            description: Some("HTTP 与 WebSocket 共用监听地址。".to_string()),
            options: None,
            rows: None,
            maxlength: Some(128),
        },
        PluginSettingField {
            key: "access_token".to_string(),
            label: "Access Token".to_string(),
            field_type: "string".to_string(),
            display: None,
            default: Some(json!("")),
            description: Some("HTTP / WebSocket API 鉴权 token；为空则不鉴权。".to_string()),
            options: None,
            rows: None,
            maxlength: Some(256),
        },
        PluginSettingField {
            key: "qq_targets".to_string(),
            label: "QQ Targets".to_string(),
            field_type: "textarea".to_string(),
            display: None,
            default: Some(json!("")),
            description: Some("每行一个目标，格式 `group:123456` 或 `private:10001`。".to_string()),
            options: None,
            rows: Some(4),
            maxlength: Some(2048),
        },
    ]
}

pub(crate) fn default_settings_json() -> Value {
    json!({
        "enabled": true,
        "mode": "api_only",
        "listen_addr": "127.0.0.1:5710",
        "access_token": "",
        "self_id": 0,
        "enable_http_api": true,
        "enable_ws_api": true,
        "enable_http_post": false,
        "http_post_urls": "",
        "http_post_secret": "",
        "qq_api_base_url": "",
        "qq_access_token": "",
        "qq_targets": ""
    })
}

pub(crate) fn enable(_manager: &PluginManager, _plugin_id: &str) -> Result<(), String> {
    Err("SeaLantern OneBot v11 builtin plugin requires the 'docker' feature".to_string())
}

pub(crate) fn disable(_plugin_id: &str) {}

pub(crate) fn reload(_manager: &PluginManager, _plugin_id: &str) -> Result<(), String> {
    Ok(())
}

pub(crate) fn notify_server_event(_plugin_id: &str, _event: &ServerEventEnvelope) {}
