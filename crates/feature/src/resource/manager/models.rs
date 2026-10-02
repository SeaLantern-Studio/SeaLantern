//! 实例资源的领域模型。
//!
//! 本模块只定义数据结构与构造期校验，不涉及任何文件系统操作。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 当前资源清单的 schema 版本。
pub const MANIFEST_SCHEMA_VERSION: u16 = 1;

/// 实例目录中可管理的资源种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceExtensionKind {
    /// 服务端插件（Bukkit / Spigot / Paper 等）。
    Plugin,
    /// 模组（Fabric / Forge / NeoForge 等）。
    Mod,
    /// 数据包。
    Datapack,
}

impl InstanceExtensionKind {
    /// 用于日志与展示的稳定标识。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plugin => "plugin",
            Self::Mod => "mod",
            Self::Datapack => "datapack",
        }
    }
}

impl std::fmt::Display for InstanceExtensionKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// 单个实例扩展的只读描述（由目录扫描得到的事实）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceExtension {
    /// 资源种类。
    pub kind: InstanceExtensionKind,
    /// 目录中的文件名（不含路径）。
    pub file_name: String,
    /// 文件的完整路径。
    pub path: PathBuf,
    /// 是否处于启用状态。
    pub enabled: bool,
    /// 文件大小（字节）。
    pub size_bytes: u64,
}

impl InstanceExtension {
    /// 构造扩展描述，校验文件名与路径非空。
    pub fn new(
        kind: InstanceExtensionKind,
        file_name: impl Into<String>,
        path: PathBuf,
        enabled: bool,
        size_bytes: u64,
    ) -> Result<Self, InstanceExtensionError> {
        let file_name = file_name.into().trim().to_string();
        if file_name.is_empty() {
            return Err(InstanceExtensionError::EmptyFileName);
        }
        if path.as_os_str().is_empty() {
            return Err(InstanceExtensionError::EmptyPath);
        }
        Ok(Self {
            kind,
            file_name,
            path,
            enabled,
            size_bytes,
        })
    }
}

/// [`InstanceExtension`] 构造时的校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceExtensionError {
    /// 文件名为空。
    EmptyFileName,
    /// 路径为空。
    EmptyPath,
}

impl std::fmt::Display for InstanceExtensionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyFileName => write!(formatter, "extension file name cannot be empty"),
            Self::EmptyPath => write!(formatter, "extension path cannot be empty"),
        }
    }
}

impl std::error::Error for InstanceExtensionError {}

/// 资源的来源平台。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceSource {
    /// 来自 Modrinth。
    Modrinth,
    /// 来自 Spiget（SpigotMC）。
    Spiget,
    /// 用户本地导入。
    Local,
}

/// 资源的来源溯源信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceProvenance {
    /// 来源平台。
    pub source: ResourceSource,
    /// 来源平台上的项目标识。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    /// 来源平台上的版本标识。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    /// 版本号（人类可读）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_number: Option<String>,
    /// 安装时间（Unix 秒）。
    pub installed_at_unix_secs: u64,
}

/// 清单中的一条资源账目。
///
/// `file_name` 与 `enabled` 派生自扫描事实（[`InstanceExtension`]）：扫描是事实
/// 来源，清单只承载元数据增强。同步（`sync`）时会以扫描结果覆盖这两个字段，
/// 因此不应把它们当作独立状态使用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedResource {
    /// 资源文件名（含 `.disabled` 后缀时表示禁用）。
    pub file_name: String,
    /// 资源种类。
    pub kind: InstanceExtensionKind,
    /// 是否启用。
    pub enabled: bool,
    /// 安装时的内容哈希（`sha256:<hex>`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// 文件大小（字节）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// 来源溯源。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ResourceProvenance>,
}

impl ManagedResource {
    /// 由扫描事实构造一条"未知来源"的账目。
    pub fn from_extension(extension: &InstanceExtension) -> Self {
        Self {
            file_name: extension.file_name.clone(),
            kind: extension.kind,
            enabled: extension.enabled,
            hash: None,
            size_bytes: Some(extension.size_bytes),
            provenance: None,
        }
    }
}

/// 从市场来源映射为资源的来源平台。
///
/// Modrinth / Spiget 两平台语义一一对应；本地导入直接以
/// [`ResourceSource::Local`] 表达，不走市场。
impl From<crate::resource::market::MarketSource> for ResourceSource {
    fn from(source: crate::resource::market::MarketSource) -> Self {
        match source {
            crate::resource::market::MarketSource::Modrinth => Self::Modrinth,
            crate::resource::market::MarketSource::Spiget => Self::Spiget,
        }
    }
}

/// 实例资源清单（持久化结构）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// schema 版本。
    pub schema_version: u16,
    /// 资源账目列表。
    #[serde(default)]
    pub resources: Vec<ManagedResource>,
}

impl Default for ResourceManifest {
    fn default() -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            resources: Vec::new(),
        }
    }
}

/// 对账后单个资源的状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceState {
    /// 文件与账目一致。
    Normal,
    /// 文件存在但清单没有账目。
    UnknownSource,
    /// 清单有账目但文件已不存在。
    Missing,
}

/// 单条资源的对账结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciledResource {
    /// 资源种类。
    pub kind: InstanceExtensionKind,
    /// 文件名（文件缺失时为账目中的文件名）。
    pub file_name: String,
    /// 扫描到的事实；文件缺失时为 `None`。
    pub extension: Option<InstanceExtension>,
    /// 清单账目；未知来源时为 `None`。
    pub managed: Option<ManagedResource>,
    /// 对账结论。
    pub state: ResourceState,
}

/// 实例资源的对账报告。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReconcileReport {
    /// 全部资源条目（含缺失项）。
    pub items: Vec<ReconciledResource>,
    /// 清单中重复出现的文件名（同一资源被记录了多次）。
    pub duplicate_entries: Vec<String>,
}

impl ReconcileReport {
    /// 条目总数。
    pub fn total(&self) -> usize {
        self.items.len()
    }

    /// 处于正常状态的条目数。
    pub fn normal_count(&self) -> usize {
        self.count(ResourceState::Normal)
    }

    /// 未知来源的条目数。
    pub fn unknown_count(&self) -> usize {
        self.count(ResourceState::UnknownSource)
    }

    /// 缺失的条目数。
    pub fn missing_count(&self) -> usize {
        self.count(ResourceState::Missing)
    }

    fn count(&self, state: ResourceState) -> usize {
        self.items.iter().filter(|item| item.state == state).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_rejects_blank_file_name() {
        let error = InstanceExtension::new(
            InstanceExtensionKind::Mod,
            "   ",
            PathBuf::from("mods/x.jar"),
            true,
            0,
        )
        .expect_err("blank file name must be rejected");
        assert_eq!(error, InstanceExtensionError::EmptyFileName);
    }

    #[test]
    fn extension_rejects_empty_path() {
        let error =
            InstanceExtension::new(InstanceExtensionKind::Mod, "x.jar", PathBuf::new(), true, 0)
                .expect_err("empty path must be rejected");
        assert_eq!(error, InstanceExtensionError::EmptyPath);
    }

    #[test]
    fn manifest_default_carries_current_schema_version() {
        assert_eq!(ResourceManifest::default().schema_version, MANIFEST_SCHEMA_VERSION);
    }

    #[test]
    fn report_counters_classify_each_state() {
        let report = ReconcileReport {
            items: vec![
                item(ResourceState::Normal),
                item(ResourceState::UnknownSource),
                item(ResourceState::Missing),
                item(ResourceState::Missing),
            ],
            duplicate_entries: Vec::new(),
        };

        assert_eq!(report.total(), 4);
        assert_eq!(report.normal_count(), 1);
        assert_eq!(report.unknown_count(), 1);
        assert_eq!(report.missing_count(), 2);
    }

    #[test]
    fn from_extension_preserves_facts_without_provenance() {
        let extension = InstanceExtension::new(
            InstanceExtensionKind::Plugin,
            "essentials.jar",
            PathBuf::from("plugins/essentials.jar"),
            false,
            2048,
        )
        .expect("fixture extension should be valid");

        let managed = ManagedResource::from_extension(&extension);
        assert_eq!(managed.file_name, "essentials.jar");
        assert_eq!(managed.kind, InstanceExtensionKind::Plugin);
        assert!(!managed.enabled);
        assert_eq!(managed.size_bytes, Some(2048));
        assert!(managed.hash.is_none());
        assert!(managed.provenance.is_none());
    }

    fn item(state: ResourceState) -> ReconciledResource {
        ReconciledResource {
            kind: InstanceExtensionKind::Mod,
            file_name: "x.jar".into(),
            extension: None,
            managed: None,
            state,
        }
    }
}
