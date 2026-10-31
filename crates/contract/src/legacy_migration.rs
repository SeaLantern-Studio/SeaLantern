//! 旧版实例配置迁移契约模型。
//!
//! 1.3.x 之前的版本把所有服务器实例集中存放在主配置目录的
//! `sea_lantern_servers.json`；更新后实例权威配置改为各服务器目录内的
//! `sl.json`，旧文件不再被读取，导致升级后服务器列表「消失」。
//!
//! 本模块只描述检测与迁移结果，供宿主（Tauri / HTTP）转交给前端弹窗；
//! 迁移本身不自动执行，必须由用户在界面上确认。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// 旧版实例配置的迁移检测结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LegacyMigrationStatus {
    /// 当前是否应当向用户提示迁移。
    ///
    /// 仅当存在旧配置、没有已受信任的实例、存在可迁移条目且用户尚未
    /// 关闭提示时为 `true`。
    pub available: bool,
    /// 用户是否已明确选择「不再提示」。
    pub dismissed: bool,
    /// 旧配置中的服务器条目；不可迁移项也会列出，供界面展示明细。
    pub entries: Vec<LegacyMigrationEntry>,
}

impl LegacyMigrationStatus {
    /// 没有任何需要提示的内容时的空状态。
    pub fn empty(dismissed: bool) -> Self {
        Self {
            available: false,
            dismissed,
            entries: Vec::new(),
        }
    }
}

/// 旧配置中的一个服务器条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LegacyMigrationEntry {
    /// 显示名；旧配置名称为空时回落到目录名。
    pub name: String,
    /// 服务器目录（旧配置 `path` 字段的原始值）。
    pub directory: PathBuf,
    /// 该条目的迁移可行性。
    pub state: LegacyMigrationEntryState,
}

/// 旧配置条目的迁移可行性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyMigrationEntryState {
    /// 目录存在且没有 `sl.json`，可以迁移。
    Migratable,
    /// 目录内已存在 `sl.json`，视为已迁移过。
    AlreadyMigrated,
    /// 旧配置记录的目录已不存在（或不是目录）。
    MissingDirectory,
    /// 旧配置条目无法转换为有效的新实例文档。
    Invalid,
}

/// 迁移执行报告。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LegacyMigrationReport {
    /// 成功迁移的实例。
    pub migrated: Vec<MigratedLegacyInstance>,
    /// 被跳过的条目及原因。
    pub skipped: Vec<SkippedLegacyInstance>,
    /// 旧配置文件是否已归档（重命名保留，避免重复提示）。
    pub legacy_file_archived: bool,
}

impl LegacyMigrationReport {
    /// 迁移是否实际写入过实例。
    pub fn has_migrated(&self) -> bool {
        !self.migrated.is_empty()
    }
}

/// 成功迁移的实例概要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MigratedLegacyInstance {
    /// 迁移后的实例 id（与旧配置不同表示已自动重新分配）。
    pub instance_id: String,
    /// 实例显示名。
    pub name: String,
    /// 实例目录。
    pub directory: PathBuf,
}

/// 被跳过的旧配置条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkippedLegacyInstance {
    /// 旧配置中的显示名。
    pub name: String,
    /// 旧配置中的服务器目录。
    pub directory: PathBuf,
    /// 跳过原因。
    pub reason: LegacyMigrationSkipReason,
}

/// 旧配置条目被跳过的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyMigrationSkipReason {
    /// 目录内已有 `sl.json`，无需重复迁移。
    AlreadyMigrated,
    /// 旧配置记录的目录已不存在。
    MissingDirectory,
    /// 条目内容无法转换为有效的实例文档。
    Invalid,
    /// 创建实例时发生冲突或持久化失败。
    CreateFailed,
}
