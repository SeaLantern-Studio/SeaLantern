use crate::plugins;
use crate::plugins::runtime::PluginRuntime;
use crate::services;
use crate::services::global::i18n_service;
use crate::utils::logger::{log_debug_ctx, log_warn_ctx};
use sl_server_info::log::{parse_log_line, DomainEvent, LogLineInput, LogStream};

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tauri::{Emitter, Listener};

fn plugin_bridge_t(key: &str) -> String {
    i18n_service().t(key)
}

fn plugin_bridge_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

fn plugin_bridge_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

fn parse_element_response_payload(payload_json: &str) -> Result<(u64, String), String> {
    let payload = serde_json::from_str::<serde_json::Value>(payload_json).map_err(|error| {
        plugin_bridge_t1("plugin.bridge.element_response_parse_failed", error.to_string())
    })?;
    let request_id = payload
        .get("request_id")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| plugin_bridge_t("plugin.bridge.element_response_request_id_missing"))?;
    let data = payload
        .get("data")
        .and_then(|value| value.as_str())
        .ok_or_else(|| plugin_bridge_t("plugin.bridge.element_response_data_missing"))?;

    Ok((request_id, data.to_string()))
}

/// 连接插件事件桥。
pub(crate) fn install_plugin_bridge(
    app: &tauri::App,
    shared_runtimes: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    shared_runtimes_for_server_ready: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    api_registry: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
) {
    plugins::api::set_api_call_handler(Arc::new(move |_source, target, api_name, args| {
        use crate::plugins::api::ApiRegistryOps;

        let lua_fn_name = api_registry
            .get_api_fn_name(target, api_name)
            .ok_or_else(|| {
                plugin_bridge_t2("plugin.bridge.api_not_registered", target, api_name)
            })?;

        let runtimes = shared_runtimes.read().unwrap_or_else(|e| e.into_inner());
        let runtime = runtimes
            .get(target)
            .ok_or_else(|| plugin_bridge_t1("plugin.bridge.runtime_missing", target))?;
        runtime.call_registered_api(&lua_fn_name, args)
    }));

    let app_handle = app.handle().clone();
    plugins::api::set_ui_event_handler(Arc::new(move |plugin_id, action, element_id, html| {
        use serde::Serialize;

        #[derive(Serialize, Clone)]
        struct PluginUiEvent {
            plugin_id: String,
            action: String,
            element_id: String,
            html: String,
        }

        let event = PluginUiEvent {
            plugin_id: plugin_id.to_string(),
            action: action.to_string(),
            element_id: element_id.to_string(),
            html: html.to_string(),
        };

        app_handle
            .emit("plugin-ui-event", event)
            .map_err(|e| plugin_bridge_t1("plugin.bridge.emit_ui_failed", e.to_string()))
    }));

    let app_handle = app.handle().clone();
    plugins::api::set_log_event_handler(Arc::new(move |plugin_id, level, message| {
        use serde::Serialize;

        #[derive(Serialize, Clone)]
        struct PluginLogEvent {
            plugin_id: String,
            level: String,
            message: String,
        }

        let event = PluginLogEvent {
            plugin_id: plugin_id.to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };

        app_handle
            .emit("plugin-log-event", event)
            .map_err(|e| plugin_bridge_t1("plugin.bridge.emit_log_failed", e.to_string()))
    }));

    let app_handle = app.handle().clone();
    plugins::api::set_context_menu_handler(Arc::new(
        move |plugin_id, action, context, items_json| {
            use serde::Serialize;

            #[derive(Serialize, Clone)]
            struct PluginContextMenuEvent {
                plugin_id: String,
                action: String,
                context: String,
                items: String,
            }

            let event = PluginContextMenuEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                context: context.to_string(),
                items: items_json.to_string(),
            };

            app_handle
                .emit("plugin-context-menu-event", event)
                .map_err(|e| {
                    plugin_bridge_t1("plugin.bridge.emit_context_menu_failed", e.to_string())
                })
        },
    ));

    let app_handle = app.handle().clone();
    plugins::api::set_sidebar_event_handler(Arc::new(move |plugin_id, action, label, icon| {
        use serde::Serialize;

        #[derive(Serialize, Clone)]
        struct PluginSidebarEvent {
            plugin_id: String,
            action: String,
            label: String,
            icon: String,
        }

        let event = PluginSidebarEvent {
            plugin_id: plugin_id.to_string(),
            action: action.to_string(),
            label: label.to_string(),
            icon: icon.to_string(),
        };

        app_handle
            .emit("plugin-sidebar-event", event)
            .map_err(|e| plugin_bridge_t1("plugin.bridge.emit_sidebar_failed", e.to_string()))
    }));

    let app_handle = app.handle().clone();
    plugins::api::set_permission_log_handler(Arc::new(
        move |plugin_id, log_type, action, detail, timestamp| {
            use serde::Serialize;

            #[derive(Serialize, Clone)]
            struct PluginPermissionLog {
                plugin_id: String,
                log_type: String,
                action: String,
                detail: String,
                timestamp: u64,
            }

            let event = PluginPermissionLog {
                plugin_id: plugin_id.to_string(),
                log_type: log_type.to_string(),
                action: action.to_string(),
                detail: detail.to_string(),
                timestamp,
            };

            app_handle
                .emit("plugin-permission-log", event)
                .map_err(|e| {
                    plugin_bridge_t1("plugin.bridge.emit_permission_log_failed", e.to_string())
                })
        },
    ));

    let app_handle = app.handle().clone();
    plugins::api::set_component_event_handler(Arc::new(move |plugin_id, payload_json| {
        let val: serde_json::Value = serde_json::from_str(payload_json).map_err(|error| {
            plugin_bridge_t2(
                "plugin.bridge.component_event_parse_failed",
                plugin_id,
                error.to_string(),
            )
        })?;
        app_handle
            .emit("plugin:ui:component", val)
            .map_err(|e| plugin_bridge_t1("plugin.bridge.emit_component_failed", e.to_string()))
    }));

    let app_handle = app.handle().clone();
    plugins::api::set_i18n_event_handler(Arc::new(move |plugin_id, action, locale, payload| {
        use serde::Serialize;

        #[derive(Serialize, Clone)]
        struct PluginI18nEvent {
            plugin_id: String,
            action: String,
            locale: String,
            payload: String,
        }

        let event = PluginI18nEvent {
            plugin_id: plugin_id.to_string(),
            action: action.to_string(),
            locale: locale.to_string(),
            payload: payload.to_string(),
        };

        app_handle
            .emit("plugin-i18n-event", event)
            .map_err(|e| plugin_bridge_t1("plugin.bridge.emit_i18n_failed", e.to_string()))
    }));

    plugins::api::set_server_ready_handler(Arc::new(move |server_id| {
        let runtimes = shared_runtimes_for_server_ready
            .read()
            .unwrap_or_else(|e| e.into_inner());
        for (plugin_id, runtime) in runtimes.iter() {
            if let Err(e) = runtime.call_lifecycle_with_arg("onServerReady", server_id) {
                log_warn_ctx(
                    "runtime.plugin_bridge",
                    "set_server_ready_handler",
                    &format!("plugin '{}' onServerReady failed: {}", plugin_id, e),
                );
            }
        }
        Ok(())
    }));

    {
        let app_handle = app.handle().clone();
        app_handle.listen("plugin-element-response", |event| {
            log_debug_ctx(
                "runtime.plugin_bridge",
                "plugin_element_response",
                "received response event",
            );
            match parse_element_response_payload(event.payload()) {
                Ok((request_id, data)) => {
                    plugins::api::element_response_resolve(request_id, data);
                }
                Err(error) => {
                    log_warn_ctx(
                        "runtime.plugin_bridge",
                        "plugin_element_response",
                        &format!("{} payload={}", error, event.payload()),
                    );
                }
            }
        });
    }

    {
        use serde::Serialize;

        #[derive(Serialize, Clone)]
        struct ServerLogLineEvent {
            server_id: String,
            line: String,
        }

        #[derive(Serialize, Clone)]
        struct StructuredServerLogEvent {
            server_id: String,
            line: String,
            stream: String,
            event_kind: Option<String>,
            player: Option<String>,
            message: Option<String>,
        }

        let app_handle = app.handle().clone();
        let _ = services::server::log_pipeline::set_server_log_event_handler(Arc::new(
            move |server_id, line, stream| {
                let event = ServerLogLineEvent {
                    server_id: server_id.to_string(),
                    line: line.to_string(),
                };
                app_handle.emit("server-log-line", event).map_err(|e| {
                    plugin_bridge_t1("plugin.bridge.emit_server_log_line_failed", e.to_string())
                })?;

                let parsed = parse_log_line(
                    None,
                    LogLineInput {
                        raw: line.to_string(),
                        stream,
                    },
                );

                let (event_kind, player, message) = match parsed.event {
                    Some(DomainEvent::ServerReady) => (Some("server_ready".to_string()), None, None),
                    Some(DomainEvent::PlayerJoin { player }) => {
                        (Some("player_join".to_string()), Some(player), None)
                    }
                    Some(DomainEvent::PlayerLeave { player }) => {
                        (Some("player_leave".to_string()), Some(player), None)
                    }
                    Some(DomainEvent::Chat { player, message }) => {
                        (Some("chat".to_string()), Some(player), Some(message))
                    }
                    Some(DomainEvent::ErrorLike { message }) => {
                        (Some("error".to_string()), None, Some(message))
                    }
                    None => (None, None, None),
                };

                let structured_event = StructuredServerLogEvent {
                    server_id: server_id.to_string(),
                    line: line.to_string(),
                    stream: match stream {
                        LogStream::Stdout => "stdout".to_string(),
                        LogStream::Stderr => "stderr".to_string(),
                        LogStream::Unknown => "unknown".to_string(),
                    },
                    event_kind,
                    player,
                    message,
                };

                app_handle.emit("server-log-structured", structured_event).map_err(|e| {
                    plugin_bridge_t1("plugin.bridge.emit_server_log_line_failed", e.to_string())
                })
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::parse_element_response_payload;

    #[test]
    fn parse_element_response_payload_rejects_invalid_json() {
        let error = parse_element_response_payload("{")
            .expect_err("invalid element response payload should not be silently ignored");

        assert!(
            error.contains("payload") || error.contains("负载"),
            "unexpected error: {}",
            error
        );
    }

    #[test]
    fn parse_element_response_payload_rejects_missing_fields() {
        let error = parse_element_response_payload(r#"{"request_id":1}"#)
            .expect_err("missing data field should be surfaced");

        assert!(error.contains("data"));
    }
}
