//! HTTP 层错误响应。
//!
//! 将 `interface` 契约错误收敛为展平的 HTTP 错误响应：
//! `{ "code": "...", "message": "..." }`。状态码只做粗略分类，能告知前端
//! 请求失败即可，不追求细粒度映射。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use sealantern_interface::InstanceServiceError;

/// 展平的 HTTP 错误响应体。
#[derive(Debug, Serialize)]
pub struct HttpErrorBody {
    /// 稳定的错误代码（机器可读）。
    code: &'static str,
    /// 人类可读的错误消息。
    message: String,
}

/// HTTP 传输层的统一错误类型。
///
/// 当前直接封装接口契约错误 [`InstanceServiceError`]；后续接入认证/校验后
/// 可在此扩展为携带更多类别的枚举。
#[derive(Debug)]
pub struct HttpError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl HttpError {
    /// 由接口契约错误构建 HTTP 错误。
    pub fn from_instance_error(error: InstanceServiceError) -> Self {
        match error {
            InstanceServiceError::InstanceNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "instance_not_found",
                message: error.to_string(),
            },
            InstanceServiceError::AlreadyExists => Self {
                status: StatusCode::CONFLICT,
                code: "instance_already_exists",
                message: error.to_string(),
            },
            InstanceServiceError::InvalidState => Self {
                status: StatusCode::BAD_REQUEST,
                code: "instance_invalid_state",
                message: error.to_string(),
            },
            InstanceServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "instance_operation_failed",
                message: error.to_string(),
            },
            InstanceServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

    /// 构建一个客户端输入错误（400），带具体错误码。
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    /// 构建一个通用内部错误（供非实例类失败使用）。
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

impl From<InstanceServiceError> for HttpError {
    fn from(error: InstanceServiceError) -> Self {
        Self::from_instance_error(error)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let body = HttpErrorBody { code: self.code, message: self.message };
        (self.status, Json(body)).into_response()
    }
}
