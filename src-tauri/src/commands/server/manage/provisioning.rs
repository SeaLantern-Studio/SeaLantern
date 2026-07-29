use super::common::manager;
use crate::models::server::*;

#[allow(clippy::too_many_arguments)]
/// 创建服务器请求并交给服务层处理
pub(super) fn create_server(
    name: String,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
    startup_mode: String,
) -> Result<ServerInstance, String> {
    let req = CreateServerRequest {
        name,
        core_type,
        mc_version,
        max_memory,
        min_memory,
        port,
        java_path,
        jar_path,
        startup_mode,
        custom_command: None,
    };
    manager().create_server(req)
}

#[allow(clippy::too_many_arguments)]
/// 导入服务端核心并创建服务器
pub(super) fn import_server(
    name: String,
    jar_path: String,
    startup_mode: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    online_mode: bool,
) -> Result<ServerInstance, String> {
    let req = ImportServerRequest {
        name,
        jar_path,
        startup_mode,
        custom_command: None,
        java_path,
        max_memory,
        min_memory,
        port,
        online_mode,
    };
    manager().import_server(req)
}

#[allow(clippy::too_many_arguments)]
/// 接入已经存在的服务器目录
pub(super) fn add_existing_server(
    name: String,
    server_path: String,
    java_path: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    startup_mode: String,
    executable_path: Option<String>,
    custom_command: Option<String>,
) -> Result<ServerInstance, String> {
    let req = AddExistingServerRequest {
        name,
        server_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        executable_path,
        custom_command,
    };
    manager().add_existing_server(req)
}

#[allow(clippy::too_many_arguments)]
/// 导入整合包并创建服务器
pub(super) fn import_modpack(
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
) -> Result<ServerInstance, String> {
    let req = ImportModpackRequest {
        name,
        modpack_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        online_mode,
        custom_command,
        run_path,
        startup_file_path,
        core_type,
        mc_version,
    };
    manager().import_modpack(req)
}

/// 解析服务端核心类型
///
/// 这里会切到阻塞线程里执行实际探测，避免卡住异步运行时
pub(super) async fn parse_server_core_type(
    source_path: String,
) -> Result<ParsedServerCoreInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::services::server::installer::parse_server_core_type(&source_path)
    })
    .await
    .map_err(|e| format!("解析核心类型任务失败: {}", e))?
}
