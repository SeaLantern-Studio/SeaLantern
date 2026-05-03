//! 外部服务地址。

pub const COMMON_HTTP_BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36 Edg/145.0.0.0";

#[cfg_attr(not(target_os = "linux"), allow(dead_code))] // Linux 调用
pub const AUR_PACKAGE_INFO_URL: &str = "https://aur.archlinux.org/rpc/v5/info/sealantern";
#[cfg_attr(not(target_os = "linux"), allow(dead_code))] // Linux 调用
pub const AUR_PACKAGE_PAGE_URL: &str = "https://aur.archlinux.org/packages/sealantern";

pub const MODRINTH_SEARCH_API_URL: &str = "https://api.modrinth.com/v2/search";
pub const MODRINTH_PROJECT_VERSION_API_BASE: &str = "https://api.modrinth.com/v2/project";
