pub const STARTER_DOWNLOAD_API_BASE_DEFAULT: &str = "https://api.mslmc.cn/v3/download/server";
pub const STARTER_DOWNLOAD_API_BASE_ENV: &str = "SEALANTERN_STARTER_DOWNLOAD_API_BASE";

pub fn starter_download_api_base() -> String {
    std::env::var(STARTER_DOWNLOAD_API_BASE_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| STARTER_DOWNLOAD_API_BASE_DEFAULT.to_string())
}
