use std::fmt;
use std::path::PathBuf;

/// Errors raised while discovering, validating, or running an app plugin.
#[derive(Debug)]
pub enum AppPluginError {
    /// The plugin targets an API that predates the breaking v2 contract.
    ApiVersionTooOld { found: Option<u32> },
    /// The plugin targets an API newer than this host supports.
    UnsupportedApiVersion { found: u32, supported: u32 },
    /// The manifest is not valid JSON or does not satisfy the v2 schema.
    MalformedManifest { path: PathBuf, message: String },
    /// A manifest field violates a filesystem-safe plugin constraint.
    InvalidPath { path: PathBuf, message: String },
    /// A required manifest field is missing or contains an invalid value.
    InvalidManifest { message: String },
    /// The manifest requests a capability that this plugin API does not expose.
    UnsupportedCapability { capability: String },
    /// A plugin file or directory could not be accessed.
    Io { path: PathBuf, message: String },
    /// The Lua plugin engine could not initialize or execute a plugin callback.
    Engine(String),
    /// Plugin private storage could not be read or written.
    Storage {
        operation: &'static str,
        message: String,
    },
    /// A plugin script exhausted the instruction budget for one execution phase.
    ExecutionLimit { operation: &'static str },
}

impl fmt::Display for AppPluginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApiVersionTooOld { .. } => formatter.write_str("版本过旧"),
            Self::UnsupportedApiVersion { found, supported } => write!(
                formatter,
                "unsupported plugin API version {found}; this host supports version {supported}"
            ),
            Self::MalformedManifest { path, message } => {
                write!(formatter, "invalid plugin manifest at {}: {message}", path.display())
            }
            Self::InvalidPath { path, message } => {
                write!(formatter, "invalid plugin path {}: {message}", path.display())
            }
            Self::InvalidManifest { message } => {
                write!(formatter, "invalid plugin manifest: {message}")
            }
            Self::UnsupportedCapability { capability } => {
                write!(formatter, "unsupported plugin capability: {capability}")
            }
            Self::Io { path, message } => {
                write!(formatter, "failed to access plugin path {}: {message}", path.display())
            }
            Self::Engine(message) => write!(formatter, "plugin engine error: {message}"),
            Self::Storage { operation, message } => {
                write!(formatter, "plugin storage failed to {operation}: {message}")
            }
            Self::ExecutionLimit { operation } => {
                write!(formatter, "plugin execution limit exceeded during {operation}")
            }
        }
    }
}

impl std::error::Error for AppPluginError {}

impl AppPluginError {
    /// 返回可安全写入结构化日志的稳定错误类别。
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ApiVersionTooOld { .. } => "api_version_too_old",
            Self::UnsupportedApiVersion { .. } => "unsupported_api_version",
            Self::MalformedManifest { .. } => "malformed_manifest",
            Self::InvalidPath { .. } => "invalid_path",
            Self::InvalidManifest { .. } => "invalid_manifest",
            Self::UnsupportedCapability { .. } => "unsupported_capability",
            Self::Io { .. } => "io",
            Self::Engine(_) => "engine",
            Self::Storage { .. } => "storage",
            Self::ExecutionLimit { .. } => "execution_limit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppPluginError;

    #[test]
    fn old_api_message_is_stable() {
        assert_eq!(AppPluginError::ApiVersionTooOld { found: None }.to_string(), "版本过旧");
    }

    #[test]
    fn future_api_message_identifies_supported_version() {
        let error = AppPluginError::UnsupportedApiVersion { found: 3, supported: 2 };

        assert_eq!(
            error.to_string(),
            "unsupported plugin API version 3; this host supports version 2"
        );
    }
}
