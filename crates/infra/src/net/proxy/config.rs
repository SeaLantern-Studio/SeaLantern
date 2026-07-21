use serde::{Deserialize, Serialize};

/// Persistable intent for selecting an outbound proxy.
///
/// The application configuration layer owns where this value is stored and how
/// it is migrated. This type deliberately contains no file-system behavior.
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
    /// Validates settings before they are applied to a controller.
    pub fn validate(&self) -> Result<(), ProxyConfigError> {
        if let ProxyMode::Manual { proxy_url } = &self.mode {
            if proxy_url.trim().is_empty() {
                return Err(ProxyConfigError::EmptyManualProxy);
            }
        }

        Ok(())
    }
}

/// Policy used to select an outbound proxy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ProxyMode {
    /// Follow the latest proxy reported by the operating system.
    Adaptive,
    /// Resolve the system proxy once and ignore subsequent network changes.
    Preserve,
    /// Always use the explicitly configured proxy URL.
    Manual { proxy_url: String },
    /// Never use a proxy, even when the operating system reports one.
    Disabled,
}

/// Settings rejected before an HTTP client is constructed.
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
