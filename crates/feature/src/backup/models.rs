use std::path::{Component, Path};

/// 校验路径片段是否安全（feature 内部使用，不跨宿主暴露）。
pub(crate) fn is_safe_path_component(value: &str) -> bool {
    if value.is_empty() || value.contains(['/', '\\', ':', '\0']) || value.contains(['\n', '\r']) {
        return false;
    }

    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

// 备份契约模型统一由 `sealantern-contract` 提供，feature 侧 re-export 保持兼容。
pub use sealantern_contract::backup::{
    BackupContentType, BackupFormat, BackupItem, BackupSettings, CompressionLevel,
    CreateBackupRequest,
};
