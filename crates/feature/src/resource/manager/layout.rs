//! 服务器类型到资源目录的映射（纯逻辑，零 IO）。
//!
//! 映射依据是服务端检查（`core::provisioning::server_inspection`）识别出的
//! 生态标记（`ecosystems`），而不是写死服务器名称。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::models::InstanceExtensionKind;
use super::path;

/// 插件类服务器生态标记。
const PLUGIN_ECOSYSTEMS: &[&str] = &[
    "bukkit",
    "spigot",
    "paper",
    "purpur",
    "folia",
    "sponge",
    "velocity",
    "bungeecord",
];

/// 模组类服务器生态标记。
const MOD_ECOSYSTEMS: &[&str] =
    &["fabric", "forge", "neoforge", "quilt", "liteloader", "rift", "connector"];

/// 一个实例需要管理的资源目录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceTarget {
    /// 资源种类。
    pub kind: InstanceExtensionKind,
    /// 相对于实例根目录的路径。
    pub relative: PathBuf,
}

impl ResourceTarget {
    /// 构造一个资源目录目标。
    pub fn new(kind: InstanceExtensionKind, relative: impl Into<PathBuf>) -> Self {
        Self { kind, relative: relative.into() }
    }

    /// 目录名（统一使用 `/` 分隔，便于展示与日志）。
    pub fn dir_name(&self) -> String {
        self.relative.to_string_lossy().replace('\\', "/")
    }
}

/// 实例可管理的资源目录清单（供界面决定展示哪些资源分类）。
///
/// 前端在资源列表为空时无法从条目反推"该实例支持什么"，因此需要单独的
/// 目标查询：既有目录清单，也有默认聚焦的主要种类。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceTargets {
    /// 需要管理的资源目录，顺序固定为「模组在前」。
    pub targets: Vec<ResourceTarget>,
    /// 主要资源种类，供界面默认聚焦；实例无资源目录时为 `None`。
    ///
    /// 由 [`targets`](Self::targets) 首项推导：[`resource_targets`] 保证模组
    /// 优先，无需另行传入生态标记。
    pub primary_kind: Option<InstanceExtensionKind>,
}

impl ResourceTargets {
    /// 由资源目录列表构造，`primary_kind` 取首项。
    pub fn new(targets: Vec<ResourceTarget>) -> Self {
        let primary_kind = targets.first().map(|target| target.kind);
        Self { targets, primary_kind }
    }
}

/// 资源目录映射错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LayoutError {
    /// 世界名非法（为空、含分隔符，或为 `.` / `..` 等）。
    #[error("非法的世界名: {0}")]
    InvalidWorldName(String),
}

/// 根据服务器生态标记推导需要管理的资源目录。
///
/// 插件类生态得到 `plugins/`，模组类生态得到 `mods/`；混合服务端
/// （如 Mohist、Arclight）同时具备两类标记，此时返回两者。
///
/// 返回顺序固定为「模组目录在前」：混合服务端以模组加载器为核心运行时、
/// 插件兼容为附加层，该优先级与 [`primary_kind`] 保持一致。
pub fn resource_targets(ecosystems: &[String]) -> Vec<ResourceTarget> {
    let mut targets = Vec::new();

    if has_mod_ecosystem(ecosystems) {
        targets.push(ResourceTarget::new(InstanceExtensionKind::Mod, "mods"));
    }
    if has_plugin_ecosystem(ecosystems) {
        targets.push(ResourceTarget::new(InstanceExtensionKind::Plugin, "plugins"));
    }

    targets
}

/// 数据包目录；世界名通常取自 `server.properties` 的 `level-name`。
///
/// # 当前未接线（预留）
///
/// 首期只管理 `mods/` 与 `plugins/`，数据包不纳入：世界名需要解析
/// `server.properties` 的 `level-name`，且应用层
/// `extension_kind` 目前会拒绝 `Datapack`。本函数为后续接入预留，
/// 生产路径不会调用。
///
/// 世界名来自外部配置（导入的服务器目录可被任意编辑），必须先校验为
/// 单一普通路径组件，避免 `..` 或绝对路径逃逸实例目录。非法世界名
/// 显式返回 [`LayoutError::InvalidWorldName`]，不做静默回退。
pub fn datapack_target(world_name: &str) -> Result<ResourceTarget, LayoutError> {
    if !path::is_single_normal_component(world_name) {
        return Err(LayoutError::InvalidWorldName(world_name.to_string()));
    }

    Ok(ResourceTarget::new(
        InstanceExtensionKind::Datapack,
        PathBuf::from(world_name).join("datapacks"),
    ))
}

fn has_plugin_ecosystem(ecosystems: &[String]) -> bool {
    contains_any(ecosystems, PLUGIN_ECOSYSTEMS)
}

fn has_mod_ecosystem(ecosystems: &[String]) -> bool {
    contains_any(ecosystems, MOD_ECOSYSTEMS)
}

fn contains_any(ecosystems: &[String], candidates: &[&str]) -> bool {
    ecosystems.iter().any(|ecosystem| {
        let normalized = ecosystem.trim().to_ascii_lowercase();
        candidates.contains(&normalized.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ecosystems(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn plugin_server_maps_to_plugins_directory() {
        let targets = resource_targets(&ecosystems(&["paper"]));
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].kind, InstanceExtensionKind::Plugin);
        assert_eq!(targets[0].dir_name(), "plugins");
    }

    #[test]
    fn mod_server_maps_to_mods_directory() {
        let targets = resource_targets(&ecosystems(&["fabric"]));
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].kind, InstanceExtensionKind::Mod);
        assert_eq!(targets[0].dir_name(), "mods");
    }

    #[test]
    fn hybrid_server_lists_mods_before_plugins() {
        let hybrid = ecosystems(&["forge", "bukkit"]);
        let targets = resource_targets(&hybrid);

        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].kind, InstanceExtensionKind::Mod);
        assert_eq!(targets[1].kind, InstanceExtensionKind::Plugin);
    }

    #[test]
    fn unknown_ecosystems_yield_no_targets() {
        assert!(resource_targets(&ecosystems(&["vanilla"])).is_empty());
    }

    #[test]
    fn targets_report_primary_kind_from_first_entry() {
        // 模组优先：混合服务端的主要种类是模组。
        let hybrid = ResourceTargets::new(resource_targets(&ecosystems(&["forge", "bukkit"])));
        assert_eq!(hybrid.primary_kind, Some(InstanceExtensionKind::Mod));

        // 纯插件服。
        let plugin = ResourceTargets::new(resource_targets(&ecosystems(&["paper"])));
        assert_eq!(plugin.primary_kind, Some(InstanceExtensionKind::Plugin));

        // 无资源目录时没有主要种类。
        let none = ResourceTargets::new(resource_targets(&ecosystems(&["vanilla"])));
        assert!(none.targets.is_empty());
        assert_eq!(none.primary_kind, None);
    }

    #[test]
    fn ecosystem_matching_is_case_insensitive_and_trimmed() {
        let targets = resource_targets(&ecosystems(&[" Paper "]));
        assert_eq!(targets[0].kind, InstanceExtensionKind::Plugin);
    }

    #[test]
    fn datapack_target_uses_world_name() {
        let target = datapack_target("world_nether").expect("valid world name");
        assert_eq!(target.kind, InstanceExtensionKind::Datapack);
        assert_eq!(target.dir_name(), "world_nether/datapacks");
    }

    #[test]
    fn datapack_target_rejects_traversal_world_names() {
        for world_name in ["..", "../evil", "/abs", "a/b", "a\\b", "", "."] {
            let error = datapack_target(world_name).expect_err("world name must be rejected");
            assert_eq!(error, LayoutError::InvalidWorldName(world_name.to_string()));
        }
    }
}
