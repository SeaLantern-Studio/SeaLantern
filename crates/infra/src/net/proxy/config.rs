use serde::{Deserialize, Serialize};

/// 选择出站代理的可持久化意图。
///
/// 应用程序配置层负责该值的存储位置和迁移方式。此类型故意不包含文件系统行为。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxySettings {
    pub mode: ProxyMode,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self { mode: ProxyMode::Adaptive }
    }
}

impl ProxySettings {
    /// 在设置应用到控制器之前进行验证。
    pub fn validate(&self) -> Result<(), ProxyConfigError> {
        if let ProxyMode::Manual { proxy_url } = &self.mode {
            if proxy_url.trim().is_empty() {
                return Err(ProxyConfigError::EmptyManualProxy);
            }
        }

        Ok(())
    }
}

impl ProxyMode {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Adaptive => "adaptive",
            Self::Preserve => "preserve",
            Self::Manual { .. } => "manual",
            Self::Disabled => "disabled",
        }
    }
}

/// 选择出站代理的策略。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ProxyMode {
    /// 跟随操作系统报告的最新代理。
    Adaptive,
    /// 解析系统代理一次，然后忽略后续的网络变化。
    Preserve,
    /// 始终使用显式配置的代理 URL。
    Manual { proxy_url: String },
    /// 从不使用代理，即使操作系统报告了代理。
    Disabled,
}

/// 在构建 HTTP 客户端之前被拒绝的设置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyConfigError {
    EmptyManualProxy,
}

impl std::fmt::Display for ProxyConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyManualProxy => formatter.write_str("manual proxy URL must not be empty"),
        }
    }
}

impl std::error::Error for ProxyConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_adaptive() {
        assert_eq!(ProxySettings::default().mode, ProxyMode::Adaptive);
    }

    #[test]
    fn empty_manual_proxy_is_rejected() {
        let settings = ProxySettings {
            mode: ProxyMode::Manual { proxy_url: "  ".into() },
        };

        assert_eq!(settings.validate(), Err(ProxyConfigError::EmptyManualProxy));
    }
}
