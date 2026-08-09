//! 更新相关的常量定义。

use std::time::Duration;

pub const CNB_BASE_URL: &str = "https://cnb.cool";
pub const CNB_RELEASES_URL: &str = "https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases";

pub const UPDATE_GITHUB_OWNER: &str = "SeaLantern-Studio";
pub const UPDATE_GITHUB_REPO: &str = "SeaLantern";
pub const UPDATE_GITHUB_API_BASE: &str = "https://api.github.com/repos";

pub const UPDATE_HTTP_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
pub(crate) const UPDATE_HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const UPDATE_HTTP_TIMEOUT: Duration = Duration::from_secs(20);

pub const PLUGIN_MARKET_HTTP_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0";

pub const AUR_PACKAGE_INFO_URL: &str = "https://aur.archlinux.org/rpc/v5/info/sealantern";
pub const AUR_PACKAGE_PAGE_URL: &str = "https://aur.archlinux.org/packages/sealantern";
