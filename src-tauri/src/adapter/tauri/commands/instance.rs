//! 实例管理 Tauri 命令。
//!
//! 前端通过 `invoke` 调用这些命令，命令内部经应用装配层拿到
//! [`CoreInstanceService`](sealantern_application::service::CoreInstanceService)
//! 执行查询与 CRUD。
//!
//! 错误统一为接口契约错误 [`InstanceServiceError`]，可序列化回前端，
//! 不携带底层敏感细节。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sealantern_application::error::InstanceError;
use sealantern_application::service::CoreInstanceService;
use sealantern_application::services::AppServices;
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec, LocalLaunch, StartupMode};
use sealantern_core::provisioning::{
    ImportExistingServerError as CoreImportError, ImportExistingServerRequest,
    SourceDirectoryError, build_import_spec, plan_existing_instance, source_directories_equal,
    validate_source_directory,
};
use sealantern_infra::archive::extract_zip;
use sealantern_interface::{InstanceService, InstanceServiceError};

/// 获取全局实例管理服务句柄（惰性初始化容器）。
///
/// 应用层主错误 [`InstanceError`] 收敛为契约错误 [`InstanceServiceError`]。
async fn instance_service() -> Result<Arc<CoreInstanceService>, InstanceServiceError> {
    let services = AppServices::get().await?;
    Ok(services.instance().clone())
}

/// 解析 Tauri 命令传入的实例 ID 字符串。
///
/// 统一映射解析错误为 [`InstanceServiceError::InvalidInput`]，避免各命令重复
/// 内联解析与错误映射；后续若调整非法输入的错误变体，只需修改此处。
fn parse_id_for_tauri(id: String) -> Result<InstanceId, InstanceServiceError> {
    InstanceId::new(id)
        .map_err(InstanceError::from)
        .map_err(InstanceServiceError::from)
}

/// 列出全部实例。
#[tauri::command(rename_all = "snake_case")]
pub async fn list_instances() -> Result<Vec<Instance>, InstanceServiceError> {
    let service = instance_service().await?;
    service.list().await
}

/// 按 ID 查找实例，不存在时返回 `None`。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_instance(id: String) -> Result<Option<Instance>, InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.find(&id).await
}

/// 创建新实例并持久化。
#[tauri::command(rename_all = "snake_case")]
pub async fn create_instance(spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
    let service = instance_service().await?;
    service.create(spec).await
}

/// 删除实例；实例不存在时返回 [`InstanceServiceError::InstanceNotFound`]。
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_instance(id: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.delete(&id).await
}

/// 重命名实例。
#[tauri::command(rename_all = "snake_case")]
pub async fn rename_instance(id: String, name: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.rename(&id, &name).await
}

/// 更新实例目录路径。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_instance_path(id: String, path: String) -> Result<(), InstanceServiceError> {
    let service = instance_service().await?;
    let id = parse_id_for_tauri(id)?;
    service.update_path(&id, &path).await
}

/// 导入已有服务器目录失败时返回给前端的错误。
///
/// 同时实现 `Serialize` 与 `std::error::Error`，便于 Tauri 序列化回调用方并携带
/// 稳定的机器可读错误码（如 `source_unavailable` / `no_launch_candidate`）。
#[derive(Debug, serde::Serialize)]
pub struct ImportExistingServerError {
    /// 稳定错误码（机器可读）。
    pub code: String,
    /// 人类可读消息。
    pub message: String,
}

impl std::fmt::Display for ImportExistingServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ImportExistingServerError {}

/// 导入已有服务器目录：校验 → 去重 → 检查 → 构建规格 → 注册。
///
/// 导入的实例直接引用原始目录（FR-5：不复制文件），启动目标由检查结果采纳最优候选。
#[tauri::command(rename_all = "snake_case")]
pub async fn import_existing_server(
    request: ImportExistingServerRequest,
) -> Result<Instance, ImportExistingServerError> {
    validate_source_directory(&request.source_directory).map_err(|error| match error {
        SourceDirectoryError::Unavailable(_) => {
            import_error("source_unavailable", error.to_string())
        }
        SourceDirectoryError::NotDirectory(_) => {
            import_error("source_not_directory", error.to_string())
        }
    })?;

    let service = instance_service()
        .await
        .map_err(|error| import_error("service_unavailable", error.to_string()))?;
    let instances = service
        .list()
        .await
        .map_err(|error| import_error("list_failed", error.to_string()))?;
    if instances.iter().any(|instance| {
        source_directories_equal(instance.directory.as_path(), request.source_directory.as_path())
    }) {
        return Err(import_error(
            "source_already_imported",
            "the selected directory is already imported as a server instance",
        ));
    }

    // `build_import_spec` 内部执行同步目录扫描（inspect_server_artifact），
    // 放到 blocking 线程，避免阻塞 async 运行时。
    let import_request = tokio::task::spawn_blocking({
        let request = request.clone();
        move || build_import_spec(&request)
    })
    .await
    .map_err(|join_error| {
        import_error("import_panic", format!("import spec build failed: {join_error}"))
    })?;
    let import_request = import_request.map_err(|error| match error {
        CoreImportError::Inspection(_) => import_error("inspection_failed", error.to_string()),
        CoreImportError::NoLaunchCandidate => {
            import_error("no_launch_candidate", error.to_string())
        }
        CoreImportError::InvalidInstance(_) => import_error("invalid_instance", error.to_string()),
    })?;

    // 流经既有后端原语 plan_existing_instance 完成启动目标校验与归一化，
    // 复用已有的导入规划能力，而非直接以裸规格创建实例。
    let plan = plan_existing_instance(import_request)
        .map_err(|error| import_error("invalid_instance", error.to_string()))?;
    service
        .create(plan.instance.spec())
        .await
        .map_err(|error| import_error("create_failed", error.to_string()))
}

/// 构造一个带稳定错误码的导入错误。
fn import_error(code: &'static str, message: impl Into<String>) -> ImportExistingServerError {
    ImportExistingServerError {
        code: code.to_string(),
        message: message.into(),
    }
}

// ============================================================================
// import_modpack 命令实现
// ============================================================================

/// 来源类型枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceType {
    /// zip/tar.gz/tgz 压缩包。
    Archive,
    /// 单个 jar 文件。
    JarFile,
    /// 已存在的文件夹。
    Folder,
}

/// 根据路径扩展名推断来源类型。
fn infer_source_type(path: &std::path::Path) -> SourceType {
    let path_str = path.to_string_lossy().to_lowercase();

    // .tar.gz 和 .tgz 必须先判断
    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        return SourceType::Archive;
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    match ext.as_deref() {
        Some("zip") => SourceType::Archive,
        Some("jar") => SourceType::JarFile,
        _ => SourceType::Folder,
    }
}

/// 整合包导入请求。
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImportModpackRequest {
    /// 实例名称。
    pub name: String,
    /// 整合包来源（zip 路径或文件夹路径）。
    pub modpack_path: PathBuf,
    /// Java 可执行文件路径。
    pub java_path: PathBuf,
    /// 最大内存（MiB）。
    pub max_memory: u32,
    /// 最小内存（MiB）。
    pub min_memory: u32,
    /// 监听端口。
    pub port: u16,
    /// 启动模式。
    pub startup_mode: String,
    /// 启动目标文件路径（相对于 run_path）。
    pub startup_file_path: Option<PathBuf>,
    /// 核心类型（paper、forge 等）。
    pub core_type: Option<String>,
    /// Minecraft 版本。
    pub mc_version: Option<String>,
    /// 自定义启动命令。
    pub custom_command: Option<String>,
    /// 目标运行目录。
    pub run_path: PathBuf,
}

/// 整合包导入失败时返回给前端的错误。
#[derive(Debug, serde::Serialize)]
pub struct ImportModpackError {
    /// 稳定错误码（机器可读）。
    pub code: String,
    /// 人类可读消息。
    pub message: String,
}

impl std::fmt::Display for ImportModpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ImportModpackError {}

/// 构造一个带稳定错误码的整合包导入错误。
fn modpack_error(code: &'static str, message: impl Into<String>) -> ImportModpackError {
    ImportModpackError {
        code: code.to_string(),
        message: message.into(),
    }
}

/// 生成实例 ID（时间戳毫秒）。
fn generate_instance_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{}", ts)
}

/// 构建 InstanceSpec。
fn build_instance_spec(
    request: &ImportModpackRequest,
    directory: &std::path::Path,
    startup_target: Option<PathBuf>,
) -> Result<InstanceSpec, ImportModpackError> {
    let id = generate_instance_id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let startup_mode = StartupMode::parse(&request.startup_mode).map_err(|error| {
        modpack_error("invalid_startup_mode", format!("invalid startup mode: {error}"))
    })?;

    let id = InstanceId::new(id).map_err(|error| {
        modpack_error("invalid_instance_id", format!("failed to create instance ID: {error}"))
    })?;

    Ok(InstanceSpec {
        id,
        name: request.name.clone(),
        aliases: Vec::new(),
        core_type: request.core_type.clone().unwrap_or_default(),
        core_version: String::new(),
        game_version: request.mc_version.clone().unwrap_or_default(),
        directory: directory.to_path_buf(),
        port: request.port,
        max_memory_mib: request.max_memory,
        min_memory_mib: request.min_memory,
        created_at_unix_secs: now,
        last_started_at_unix_secs: None,
        server_metadata: None,
        launch: LocalLaunch {
            startup_mode,
            startup_target,
            custom_command: request.custom_command.clone(),
            custom_executable: None,
            custom_arguments: Vec::new(),
            java_executable: Some(request.java_path.clone()),
            jvm_arguments: Vec::new(),
        },
    })
}

/// 导入整合包或服务器文件夹。
///
/// 支持三种来源：
/// - zip/tar.gz/tgz 压缩包：解压到 run_path
/// - jar 单文件：复制到 run_path
/// - 文件夹：直接引用原路径
#[tauri::command(rename_all = "snake_case")]
pub async fn import_modpack(request: ImportModpackRequest) -> Result<Instance, ImportModpackError> {
    // 1. 判断来源类型
    let source_type = infer_source_type(&request.modpack_path);

    // 2. 根据类型处理文件
    let (effective_directory, startup_target) = match source_type {
        SourceType::Archive => {
            // 解压 zip/tar.gz 到 run_path
            extract_zip(&request.modpack_path, &request.run_path).map_err(|error| {
                modpack_error("extract_failed", format!("failed to extract archive: {error}"))
            })?;
            let target = request
                .startup_file_path
                .as_ref()
                .map(|p| request.run_path.join(p));
            (request.run_path.clone(), target)
        }
        SourceType::JarFile => {
            // 创建 run_path 目录并复制 jar
            std::fs::create_dir_all(&request.run_path).map_err(|error| {
                modpack_error(
                    "create_directory_failed",
                    format!("failed to create run directory: {error}"),
                )
            })?;
            let jar_name = request
                .modpack_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("server.jar");
            let dest_path = request.run_path.join(jar_name);
            std::fs::copy(&request.modpack_path, &dest_path).map_err(|error| {
                modpack_error("copy_jar_failed", format!("failed to copy jar file: {error}"))
            })?;
            (request.run_path.clone(), Some(dest_path))
        }
        SourceType::Folder => {
            // 直接引用原目录
            let target = request
                .startup_file_path
                .as_ref()
                .map(|p| request.modpack_path.join(p));
            (request.modpack_path.clone(), target)
        }
    };

    // 3. 构建 InstanceSpec
    let spec = build_instance_spec(&request, &effective_directory, startup_target)?;

    // 4. 创建实例
    let service = instance_service()
        .await
        .map_err(|error| modpack_error("service_unavailable", error.to_string()))?;
    service
        .create(spec)
        .await
        .map_err(|error| modpack_error("create_failed", error.to_string()))
}
