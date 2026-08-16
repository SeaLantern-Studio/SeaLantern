use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::instance::{
    plan_import, Instance, InstanceError, InstanceId, InstanceImportError, InstanceImportPlan,
    InstanceImportRequest, InstanceSpec, LocalLaunch, StartupMode,
};
use crate::provisioning::{
    apply_server_inspection_with_options, inspect_server_artifact, InspectionOptions,
    LaunchProfilePolicy, ServerInspectionError, ServerInspectionProjectionOptions,
};

/// 为已有服务器目录构建导入计划。
///
/// 该函数只处理路径与启动目标合同；目录扫描和文件操作由上层实现。
pub fn plan_existing_instance(
    request: InstanceImportRequest,
) -> Result<InstanceImportPlan, ExistingInstanceError> {
    plan_import(request).map_err(ExistingInstanceError::Import)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExistingInstanceError {
    Import(InstanceImportError),
}

impl std::fmt::Display for ExistingInstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import(error) => write!(formatter, "invalid existing instance import: {error}"),
        }
    }
}

impl std::error::Error for ExistingInstanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Import(error) => Some(error),
        }
    }
}

/// 导入已有服务器目录的请求。
///
/// 所有字段均可选覆盖；缺省时由检查结果推导（目录名、默认端口、默认内存等）。
/// `source_directory` 为原始服务器目录，导入后实例直接引用，不复制文件（FR-5）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ImportExistingServerRequest {
    /// 待导入的服务器目录（原始目录；导入后实例直接引用，不复制）。
    pub source_directory: PathBuf,
    /// 可选的实例名称；缺省时回退为目录名。
    #[serde(default)]
    pub name: Option<String>,
    /// 监听端口；缺省时回退默认 25565。
    #[serde(default)]
    pub port: Option<u16>,
    /// 最大内存（MiB）；缺省时回退默认 4096。
    #[serde(default)]
    pub max_memory_mib: Option<u32>,
    /// 最小内存（MiB）；缺省时回退默认 1024。
    #[serde(default)]
    pub min_memory_mib: Option<u32>,
    /// 可选的 Java 可执行文件覆盖。
    #[serde(default)]
    pub java_executable: Option<PathBuf>,
    /// 可选的 JVM 参数覆盖（覆盖识别所得，或作为无识别结果时的兜底）。
    #[serde(default)]
    pub jvm_arguments: Option<Vec<String>>,
    /// 可选的启动配置覆盖：指定所要采纳的检查启动候选 `profile_id`。
    #[serde(default)]
    pub selected_launch_profile_id: Option<String>,
}

/// 构建导入规格失败的原因。
#[derive(Debug)]
pub enum ImportExistingServerError {
    /// 目录检查失败（目录不存在、不可读、不含可识别服务器等）。
    Inspection(ServerInspectionError),
    /// 未找到任何可采纳的启动目标。
    NoLaunchCandidate,
    /// 生成的实例规格不合法（名称/端口/内存等）。
    InvalidInstance(InstanceError),
}

impl std::fmt::Display for ImportExistingServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inspection(error) => write!(f, "server inspection failed: {error}"),
            Self::NoLaunchCandidate => {
                write!(f, "no launchable server artifact was found in the directory")
            }
            Self::InvalidInstance(error) => write!(f, "invalid instance spec: {error}"),
        }
    }
}

impl std::error::Error for ImportExistingServerError {}

/// 检查已有服务器目录并构建可直接注册的导入 DTO。
///
/// 返回 [`InstanceImportRequest`]（打包源目录与实例规格），由调用方直接用于创建实例
/// 或进一步规划，而非以裸 `InstanceSpec` 层层传参。规格的 `directory` 指向原始目录
/// （FR-5：直接引用不复制）；启动目标由检查结果按
/// [`LaunchProfilePolicy::AdoptBestCompatible`] 采纳最优候选。调用方可通过请求字段
/// 覆盖名称、端口、内存与 Java 配置，或显式指定要采纳的启动候选。
pub fn build_import_spec(
    request: &ImportExistingServerRequest,
) -> Result<InstanceImportRequest, ImportExistingServerError> {
    let source = &request.source_directory;
    let report = inspect_server_artifact(source, &InspectionOptions::default())
        .map_err(ImportExistingServerError::Inspection)?;

    let folder_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("imported-server")
        .to_string();

    let mut spec = InstanceSpec {
        id: make_instance_id(&folder_name),
        name: request
            .name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| folder_name.clone()),
        aliases: Vec::new(),
        core_type: String::new(),
        core_version: String::new(),
        game_version: String::new(),
        directory: source.clone(),
        port: request.port.unwrap_or(25565),
        max_memory_mib: request.max_memory_mib.unwrap_or(4096),
        min_memory_mib: request.min_memory_mib.unwrap_or(1024),
        created_at_unix_secs: current_unix_secs(),
        last_started_at_unix_secs: None,
        server_metadata: None,
        launch: LocalLaunch {
            startup_mode: StartupMode::Jar,
            startup_target: None,
            custom_command: None,
            custom_executable: None,
            custom_arguments: Vec::new(),
            java_executable: request.java_executable.clone(),
            jvm_arguments: request.jvm_arguments.clone().unwrap_or_default(),
        },
    };

    let projection = apply_server_inspection_with_options(
        &mut spec,
        Ok(&report),
        &ServerInspectionProjectionOptions {
            launch_profile_policy: LaunchProfilePolicy::AdoptBestCompatible,
            inspected_at_unix_secs: Some(current_unix_secs()),
        },
    );

    // 调用方显式指定启动候选时，覆盖采纳结果。
    if let Some(selected) = &request.selected_launch_profile_id {
        if let Some(candidate) = projection
            .launch_candidates
            .iter()
            .find(|candidate| &candidate.profile_id == selected)
        {
            spec.launch = candidate.launch.clone();
            if spec.launch.jvm_arguments.is_empty() {
                spec.launch.jvm_arguments = request.jvm_arguments.clone().unwrap_or_default();
            }
            if spec.launch.java_executable.is_none() {
                spec.launch.java_executable = request.java_executable.clone();
            }
        }
    }

    // 调用方显式提供的 JVM 参数与 Java 可执行文件优先于识别结果，避免被静默丢弃。
    // 字段文档约定二者为“覆盖”，因此一旦提供即以用户值覆盖识别所得。
    if let Some(jvm_arguments) = &request.jvm_arguments {
        spec.launch.jvm_arguments = jvm_arguments.clone();
    }
    if let Some(java_executable) = &request.java_executable {
        spec.launch.java_executable = Some(java_executable.clone());
    }

    if spec.launch.startup_target.is_none()
        && spec.launch.custom_command.is_none()
        && spec.launch.custom_executable.is_none()
    {
        return Err(ImportExistingServerError::NoLaunchCandidate);
    }

    // 复用 Instance 字段级校验，提前暴露非法覆盖（端口/内存/名称等）。
    Instance::new(spec.clone())
        .map(|_| ())
        .map_err(ImportExistingServerError::InvalidInstance)?;

    // 以 DTO 形式贯穿：打包源目录与实例规格，供调用方直接创建或规划。
    Ok(InstanceImportRequest {
        source_directory: request.source_directory.clone(),
        instance: spec,
    })
}

/// 由目录名生成稳定、合法的实例 ID（小写，仅保留字母数字与 `-`/`_`，追加纳秒时间戳防重）。
fn make_instance_id(folder_name: &str) -> InstanceId {
    let base: String = folder_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase();
    let base = if base.is_empty() {
        "imported".to_string()
    } else {
        base
    };
    // 使用纳秒级时间戳作为后缀，避免同一秒内导入产生相同实例 ID。
    let stamp = current_unix_nanos();
    InstanceId::new(format!("{base}-{stamp}")).expect("generated instance id must be non-empty")
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn current_unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}
