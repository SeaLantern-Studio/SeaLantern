use super::common::{parse_params, CommandHandler, RegistryBuilder};
use crate::commands::plugins::manage as plugin_commands;
use crate::plugins::loader::PluginLoader;
use crate::services::global;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct PluginIdRequest {
    #[serde(alias = "pluginId")]
    plugin_id: String,
    confirmation: Option<crate::models::plugin::PluginEnableConfirmation>,
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

pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("list_plugins", handle_list_plugins as CommandHandler);
    builder.register("scan_plugins", handle_scan_plugins as CommandHandler);
    builder.register("enable_plugin", handle_enable_plugin as CommandHandler);
    builder.register("disable_plugin", handle_disable_plugin as CommandHandler);
    builder.register("get_plugin_nav_items", handle_get_plugin_nav_items as CommandHandler);
    builder.register("install_plugin", handle_install_plugin as CommandHandler);
    builder.register("install_plugins_batch", handle_install_plugins_batch as CommandHandler);
    builder.register("get_plugin_icon", handle_get_plugin_icon as CommandHandler);
    builder.register("get_plugin_settings", handle_get_plugin_settings as CommandHandler);
    builder.register("set_plugin_settings", handle_set_plugin_settings as CommandHandler);
    builder.register("get_plugin_css", handle_get_plugin_css as CommandHandler);
    builder.register("get_all_plugin_css", handle_get_all_plugin_css as CommandHandler);
    builder.register("check_plugin_update", handle_check_plugin_update as CommandHandler);
    builder.register("check_all_plugin_updates", handle_check_all_plugin_updates as CommandHandler);
    builder.register("delete_plugin", handle_delete_plugin as CommandHandler);
    builder.register("delete_plugins", handle_delete_plugins as CommandHandler);
    builder.register("fetch_market_plugins", handle_fetch_market_plugins as CommandHandler);
    builder.register("fetch_market_categories", handle_fetch_market_categories as CommandHandler);
    builder.register(
        "fetch_market_plugin_detail",
        handle_fetch_market_plugin_detail as CommandHandler,
    );
    builder.register("install_from_market", handle_install_from_market as CommandHandler);
    builder.register("context_menu_callback", handle_context_menu_callback as CommandHandler);
    builder.register("context_menu_show_notify", handle_context_menu_show_notify as CommandHandler);
    builder.register("context_menu_hide_notify", handle_context_menu_hide_notify as CommandHandler);
    builder.register("on_locale_changed", handle_on_locale_changed as CommandHandler);
    builder.register("get_locale_bundle", handle_get_locale_bundle as CommandHandler);
    builder
        .register("component_mirror_register", handle_component_mirror_register as CommandHandler);
    builder.register(
        "component_mirror_unregister",
        handle_component_mirror_unregister as CommandHandler,
    );
    builder.register("component_mirror_clear", handle_component_mirror_clear as CommandHandler);
    builder.register("on_page_changed", handle_on_page_changed as CommandHandler);
    builder.register("get_plugin_ui_snapshot", handle_get_plugin_ui_snapshot as CommandHandler);
    builder.register(
        "get_plugin_sidebar_snapshot",
        handle_get_plugin_sidebar_snapshot as CommandHandler,
    );
    builder.register(
        "get_plugin_context_menu_snapshot",
        handle_get_plugin_context_menu_snapshot as CommandHandler,
    );
    builder.register(
        "get_plugin_component_snapshot",
        handle_get_plugin_component_snapshot as CommandHandler,
    );
    builder.register(
        "get_plugin_permission_logs",
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

fn handle_get_locale_bundle(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let locale = params
            .get("locale")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());
        let i18n = global::i18n_service();
        let resolved_locale = locale.unwrap_or_else(|| i18n.get_locale());
        serde_json::to_value(crate::commands::app::i18n::LocaleBundleResponse {
            locale: resolved_locale.clone(),
            entries: i18n.get_translations_for_locale(&resolved_locale),
            available_locales: i18n.get_available_locales(),
        })
        .map_err(|error| error.to_string())
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
        let result = manager.enable_plugin(&req.plugin_id, req.confirmation)?;
        serde_json::to_value(result).map_err(|error| error.to_string())
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
        let result = manager
            .install_plugin(std::path::Path::new(&path))
            .map_err(|error| {
                if let Some(issue) = PluginLoader::classify_install_error(&error) {
                    return issue.into_command_error(error).to_json_string();
                }
                error
            })?;
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
                Err(error) => failed.push(crate::models::plugin::BatchInstallError {
                    path: path_str,
                    issue: PluginLoader::classify_install_error(&error),
                    error,
                }),
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
        let update_target = {
            let manager = global::plugin_manager();
            let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
            manager.check_plugin_update(&req.plugin_id)?
        };
        let result = match update_target {
            Some((plugin_id, current_version)) => {
                plugin_commands::market::check_plugin_update_without_manager(
                    current_version,
                    plugin_id,
                )
                .await?
            }
            None => None,
        };
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
            manager.collect_update_versions()
        };

        let result =
            plugin_commands::market::check_all_plugin_updates_without_manager(plugin_versions)
                .await?;
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
        let result =
            plugin_commands::market::fetch_market_plugins_without_manager(req.market_url).await?;
        serde_json::to_value(result).map_err(|error| error.to_string())
    })
}

fn handle_fetch_market_categories(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: MarketUrlRequest = parse_params(params)?;
        plugin_commands::market::fetch_market_categories(req.market_url).await
    })
}

fn handle_fetch_market_plugin_detail(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: MarketPluginDetailRequest = parse_params(params)?;
        plugin_commands::market::fetch_market_plugin_detail_without_manager(
            req.plugin_path,
            req.market_url,
        )
        .await
    })
}

fn handle_install_from_market(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: plugin_commands::market::InstallFromMarketRequest = parse_params(params)?;
        let result = plugin_commands::market::install_from_market_for_http(req).await?;
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
        manager.dispatch_context_menu_callback(
            &req.plugin_id,
            &req.context,
            &req.item_id,
            req.target_data,
        )?;
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
        manager.notify_context_menu_show(&req.context, &req.target_data, req.x, req.y);
        Ok(Value::Null)
    })
}

fn handle_context_menu_hide_notify(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let manager = global::plugin_manager();
        let manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        manager.notify_context_menu_hide();
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
