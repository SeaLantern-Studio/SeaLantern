//! 实例发现视图的契约 DTO。
//!
//! 与 `feature::config::instance` 的内部类型对应，但字段全部是可序列化的
//! 传输值（不含 core 类型），供端口方法 `InstanceService::discovery` 携带
//! 「待处理实例 + 问题清单 + 孤儿信任记录」给宿主与前端。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 一个身份有效、但尚未登记信任的实例条目（「未标记」态）。
///
/// 仅携带展示所需的最小概要；完整配置通过实例 id 另行拉取。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PendingInstance {
    /// 实例目录（词法规范化后）。
    pub dir: PathBuf,
    /// `sl.json` 中登记的实例 id。
    pub instance_id: String,
    /// `sl.json` 中登记的显示名。
    pub name: String,
}

/// 发现问题的机器可读类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryProblemKind {
    /// 目录或 `sl.json` 无法读取。
    UnreadableDir,
    /// 目录里没有 `sl.json`。
    MissingDocument,
    /// `sl.json` 存在但不可用（损坏 / 缺版本号 / 校验失败）。
    InvalidDocument,
    /// `sl.json` 由更新版本的程序写入。
    UnsupportedSchemaVersion,
    /// 同一实例 id 出现在多个目录（`dirs` 携带全部成员）。
    DuplicateId,
    /// 同一目录经不同写法被扫描多次（`alias_of` 指向首次出现）。
    DuplicateDirAlias,
}

/// 一条发现问题的传输表示。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DiscoveryProblemEntry {
    /// 问题类别。
    pub kind: DiscoveryProblemKind,
    /// 主目录（`DuplicateId` 时为 None，成员在 `dirs`）。
    #[serde(default)]
    pub dir: Option<PathBuf>,
    /// 涉及的目录集合（仅 `DuplicateId` 使用）。
    #[serde(default)]
    pub dirs: Vec<PathBuf>,
    /// 重复 id 的实例标识（仅 `DuplicateId`）。
    #[serde(default)]
    pub id: Option<String>,
    /// 文档写入的 schema 版本（仅 `UnsupportedSchemaVersion`）。
    #[serde(default)]
    pub schema_version: Option<u32>,
    /// 目录别名指向的首次出现目录（仅 `DuplicateDirAlias`）。
    #[serde(default)]
    pub alias_of: Option<PathBuf>,
    /// 诊断消息（不含敏感数据）。
    #[serde(default)]
    pub message: Option<String>,
}

/// 发现问题 + 是否整体被忽略（涉及目录全在忽略名单时可默认折叠）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClassifiedProblemEntry {
    pub problem: DiscoveryProblemEntry,
    pub ignored: bool,
}
