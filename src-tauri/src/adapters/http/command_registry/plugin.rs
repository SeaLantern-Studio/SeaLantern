use super::common::{parse_params, CommandHandler};
use crate::commands::plugins::manage as plugin_commands;
use crate::services::global;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize)]
struct PluginIdRequest {
    #[serde(alias = "pluginId")]
    plugin_id: String,
}

#[derive(Deserialize)]
struct PluginSettingsRequest {
    #[serde(alias = "pluginId")]
    plugin_id: String,
    settings: serde_json::Value,
}

#[derive(Deserialize)]
struct DeletePluginRequest {
    #[serde(alias = "pluginId")]
    plugin_id: String,
    #[serde(alias = "deleteData")]
    delete_data: Option<bool>,
}

#[derive(Deserialize)]
struct DeletePluginsRequest {
    #[serde(alias = "pluginIds")]
    plugin_ids: Vec<String>,
    #[serde(alias = "deleteData")]
    delete_data: Option<bool>,
}

#[derive(Deserialize)]
struct BatchInstallRequest {
    paths: Vec<String>,
}

#[derive(Deserialize)]
struct ContextMenuShowRequest {
    context: String,
    #[serde(alias = "targetData")]
    target_data: serde_json::Value,
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
struct ContextMenuCallbackRequest {
    #[serde(alias = "pluginId")]
    plugin_id: String,
    context: String,
    #[serde(alias = "itemId")]
    item_id: String,
    #[serde(alias = "targetData")]
    target_data: serde_json::Value,
}

#[derive(Deserialize)]
struct LocaleChangedRequest {
    locale: String,
}

#[derive(Deserialize)]
struct ComponentMirrorRegisterRequest {
    id: String,
    #[serde(alias = "componentType")]
    component_type: String,
}

#[derive(Deserialize)]
struct ComponentMirrorIdRequest {
    id: String,
}

#[derive(Deserialize)]
struct MarketUrlRequest {
    #[serde(alias = "marketUrl")]
    market_url: Option<String>,
}

#[derive(Deserialize)]
struct MarketPluginDetailRequest {
    #[serde(alias = "pluginPath")]
    plugin_path: String,
    #[serde(alias = "marketUrl")]
    market_url: Option<String>,
}

#[derive(Deserialize)]
struct PageChangedRequest {
    path: String,
}

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("list_plugins".to_string(), handle_list_plugins as CommandHandler);
    handlers.insert("scan_plugins".to_string(), handle_scan_plugins as CommandHandler);
    handlers.insert("enable_plugin".to_string(), handle_enable_plugin as CommandHandler);
    handlers.insert("disable_plugin".to_string(), handle_disable_plugin as CommandHandler);
    handlers.insert(
        "get_plugin_nav_items".to_string(),
        handle_get_plugin_nav_items as CommandHandler,
    );
    handlers.insert("install_plugin".to_string(), handle_install_plugin as CommandHandler);
    handlers.insert(
        "install_plugins_batch".to_string(),
        handle_install_plugins_batch as CommandHandler,
    );
    handlers.insert("get_plugin_icon".to_string(), handle_get_plugin_icon as CommandHandler);
    handlers
        .insert("get_plugin_settings".to_string(), handle_get_plugin_settings as CommandHandler);
    handlers
        .insert("set_plugin_settings".to_string(), handle_set_plugin_settings as CommandHandler);
    handlers.insert("get_plugin_css".to_string(), handle_get_plugin_css as CommandHandler);
    handlers.insert("get_all_plugin_css".to_string(), handle_get_all_plugin_css as CommandHandler);
    handlers
        .insert("check_plugin_update".to_string(), handle_check_plugin_update as CommandHandler);
    handlers.insert(
        "check_all_plugin_updates".to_string(),
        handle_check_all_plugin_updates as CommandHandler,
    );
    handlers.insert("delete_plugin".to_string(), handle_delete_plugin as CommandHandler);
    handlers.insert("delete_plugins".to_string(), handle_delete_plugins as CommandHandler);
    handlers.insert(
        "fetch_market_plugins".to_string(),
        handle_fetch_market_plugins as CommandHandler,
    );
    handlers.insert(
        "fetch_market_categories".to_string(),
        handle_fetch_market_categories as CommandHandler,
    );
    handlers.insert(
        "fetch_market_plugin_detail".to_string(),
        handle_fetch_market_plugin_detail as CommandHandler,
    );
    handlers
        .insert("install_from_market".to_string(), handle_install_from_market as CommandHandler);
    handlers.insert(
        "context_menu_callback".to_string(),
        handle_context_menu_callback as CommandHandler,
    );
    handlers.insert(
        "context_menu_show_notify".to_string(),
        handle_context_menu_show_notify as CommandHandler,
    );
    handlers.insert(
        "context_menu_hide_notify".to_string(),
        handle_context_menu_hide_notify as CommandHandler,
    );
    handlers.insert("on_locale_changed".to_string(), handle_on_locale_changed as CommandHandler);
    handlers.insert(
        "component_mirror_register".to_string(),
        handle_component_mirror_register as CommandHandler,
    );
    handlers.insert(
        "component_mirror_unregister".to_string(),
        handle_component_mirror_unregister as CommandHandler,
    );
    handlers.insert(
        "component_mirror_clear".to_string(),
        handle_component_mirror_clear as CommandHandler,
    );
    handlers.insert("on_page_changed".to_string(), handle_on_page_changed as CommandHandler);
    handlers.insert(
        "get_plugin_ui_snapshot".to_string(),
        handle_get_plugin_ui_snapshot as CommandHandler,
    );
    handlers.insert(
        "get_plugin_sidebar_snapshot".to_string(),
        handle_get_plugin_sidebar_snapshot as CommandHandler,
    );
    handlers.insert(
        "get_plugin_context_menu_snapshot".to_string(),
        handle_get_plugin_context_menu_snapshot as CommandHandler,
    );
    handlers.insert(
        "get_plugin_component_snapshot".to_string(),
        handle_get_plugin_component_snapshot as CommandHandler,
    );
    handlers.insert(
        "get_plugin_permission_logs".to_string(),
        handle_get_plugin_permission_logs as CommandHandler,
    );
}

fn handle_list_plugins(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        serde_json::to_value(manager.get_plugin_list()).map_err(|error| error.to_string())
    })
}

fn handle_scan_plugins(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let result = manager.scan_plugins()?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_enable_plugin(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.enable_plugin(&req.plugin_id)?;
        Ok(Value::Null)
    })
}

fn handle_disable_plugin(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let result = manager.disable_plugin(&req.plugin_id)?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_nav_items(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        serde_json::to_value(manager.get_nav_items()).map_err(|error| error.to_string())
    })
}

fn handle_install_plugin(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let path = params
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing path".to_string())?
            .to_string();
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let result = manager.install_plugin(std::path::Path::new(&path))?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_install_plugins_batch(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: BatchInstallRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());

        let mut success = Vec::new();
        let mut failed = Vec::new();
        for path_str in req.paths {
            let path = std::path::PathBuf::from(&path_str);
            match manager.install_plugin(&path) {
                Ok(install_result) => success.push(install_result),
                Err(error) => {
                    failed.push(crate::models::plugin::BatchInstallError { path: path_str, error })
                }
            }
        }

        let result = crate::models::plugin::BatchInstallResult { success, failed };
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_icon(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let result = manager.get_plugin_icon(&req.plugin_id)?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.get_plugin_settings(&req.plugin_id)
    })
}

fn handle_set_plugin_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginSettingsRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.set_plugin_settings(&req.plugin_id, req.settings)?;
        Ok(Value::Null)
    })
}

fn handle_get_plugin_css(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let result = manager.get_plugin_css(&req.plugin_id)?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_get_all_plugin_css(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let result = manager.get_all_plugin_css()?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_check_plugin_update(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let current_version = {
            let manager = global::plugin_manager();
            let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
            let plugin_info = manager
                .plugins()
                .get(&req.plugin_id)
                .ok_or_else(|| format!("Plugin not found: {}", req.plugin_id))?;
            plugin_info.manifest.version.clone()
        };

        let result =
            plugin_commands::check_plugin_update_for_http(current_version, req.plugin_id).await?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_check_all_plugin_updates(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let plugin_versions = {
            let manager = global::plugin_manager();
            let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
            manager
                .plugins()
                .iter()
                .map(|(id, info)| (id.clone(), info.manifest.version.clone()))
                .collect::<Vec<_>>()
        };

        let result = plugin_commands::check_all_plugin_updates_for_http(plugin_versions).await?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_delete_plugin(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: DeletePluginRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.delete_plugin(&req.plugin_id, req.delete_data.unwrap_or(false))?;
        Ok(Value::Null)
    })
}

fn handle_delete_plugins(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: DeletePluginsRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let mut manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let delete_data = req.delete_data.unwrap_or(false);
        for plugin_id in req.plugin_ids {
            manager.delete_plugin(&plugin_id, delete_data)?;
        }
        Ok(Value::Null)
    })
}

fn handle_fetch_market_plugins(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: MarketUrlRequest = parse_params(params)?;
        let result = plugin_commands::fetch_market_plugins_for_http(req.market_url).await?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_fetch_market_categories(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: MarketUrlRequest = parse_params(params)?;
        plugin_commands::fetch_market_categories_for_http(req.market_url).await
    })
}

fn handle_fetch_market_plugin_detail(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: MarketPluginDetailRequest = parse_params(params)?;
        plugin_commands::fetch_market_plugin_detail_for_http(req.plugin_path, req.market_url).await
    })
}

fn handle_install_from_market(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: crate::commands::plugins::manage::InstallFromMarketRequest = parse_params(params)?;
        let result = plugin_commands::install_from_market_for_http(req).await?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_context_menu_callback(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ContextMenuCallbackRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let runtimes = manager.get_shared_runtimes();
        let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());
        let runtime = runtimes_guard
            .get(&req.plugin_id)
            .ok_or_else(|| format!("插件 '{}' 的运行时不存在", req.plugin_id))?;
        runtime.call_context_menu_callback(&req.context, &req.item_id, req.target_data)?;
        Ok(Value::Null)
    })
}

fn handle_context_menu_show_notify(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ContextMenuShowRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let runtimes = manager.get_shared_runtimes();
        let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

        for runtime in runtimes_guard.values() {
            let _ = runtime.call_context_menu_show_callback(
                &req.context,
                req.target_data.clone(),
                req.x,
                req.y,
            );
        }

        Ok(Value::Null)
    })
}

fn handle_context_menu_hide_notify(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        let runtimes = manager.get_shared_runtimes();
        let runtimes_guard = runtimes.read().unwrap_or_else(|e| e.into_inner());

        for runtime in runtimes_guard.values() {
            let _ = runtime.call_context_menu_hide_callback();
        }

        Ok(Value::Null)
    })
}

fn handle_on_locale_changed(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: LocaleChangedRequest = parse_params(params)?;
        crate::services::global::i18n_service().set_locale(&req.locale);
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.notify_locale_changed(&req.locale);
        Ok(Value::Null)
    })
}

fn handle_component_mirror_register(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ComponentMirrorRegisterRequest = parse_params(params)?;
        crate::plugins::api::component_mirror_register(&req.id, &req.component_type);
        Ok(Value::Null)
    })
}

fn handle_component_mirror_unregister(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ComponentMirrorIdRequest = parse_params(params)?;
        crate::plugins::api::component_mirror_unregister(&req.id);
        Ok(Value::Null)
    })
}

fn handle_component_mirror_clear(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        crate::plugins::api::component_mirror_clear();
        Ok(Value::Null)
    })
}

fn handle_on_page_changed(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PageChangedRequest = parse_params(params)?;
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.notify_page_changed(&req.path);
        Ok(Value::Null)
    })
}

fn handle_get_plugin_ui_snapshot(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        serde_json::to_value(crate::plugins::api::take_ui_event_snapshot())
            .map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_sidebar_snapshot(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        serde_json::to_value(crate::plugins::api::take_sidebar_event_snapshot())
            .map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_context_menu_snapshot(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        serde_json::to_value(crate::plugins::api::take_context_menu_snapshot())
            .map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_component_snapshot(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        serde_json::to_value(crate::plugins::api::take_component_event_snapshot())
            .map_err(|error| error.to_string())
    })
}

fn handle_get_plugin_permission_logs(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PluginIdRequest = parse_params(params)?;
        let result = crate::plugins::api::get_plugin_permission_logs(&req.plugin_id);
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}
