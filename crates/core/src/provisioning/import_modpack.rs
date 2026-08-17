//! 整合包导入核心逻辑。
//!
//! 支持三种来源：
//! - zip/tar.gz/tgz 压缩包：解压到 run_path
//! - jar 单文件：复制到 run_path
//! - 文件夹：直接引用原路径

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::instance::{InstanceError, InstanceId, InstanceSpec, LocalLaunch, StartupMode};

/// 来源类型枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    /// zip/tar.gz/tgz 压缩包。
    Archive,
    /// 单个 jar 文件。
    JarFile,
    /// 已存在的文件夹。
    Folder,
}

/// 根据路径扩展名推断来源类型。
pub fn infer_source_type(path: &Path) -> SourceType {
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
#[serde(rename_all = "camelCase")]
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

/// 整合包导入错误。
#[derive(Debug, Clone)]
pub enum ImportModpackError {
    /// 无效的启动模式。
    InvalidStartupMode(String),
    /// 无效的实例 ID。
    InvalidInstanceId(InstanceError),
    /// 解压失败。
    ExtractFailed(String),
    /// 创建目录失败。
    CreateDirectoryFailed(String),
    /// 复制文件失败。
    CopyFailed(String),
}

impl std::fmt::Display for ImportModpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStartupMode(msg) => write!(f, "invalid startup mode: {msg}"),
            Self::InvalidInstanceId(err) => write!(f, "invalid instance ID: {err}"),
            Self::ExtractFailed(msg) => write!(f, "failed to extract archive: {msg}"),
            Self::CreateDirectoryFailed(msg) => write!(f, "failed to create directory: {msg}"),
            Self::CopyFailed(msg) => write!(f, "failed to copy file: {msg}"),
        }
    }
}

impl std::error::Error for ImportModpackError {}

/// 导入结果，包含有效的实例目录和启动目标。
#[derive(Debug, Clone)]
pub struct ImportModpackResult {
    /// 有效的实例目录。
    pub directory: PathBuf,
    /// 启动目标路径。
    pub startup_target: Option<PathBuf>,
    /// 构建好的实例规格。
    pub spec: InstanceSpec,
}

/// 构建实例规格。
///
/// 根据 import 请求和有效的目录路径构建 `InstanceSpec`。
pub fn build_instance_spec(
    request: &ImportModpackRequest,
    directory: &Path,
    startup_target: Option<PathBuf>,
) -> Result<InstanceSpec, ImportModpackError> {
    let id = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let startup_mode = StartupMode::parse(&request.startup_mode)
        .map_err(|error| ImportModpackError::InvalidStartupMode(error.to_string()))?;

    let id = InstanceId::new(id).map_err(ImportModpackError::InvalidInstanceId)?;

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

/// 规划整合包导入。
///
/// 根据来源类型计算目标目录和启动目标，返回构建好的 `InstanceSpec`。
/// 此函数不执行任何文件系统操作，仅做规划。
pub fn plan_import_modpack(
    request: &ImportModpackRequest,
) -> Result<ImportModpackResult, ImportModpackError> {
    let source_type = infer_source_type(&request.modpack_path);

    let (directory, startup_target) = match source_type {
        SourceType::Archive => {
            // 解压到 run_path（调用者负责实际解压）
            let target = request
                .startup_file_path
                .as_ref()
                .map(|p| request.run_path.join(p));
            (request.run_path.clone(), target)
        }
        SourceType::JarFile => {
            // 复制 jar 到 run_path（调用者负责实际复制）
            let jar_name = request
                .modpack_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("server.jar");
            let dest_path = request.run_path.join(jar_name);
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

    let spec = build_instance_spec(request, &directory, startup_target.clone())?;

    Ok(ImportModpackResult { directory, startup_target, spec })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_source_type_detects_zip() {
        assert_eq!(infer_source_type(Path::new("/path/to/modpack.zip")), SourceType::Archive);
    }

    #[test]
    fn infer_source_type_detects_jar() {
        assert_eq!(infer_source_type(Path::new("/path/to/server.jar")), SourceType::JarFile);
    }

    #[test]
    fn infer_source_type_detects_tar_gz() {
        assert_eq!(infer_source_type(Path::new("/path/to/modpack.tar.gz")), SourceType::Archive);
    }

    #[test]
    fn infer_source_type_detects_tgz() {
        assert_eq!(infer_source_type(Path::new("/path/to/modpack.tgz")), SourceType::Archive);
    }

    #[test]
    fn infer_source_type_detects_folder() {
        assert_eq!(infer_source_type(Path::new("/path/to/server-folder")), SourceType::Folder);
    }
}
