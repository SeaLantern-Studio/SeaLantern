use std::fmt;

pub use sealantern_contract::java::JavaDiscoveryError;

/// 显式 Java 路径校验错误。
#[derive(Debug)]
pub enum JavaValidationError {
    EmptyPath,
    InvalidInstallation { path: String, message: String },
}

impl fmt::Display for JavaValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => formatter.write_str("Java 路径不能为空"),
            Self::InvalidInstallation { path, message } => {
                write!(formatter, "无法验证 Java 路径 '{}': {message}", path)
            }
        }
    }
}

impl std::error::Error for JavaValidationError {}

#[cfg(test)]
mod tests {
    use super::JavaValidationError;

    #[test]
    fn validation_error_keeps_path_context() {
        let error = JavaValidationError::InvalidInstallation {
            path: "C:\\Java\\missing".to_string(),
            message: "not found".to_string(),
        };

        assert!(error.to_string().contains("C:\\Java\\missing"));
        assert!(error.to_string().contains("not found"));
    }
}
