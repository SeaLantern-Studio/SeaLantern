use super::common::manager;
use crate::models::server::*;
use sea_lantern_server_installer_core::parse_server_core_type as parse_shared_server_core_type;

#[allow(clippy::too_many_arguments)]
/// 创建服务器请求并交给服务层处理
pub(super) fn create_server(
    name: String,
    aliases: Option<Vec<String>>,
    core_type: String,
    mc_version: String,
    max_memory: u32,
    min_memory: u32,
    port: u16,
    java_path: String,
    jar_path: String,
    server_path: Option<String>,
    startup_mode: String,
    custom_command: Option<String>,
    jvm_args: Vec<String>,
    cpu_policy: CpuPolicyConfig,
    jvm_preset: JvmPresetConfig,
) -> Result<ServerInstance, String> {
    let req = CreateServerRequest {
        name,
        aliases: aliases.unwrap_or_default(),
        core_type,
        mc_version,
        max_memory,
        min_memory,
        port,
        java_path,
        jar_path,
        server_path,
        startup_mode,
        custom_command,
        jvm_args,
        cpu_policy,
        jvm_preset,
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
    jvm_args: Vec<String>,
    cpu_policy: CpuPolicyConfig,
    jvm_preset: JvmPresetConfig,
) -> Result<ServerInstance, String> {
    let req = ImportServerRequest {
        name,
        aliases: Vec::new(),
        jar_path,
        startup_mode,
        custom_command: None,
        java_path,
        max_memory,
        min_memory,
        port,
        online_mode,
        jvm_args,
        cpu_policy,
        jvm_preset,
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
    core_type: Option<String>,
    mc_version: Option<String>,
    jvm_args: Vec<String>,
    cpu_policy: CpuPolicyConfig,
    jvm_preset: JvmPresetConfig,
) -> Result<ServerInstance, String> {
    let req = AddExistingServerRequest {
        name,
        aliases: Vec::new(),
        server_path,
        java_path,
        max_memory,
        min_memory,
        port,
        startup_mode,
        executable_path,
        custom_command,
        core_type,
        mc_version,
        jvm_args,
        cpu_policy,
        jvm_preset,
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
    jvm_args: Vec<String>,
    cpu_policy: CpuPolicyConfig,
    jvm_preset: JvmPresetConfig,
) -> Result<ServerInstance, String> {
    let req = ImportModpackRequest {
        name,
        aliases: Vec::new(),
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
        jvm_args,
        cpu_policy,
        jvm_preset,
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
        parse_shared_server_core_type(&source_path)
    })
    .await
    .map_err(|e| format!("解析核心类型任务失败: {}", e))?
}
