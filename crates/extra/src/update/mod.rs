mod install;

pub mod constants;
pub mod version;
pub mod types;
pub mod arch;
pub mod checksum;
pub mod cnb;
pub mod download;
pub mod github;

pub use constants::UPDATE_HTTP_USER_AGENT;
pub use install::{get_update_cache_dir, get_pending_update_file, check_pending_update, clear_pending_update, write_pending_update, INSTALL_IN_PROGRESS};
pub use types::{UpdateInfo, DownloadProgress, PendingUpdate, ReleaseResponse, ReleaseAsset, RepoConfig, get_github_config};
pub use version::{compare_versions, parse_version, normalize_release_tag_version, ParsedVersion, PreIdent};
pub use arch::{is_arch_linux, get_aur_helper};
#[cfg(target_os = "linux")]
pub use arch::check_aur_update;
pub use checksum::{parse_sha256_from_checksum_content, find_sha256_assets, fetch_sha256_from_asset, resolve_asset_sha256};
pub use cnb::{fetch_release as fetch_cnb_release, resolve_download_candidate_by_version};
pub use download::{file_name_from_url, calculate_sha256, download_update_file_without_events, calculate_progress};
pub use github::{find_suitable_asset, fetch_release as fetch_github_release};

#[cfg(target_os = "windows")]
pub use install::windows::spawn_elevated_windows_process;