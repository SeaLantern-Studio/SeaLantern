//! 插件市场相关的固定内容。

pub const PLUGIN_MARKET_BASE_URL: &str = "https://sealantern-studio.github.io/plugin-market";

pub const PLUGIN_MARKET_ALLOWED_DOWNLOAD_DOMAINS: &[&str] = &[
    "localhost",
    "sealanternpluginmarket.little100.top",
    "github.com",
    "raw.githubusercontent.com",
    "codeload.github.com",
    "api.github.com",
];

pub const GITHUB_PROFILE_BASE_URL: &str = "https://github.com";
pub const GITHUB_RELEASE_API_BASE_URL: &str = "https://api.github.com/repos";
pub const GITHUB_CODELOAD_BASE_URL: &str = "https://codeload.github.com";
pub const PLUGIN_MARKET_HTTP_USER_AGENT: &str = "SeaLantern";
