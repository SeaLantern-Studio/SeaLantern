use super::common::{parse_params, CommandHandler, RegistryBuilder};
use super::requests::{
    AddExistingServerRequest, CollectCopyConflictsRequest, CopyDirectoryContentsRequest,
    CreateServerRequest, GetLogsRequest, GetServerStatusRequest, ImportModpackRequest,
    ImportServerRequest, ParseServerCoreTypeRequest, ScanStartupCandidatesRequest,
    SendCommandRequest, ServerIdRequest, UpdateJavaPathRequest, UpdateNameRequest,
};
use crate::commands::server::manage as server_commands;
use serde_json::Value;
pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("create_server", handle_create_server as CommandHandler);
    builder.register("import_server", handle_import_server as CommandHandler);
    builder.register("import_modpack", handle_import_modpack as CommandHandler);
    builder.register("start_server", handle_start_server as CommandHandler);
    builder.register("stop_server", handle_stop_server as CommandHandler);
    builder.register("send_command", handle_send_command as CommandHandler);
    builder.register("get_server_list", handle_get_server_list as CommandHandler);
    builder.register("get_server_status", handle_get_server_status as CommandHandler);
    builder.register("delete_server", handle_delete_server as CommandHandler);
    builder.register("get_server_logs", handle_get_server_logs as CommandHandler);
    builder.register("update_server_name", handle_update_server_name as CommandHandler);
    builder.register("update_server_java_path", handle_update_server_java_path as CommandHandler);
    builder.register("scan_startup_candidates", handle_scan_startup_candidates as CommandHandler);
    builder.register("parse_server_core_type", handle_parse_server_core_type as CommandHandler);
    builder.register("collect_copy_conflicts", handle_collect_copy_conflicts as CommandHandler);
    builder.register("copy_directory_contents", handle_copy_directory_contents as CommandHandler);
    builder.register("add_existing_server", handle_add_existing_server as CommandHandler);
}

fn handle_create_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: CreateServerRequest = parse_params(params)?;
        let result = server_commands::create_server(
            req.name,
            Some(req.aliases),
            req.core_type,
            req.mc_version,
            req.max_memory,
            req.min_memory,
            req.port,
            req.java_path,
            req.jar_path,
            req.server_path,
            req.startup_mode,
            req.custom_command,
            req.jvm_args,
            req.cpu_policy,
            req.jvm_preset,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_import_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ImportServerRequest = parse_params(params)?;
        let result = server_commands::import_server(
            req.name,
            req.jar_path,
            req.startup_mode,
            req.java_path,
            req.max_memory,
            req.min_memory,
            req.port,
            req.online_mode,
            req.jvm_args,
            req.cpu_policy,
            req.jvm_preset,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_import_modpack(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ImportModpackRequest = parse_params(params)?;
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
            req.jvm_args,
            req.cpu_policy,
            req.jvm_preset,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_start_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ServerIdRequest = parse_params(params)?;
        crate::services::global::server_manager().start_server(&req.id)?;
        Ok(Value::Null)
    })
}

fn handle_stop_server(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ServerIdRequest = parse_params(params)?;
        server_commands::stop_server(req.id)?;
        Ok(Value::Null)
    })
}

fn handle_send_command(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: SendCommandRequest = parse_params(params)?;
        server_commands::send_command(req.id, req.command)?;
        Ok(Value::Null)
    })
}

fn handle_get_server_list(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = server_commands::get_server_list_checked()?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_server_status(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: GetServerStatusRequest = parse_params(params)?;
        let result = crate::services::global::server_manager().get_server_status(&req.id);
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_delete_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ServerIdRequest = parse_params(params)?;
        server_commands::delete_server(req.id)?;
        Ok(Value::Null)
    })
}

fn handle_get_server_logs(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: GetLogsRequest = parse_params(params)?;
        let result = server_commands::get_server_logs(req.id, req.since, None)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_update_server_name(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: UpdateNameRequest = parse_params(params)?;
        server_commands::update_server_name(req.id, req.name)?;
        Ok(Value::Null)
    })
}

fn handle_update_server_java_path(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: UpdateJavaPathRequest = parse_params(params)?;
        let result = server_commands::update_server_java_path(req.id, req.java_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_scan_startup_candidates(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ScanStartupCandidatesRequest = parse_params(params)?;
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
        let req: ParseServerCoreTypeRequest = parse_params(params)?;
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
        let req: CollectCopyConflictsRequest = parse_params(params)?;
        let result = server_commands::collect_copy_conflicts(req.source_dir, req.target_dir)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_copy_directory_contents(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: CopyDirectoryContentsRequest = parse_params(params)?;
        server_commands::copy_directory_contents(req.source_dir, req.target_dir)?;
        Ok(Value::Null)
    })
}

fn handle_add_existing_server(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: AddExistingServerRequest = parse_params(params)?;
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
            req.core_type,
            req.mc_version,
            req.jvm_args,
            req.cpu_policy,
            req.jvm_preset,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}
