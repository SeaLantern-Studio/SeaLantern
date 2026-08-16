//! 更新检查与安装模块。
//!
//! 支持从多个来源检查 SeaLantern 的版本更新：
//!
//! | 来源 | 模块 | 适用平台 |
//! |------|------|----------|
//! | GitHub Releases | [`github`] | 全平台（主要分发渠道） |
//! | CNB.cool | [`cnb`] | 全平台（国内镜像） |
//! | AUR | [`arch`] | Arch Linux |
//!
//! 包含文件下载、SHA256 校验和验证、版本号语义比较和安装管理等完整流程。

mod checker;
mod error;
mod install;

pub mod arch;
pub mod checksum;
pub mod cnb;
pub mod constants;
pub mod download;
pub mod github;
pub mod types;
pub mod version;

#[cfg(target_os = "linux")]
pub use arch::check_aur_update;
pub use arch::{get_aur_helper, is_arch_linux};
pub use checker::{ReleaseUpdateChecker, UpdateChecker};
pub use checksum::{
    fetch_sha256_from_asset, find_sha256_assets, parse_sha256_from_checksum_content,
    resolve_asset_sha256,
};
pub use cnb::{fetch_release as fetch_cnb_release, resolve_download_candidate_by_version};
pub use constants::UPDATE_HTTP_USER_AGENT;
pub use download::{calculate_progress, download_update_file_without_events, file_name_from_url};
pub use error::UpdateCheckError;
pub use github::{fetch_release as fetch_github_release, find_suitable_asset};
pub use install::{
    check_pending_update, clear_pending_update, get_pending_update_file, get_update_cache_dir,
    write_pending_update, INSTALL_IN_PROGRESS,
};
pub use types::{
    get_github_config, DownloadProgress, PendingUpdate, ReleaseAsset, ReleaseResponse, RepoConfig,
    UpdateInfo,
};
pub use version::{
    compare_versions, normalize_release_tag_version, parse_version, ParsedVersion, PreIdent,
};

#[cfg(target_os = "windows")]
pub use install::windows::spawn_elevated_windows_process;
