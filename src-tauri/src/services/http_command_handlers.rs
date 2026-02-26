use crate::commands::config as config_commands;
use crate::commands::java as java_commands;
use crate::commands::player as player_commands;
use crate::commands::server as server_commands;
use crate::commands::settings as settings_commands;
use crate::commands::system as system_commands;
use crate::commands::update as update_commands;
use crate::models::settings::AppSettings;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// HTTP API 命令处理器类型（不使用 AppHandle，因为 HTTP 模式下不需要）
pub type CommandHandler = fn(Value) -> futures::future::BoxFuture<'static, Result<Value, String>>;

/// 命令注册表
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut handlers: HashMap<String, CommandHandler> = HashMap::new();

        // 注册 Server 命令
        handlers.insert("create_server".to_string(), handle_create_server as CommandHandler);
        handlers.insert("import_server".to_string(), handle_import_server as CommandHandler);
        handlers.insert("import_modpack".to_string(), handle_import_modpack as CommandHandler);
        handlers.insert("start_server".to_string(), handle_start_server as CommandHandler);
        handlers.insert("stop_server".to_string(), handle_stop_server as CommandHandler);
        handlers.insert("send_command".to_string(), handle_send_command as CommandHandler);
        handlers.insert("get_server_list".to_string(), handle_get_server_list as CommandHandler);
        handlers
            .insert("get_server_status".to_string(), handle_get_server_status as CommandHandler);
        handlers.insert("delete_server".to_string(), handle_delete_server as CommandHandler);
        handlers.insert("get_server_logs".to_string(), handle_get_server_logs as CommandHandler);
        handlers
            .insert("update_server_name".to_string(), handle_update_server_name as CommandHandler);
        handlers.insert(
            "scan_startup_candidates".to_string(),
            handle_scan_startup_candidates as CommandHandler,
        );
        handlers.insert(
            "parse_server_core_type".to_string(),
            handle_parse_server_core_type as CommandHandler,
        );
        handlers.insert(
            "collect_copy_conflicts".to_string(),
            handle_collect_copy_conflicts as CommandHandler,
        );
        handlers.insert(
            "copy_directory_contents".to_string(),
            handle_copy_directory_contents as CommandHandler,
        );
        handlers.insert(
            "add_existing_server".to_string(),
            handle_add_existing_server as CommandHandler,
        );

        // 注册 Java 命令
        handlers.insert("detect_java".to_string(), handle_detect_java as CommandHandler);
        handlers
            .insert("validate_java_path".to_string(), handle_validate_java_path as CommandHandler);
        // 注意：install_java 和 cancel_java_install 需要特殊处理，暂时不支持
        handlers.insert(
            "cancel_java_install".to_string(),
            handle_cancel_java_install as CommandHandler,
        );

        // 注册 Config 命令
        handlers.insert("read_config".to_string(), handle_read_config as CommandHandler);
        handlers.insert("write_config".to_string(), handle_write_config as CommandHandler);
        handlers.insert(
            "read_server_properties".to_string(),
            handle_read_server_properties as CommandHandler,
        );
        handlers.insert(
            "write_server_properties".to_string(),
            handle_write_server_properties as CommandHandler,
        );

        // 注册 System 命令
        handlers.insert("get_system_info".to_string(), handle_get_system_info as CommandHandler);
        // 注意：文件选择器命令在 HTTP 模式下不支持
        handlers.insert("pick_jar_file".to_string(), handle_unsupported as CommandHandler);
        handlers.insert("pick_startup_file".to_string(), handle_unsupported as CommandHandler);
        handlers.insert("pick_java_file".to_string(), handle_unsupported as CommandHandler);
        handlers.insert("pick_folder".to_string(), handle_unsupported as CommandHandler);
        handlers.insert("pick_image_file".to_string(), handle_unsupported as CommandHandler);

        // 注册 Player 命令
        handlers.insert("get_whitelist".to_string(), handle_get_whitelist as CommandHandler);
        handlers
            .insert("get_banned_players".to_string(), handle_get_banned_players as CommandHandler);
        handlers.insert("get_ops".to_string(), handle_get_ops as CommandHandler);
        handlers.insert("add_to_whitelist".to_string(), handle_add_to_whitelist as CommandHandler);
        handlers.insert(
            "remove_from_whitelist".to_string(),
            handle_remove_from_whitelist as CommandHandler,
        );
        handlers.insert("ban_player".to_string(), handle_ban_player as CommandHandler);
        handlers.insert("unban_player".to_string(), handle_unban_player as CommandHandler);
        handlers.insert("add_op".to_string(), handle_add_op as CommandHandler);
        handlers.insert("remove_op".to_string(), handle_remove_op as CommandHandler);
        handlers.insert("kick_player".to_string(), handle_kick_player as CommandHandler);
        handlers.insert("export_logs".to_string(), handle_export_logs as CommandHandler);

        // 注册 Settings 命令
        handlers.insert("get_settings".to_string(), handle_get_settings as CommandHandler);
        handlers.insert("save_settings".to_string(), handle_save_settings as CommandHandler);
        handlers.insert("reset_settings".to_string(), handle_reset_settings as CommandHandler);
        handlers.insert("export_settings".to_string(), handle_export_settings as CommandHandler);
        handlers.insert("import_settings".to_string(), handle_import_settings as CommandHandler);
        handlers.insert(
            "check_acrylic_support".to_string(),
            handle_check_acrylic_support as CommandHandler,
        );
        handlers.insert("apply_acrylic".to_string(), handle_apply_acrylic as CommandHandler);
        handlers.insert("get_system_fonts".to_string(), handle_get_system_fonts as CommandHandler);

        // 注册 Update 命令
        handlers.insert("check_update".to_string(), handle_check_update as CommandHandler);
        handlers
            .insert("open_download_url".to_string(), handle_open_download_url as CommandHandler);

        // 注意：Plugin 命令暂时不支持，需要插件管理器在 HTTP 模式下的支持
        let plugin_commands = vec![
            "list_plugins",
            "scan_plugins",
            "enable_plugin",
            "disable_plugin",
            "get_plugin_nav_items",
            "install_plugin",
            "get_plugin_icon",
            "get_plugin_settings",
            "set_plugin_settings",
            "get_plugin_css",
            "get_all_plugin_css",
            "delete_plugin",
            "delete_plugins",
            "check_plugin_update",
            "check_all_plugin_updates",
            "fetch_market_plugins",
            "fetch_market_categories",
            "fetch_market_plugin_detail",
            "install_from_market",
            "install_plugins_batch",
            "context_menu_callback",
            "context_menu_show_notify",
            "context_menu_hide_notify",
            "on_locale_changed",
            "component_mirror_register",
            "component_mirror_unregister",
            "component_mirror_clear",
            "on_page_changed",
            "get_plugin_component_snapshot",
            "get_plugin_ui_snapshot",
            "get_plugin_sidebar_snapshot",
            "get_plugin_context_menu_snapshot",
        ];
        for cmd in plugin_commands {
            handlers.insert(cmd.to_string(), handle_unsupported as CommandHandler);
        }

        Self { handlers }
    }

    pub fn get_handler(&self, command: &str) -> Option<&CommandHandler> {
        self.handlers.get(command)
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Server 命令处理器 ============

fn handle_create_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: CreateServerRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::create_server(
            req.name,
            req.core_type,
            req.mc_version,
            req.max_memory,
            req.min_memory,
            req.port,
            req.java_path,
            req.jar_path,
            req.startup_mode,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_import_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ImportServerRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::import_server(
            req.name,
            req.jar_path,
            req.startup_mode,
            req.java_path,
            req.max_memory,
            req.min_memory,
            req.port,
            req.online_mode,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_import_modpack(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ImportModpackRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::import_modpack(
            req.name,
            req.modpack_path,
            req.java_path,
            req.max_memory,
            req.min_memory,
            req.port,
            req.startup_mode,
            req.online_mode,
            req.custom_command,
            req.run_path,
            req.startup_file_path,
            req.core_type,
            req.mc_version,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_start_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let id: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        server_commands::start_server(id)?;
        Ok(Value::Null)
    })
}

fn handle_stop_server(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let id: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        server_commands::stop_server(id)?;
        Ok(Value::Null)
    })
}

fn handle_send_command(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: SendCommandRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        server_commands::send_command(req.id, req.command)?;
        Ok(Value::Null)
    })
}

fn handle_get_server_list(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = server_commands::get_server_list();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_server_status(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let id: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::get_server_status(id);
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_delete_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let id: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        server_commands::delete_server(id)?;
        Ok(Value::Null)
    })
}

fn handle_get_server_logs(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: GetLogsRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::get_server_logs(req.id, req.since);
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_update_server_name(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: UpdateNameRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        server_commands::update_server_name(req.id, req.name)?;
        Ok(Value::Null)
    })
}

fn handle_scan_startup_candidates(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ScanStartupCandidatesRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::scan_startup_candidates(req.source_path, req.source_type)
            .await
            .map_err(|e| format!("Failed to scan startup candidates: {}", e))?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_parse_server_core_type(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ParseServerCoreTypeRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::parse_server_core_type(req.source_path)
            .await
            .map_err(|e| format!("Failed to parse server core type: {}", e))?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_collect_copy_conflicts(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: CollectCopyConflictsRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::collect_copy_conflicts(req.source_dir, req.target_dir)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_copy_directory_contents(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: CopyDirectoryContentsRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        server_commands::copy_directory_contents(req.source_dir, req.target_dir)?;
        Ok(Value::Null)
    })
}

fn handle_add_existing_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: AddExistingServerRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = server_commands::add_existing_server(
            req.name,
            req.server_path,
            req.java_path,
            req.max_memory,
            req.min_memory,
            req.port,
            req.startup_mode,
            req.executable_path,
            req.custom_command,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

// ============ Java 命令处理器 ============

fn handle_detect_java(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = java_commands::detect_java().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_validate_java_path(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let path: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = java_commands::validate_java_path(path).await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_cancel_java_install(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        java_commands::cancel_java_install().await?;
        Ok(Value::Null)
    })
}

// ============ Config 命令处理器 ============

fn handle_read_config(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ReadConfigRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = config_commands::read_config(req.server_path, req.path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_config(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteConfigRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        config_commands::write_config(req.server_path, req.path, req.values)?;
        Ok(Value::Null)
    })
}

fn handle_read_server_properties(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let server_path: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = config_commands::read_server_properties(server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_server_properties(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteServerPropertiesRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        config_commands::write_server_properties(req.server_path, req.values)?;
        Ok(Value::Null)
    })
}

// ============ System 命令处理器 ============

fn handle_get_system_info(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = system_commands::get_system_info()?;
        Ok(result)
    })
}

fn handle_unsupported(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move { Err("This command is not supported in HTTP/Docker mode".to_string()) })
}

// ============ Player 命令处理器 ============

fn handle_get_whitelist(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let server_path: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::get_whitelist(server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_banned_players(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let server_path: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::get_banned_players(server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_ops(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let server_path: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::get_ops(server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_add_to_whitelist(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::add_to_whitelist(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_remove_from_whitelist(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::remove_from_whitelist(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_ban_player(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: BanPlayerRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::ban_player(req.server_id, req.name, req.reason)?;
        Ok(Value::String(result))
    })
}

fn handle_unban_player(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::unban_player(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_add_op(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::add_op(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_remove_op(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::remove_op(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_kick_player(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: KickPlayerRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = player_commands::kick_player(req.server_id, req.name, req.reason)?;
        Ok(Value::String(result))
    })
}

fn handle_export_logs(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ExportLogsRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        player_commands::export_logs(req.logs, req.save_path)?;
        Ok(Value::Null)
    })
}

// ============ Settings 命令处理器 ============

fn handle_get_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_settings();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_save_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let settings: AppSettings =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        settings_commands::save_settings(settings)?;
        Ok(Value::Null)
    })
}

fn handle_reset_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::reset_settings()?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_export_settings(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::export_settings()?;
        Ok(Value::String(result))
    })
}

fn handle_import_settings(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let json: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        let result = settings_commands::import_settings(json)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_check_acrylic_support(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err(
            "check_acrylic_support is not supported in HTTP/Docker mode (requires Window handle)"
                .to_string(),
        )
    })
}

fn handle_apply_acrylic(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        Err("apply_acrylic is not supported in HTTP/Docker mode (requires Window handle)"
            .to_string())
    })
}

fn handle_get_system_fonts(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = settings_commands::get_system_fonts();
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

// ============ Update 命令处理器 ============

fn handle_check_update(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = update_commands::check_update().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_open_download_url(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let url: String =
            serde_json::from_value(params).map_err(|e| format!("Invalid parameters: {}", e))?;
        update_commands::open_download_url(url).await?;
        Ok(Value::Null)
    })
}

// ============ 请求结构体 ============

#[derive(Debug, Deserialize)]
struct CreateServerRequest {
    name: String,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
    startup_mode: String,
}

#[derive(Debug, Deserialize)]
struct ImportServerRequest {
    name: String,
    jar_path: String,
    startup_mode: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    online_mode: bool,
}

#[derive(Debug, Deserialize)]
struct ImportModpackRequest {
    name: String,
    modpack_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    online_mode: bool,
    custom_command: Option<String>,
    run_path: String,
    startup_file_path: Option<String>,
    core_type: Option<String>,
    mc_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SendCommandRequest {
    id: String,
    command: String,
}

#[derive(Debug, Deserialize)]
struct GetLogsRequest {
    id: String,
    since: usize,
}

#[derive(Debug, Deserialize)]
struct UpdateNameRequest {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ScanStartupCandidatesRequest {
    source_path: String,
    source_type: String,
}

#[derive(Debug, Deserialize)]
struct ParseServerCoreTypeRequest {
    source_path: String,
}

#[derive(Debug, Deserialize)]
struct CollectCopyConflictsRequest {
    source_dir: String,
    target_dir: String,
}

#[derive(Debug, Deserialize)]
struct CopyDirectoryContentsRequest {
    source_dir: String,
    target_dir: String,
}

#[derive(Debug, Deserialize)]
struct AddExistingServerRequest {
    name: String,
    server_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    executable_path: Option<String>,
    custom_command: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReadConfigRequest {
    server_path: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct WriteConfigRequest {
    server_path: String,
    path: String,
    values: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct WriteServerPropertiesRequest {
    server_path: String,
    values: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct PlayerActionRequest {
    server_id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct BanPlayerRequest {
    server_id: String,
    name: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct KickPlayerRequest {
    server_id: String,
    name: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct ExportLogsRequest {
    logs: Vec<String>,
    save_path: String,
}
