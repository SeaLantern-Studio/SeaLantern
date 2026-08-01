use java_manager::JavaInfo as VendorJavaInfo;

use super::error::JavaValidationError;
use super::mapping::to_app_java_info;
use crate::config::JavaInfo;
use crate::observability;

/// 校验并读取指定路径下的 Java 安装信息。
pub fn validate_java(path: &str) -> Result<JavaInfo, JavaValidationError> {
    let path = path.trim();
    observability::java_validation_started(path);
    if path.is_empty() {
        let error = JavaValidationError::EmptyPath;
        observability::java_validation_failed(path, &error);
        return Err(error);
    }

    match VendorJavaInfo::new(path.to_string()) {
        Ok(info) => {
            let info = to_app_java_info(info);
            observability::java_validation_completed(path, info.major_version);
            Ok(info)
        }
        Err(error) => {
            let error = JavaValidationError::InvalidInstallation {
                path: path.to_string(),
                message: error.to_string(),
            };
            observability::java_validation_failed(path, &error);
            Err(error)
        }
    }
}
