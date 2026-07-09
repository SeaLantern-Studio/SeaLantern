pub(crate) const CNB_BASE_URL: &str = "https://cnb.cool";
pub(crate) const CNB_RELEASES_URL: &str = "https://cnb.cool/_next/data/f79eab91170dd314/zh/SeaLantern-studio/SeaLantern/-/releases.json?slug=SeaLantern-studio&slug=SeaLantern&slug=-&slug=releases";

#[cfg(not(debug_assertions))]
pub(crate) const UPDATE_GITHUB_OWNER: &str = "SeaLantern-Studio";
#[cfg(not(debug_assertions))]
pub(crate) const UPDATE_GITHUB_REPO: &str = "SeaLantern";
#[cfg(not(debug_assertions))]
pub(crate) const UPDATE_GITHUB_API_BASE: &str = "https://api.github.com/repos";

pub const UPDATE_HTTP_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

#[cfg_attr(not(all(target_os = "linux", not(debug_assertions))), allow(dead_code))]
pub(crate) const AUR_PACKAGE_INFO_URL: &str = "https://aur.archlinux.org/rpc/v5/info/sealantern";

#[cfg_attr(not(all(target_os = "linux", not(debug_assertions))), allow(dead_code))]
pub(crate) const AUR_PACKAGE_PAGE_URL: &str = "https://aur.archlinux.org/packages/sealantern";

#[cfg(all(target_os = "linux", not(debug_assertions)))]
pub(crate) const PLUGIN_MARKET_HTTP_USER_AGENT: &str = "SeaLantern";
