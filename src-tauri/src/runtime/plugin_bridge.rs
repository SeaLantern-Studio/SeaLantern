use crate::plugins;
use crate::services;

use std::sync::Arc;
use tauri::{Emitter, Listener};

/// 连接插件事件桥。
pub(crate) fn install_plugin_bridge(
    app: &tauri::App,
    shared_runtimes: crate::plugins::manager::SharedRuntimes,
    shared_runtimes_for_server_ready: crate::plugins::manager::SharedRuntimes,
    api_registry: crate::plugins::api::ApiRegistry,
) {
    plugins::api::set_api_call_handler(Arc::new(move |_source, target, api_name, args| {
        use crate::plugins::api::ApiRegistryOps;

        let lua_fn_name = api_registry
            .get_api_fn_name(target, api_name)
            .ok_or_else(|| format!("插件 '{}' 没有注册 API '{}'", target, api_name))?;

        let runtimes = shared_runtimes.read().unwrap_or_else(|e| e.into_inner());
        let runtime = runtimes
            .get(target)
            .ok_or_else(|| format!("插件 '{}' 的运行时不存在", target))?;
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
            .map_err(|e| format!("Failed to emit UI event: {}", e))
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
            .map_err(|e| format!("Failed to emit log event: {}", e))
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
                .map_err(|e| format!("Failed to emit context menu event: {}", e))
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
            .map_err(|e| format!("Failed to emit sidebar event: {}", e))
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
                .map_err(|e| format!("Failed to emit permission log: {}", e))
        },
    ));

    let app_handle = app.handle().clone();
    plugins::api::set_component_event_handler(Arc::new(move |_plugin_id, payload_json| {
        let val: serde_json::Value =
            serde_json::from_str(payload_json).unwrap_or(serde_json::Value::Null);
        app_handle
            .emit("plugin:ui:component", val)
            .map_err(|e| format!("Failed to emit component event: {}", e))
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
            .map_err(|e| format!("Failed to emit i18n event: {}", e))
    }));

    plugins::api::set_server_ready_handler(Arc::new(move |server_id| {
        let runtimes = shared_runtimes_for_server_ready
            .read()
            .unwrap_or_else(|e| e.into_inner());
        for (plugin_id, runtime) in runtimes.iter() {
            if let Err(e) = runtime.call_lifecycle_with_arg("onServerReady", server_id) {
                eprintln!("[WARN] plugin '{}' onServerReady failed: {}", plugin_id, e);
            }
        }
        Ok(())
    }));

    {
        let app_handle = app.handle().clone();
        app_handle.listen("plugin-element-response", |event| {
            eprintln!("[Element] Received response event");
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                if let (Some(request_id), Some(data)) = (
                    payload.get("request_id").and_then(|v| v.as_u64()),
                    payload.get("data").and_then(|v| v.as_str()),
                ) {
                    plugins::api::element_response_resolve(request_id, data.to_string());
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

        let app_handle = app.handle().clone();
        let _ = services::server::log_pipeline::set_server_log_event_handler(Arc::new(
            move |server_id, line| {
                let event = ServerLogLineEvent {
                    server_id: server_id.to_string(),
                    line: line.to_string(),
                };
                app_handle
                    .emit("server-log-line", event)
                    .map_err(|e| format!("Failed to emit server log line event: {}", e))
            },
        ));
    }
}
