//! 路径组件安全校验（纯逻辑，零 IO）。
//!
//! 资源管理器接受来自前端与 `server.properties` 的名称（资源文件名、
//! 世界名）。这些输入必须限制为"单一普通路径组件"，否则 `Path::join`
//! 会被绝对路径替换整条路径、或被 `..` 逃逸到实例目录之外。

use std::path::{Component, Path};

/// 判断名称是否为单一普通路径组件。
///
/// 拒绝：空串、`.`、`..`、含路径分隔符（`/` 与 `\`，跨平台一致）、
/// 绝对路径与盘符前缀。
pub fn is_single_normal_component(name: &str) -> bool {
    // 显式拒绝两种分隔符：`Component` 在非 Windows 平台不把 `\` 视为分隔符，
    // 但资源要在 Windows 上落盘，必须统一拒绝。
    if name.is_empty() || name.contains(['/', '\\']) {
        return false;
    }

    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_names() {
        assert!(is_single_normal_component("sodium.jar"));
        assert!(is_single_normal_component("sodium.jar.disabled"));
        assert!(is_single_normal_component("world_nether"));
        assert!(is_single_normal_component("模组.jar"));
    }

    #[test]
    fn rejects_empty_current_and_parent_components() {
        assert!(!is_single_normal_component(""));
        assert!(!is_single_normal_component("."));
        assert!(!is_single_normal_component(".."));
    }

    #[test]
    fn rejects_separators_and_traversal() {
        assert!(!is_single_normal_component("a/b"));
        assert!(!is_single_normal_component("a\\b"));
        assert!(!is_single_normal_component("../x.jar"));
        assert!(!is_single_normal_component("..\\x.jar"));
        assert!(!is_single_normal_component("/etc/passwd"));
        assert!(!is_single_normal_component("C:\\Windows\\system32"));
    }
}
