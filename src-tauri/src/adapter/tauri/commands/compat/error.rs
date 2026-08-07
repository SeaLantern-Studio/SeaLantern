//! 跨域错误映射辅助。
//!
//! 兼容命令中少数跨域调用（如 `get_server_resource_usage` 同时用 instance + system
//! 服务）需要把一个域的契约错误映射到另一个域，避免向前端抛出混合错误类型。
//! Tauri 命令返回类型必须固定为单一错误类型，故在此提供收敛映射。

use sealantern_interface::{InstanceServiceError, SystemServiceError};

/// 把实例服务错误映射为系统服务错误。
///
/// 用于 `get_server_resource_usage`：先调 instance 服务取 pid，
/// 失败时收敛为 `SystemServiceError` 向前端统一返回。
pub(crate) fn instance_err_to_system(error: InstanceServiceError) -> SystemServiceError {
    match error {
        InstanceServiceError::InstanceNotFound => SystemServiceError::ProcessNotFound,
        InstanceServiceError::Unsupported => SystemServiceError::Unsupported,
        _ => SystemServiceError::OperationFailed,
    }
}
