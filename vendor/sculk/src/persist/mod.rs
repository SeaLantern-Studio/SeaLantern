//! 应用数据持久化：iroh 密钥管理与用户偏好 TOML Profile。
//!
//! 需在 `sculk` 中启用 `persist` feature（默认关闭）。
//! 数据目录为 `{dirs::data_dir()}/sculk/`：
//! - macOS：`~/Library/Application Support/sculk/`
//! - Linux：`~/.local/share/sculk/`
//! - Windows：`%APPDATA%\sculk\`

mod key;
mod profile;

use std::path::PathBuf;

use crate::Result;
use crate::error::PersistError;

pub use key::{generate_new_key, load_or_generate_key};
pub use profile::{HostProfile, JoinProfile, Profile, RelayProfile};

/// 应用数据目录。
pub fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join("sculk"))
        .ok_or_else(|| PersistError::SystemDataDirUnavailable.into())
}

/// 默认密钥文件路径。
pub fn default_key_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("secret.key"))
}
