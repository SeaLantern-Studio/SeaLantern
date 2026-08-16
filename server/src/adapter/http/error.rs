//! HTTP 层错误响应。
//!
//! 将 `interface` 契约错误收敛为展平的 HTTP 错误响应：
//! `{ "code": "...", "message": "..." }`。状态码只做粗略分类，能告知前端
//! 请求失败即可，不追求细粒度映射。

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use sealantern_interface::{
    ConsoleServiceError, CronTaskServiceError, DownloadServiceError, InstanceServiceError,
    ServerServiceError, SettingsServiceError, SystemServiceError, UpdateCheckServiceError,
};

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
    /// 由应用更新检查契约错误构建 HTTP 错误。
    pub fn from_update_error(error: UpdateCheckServiceError) -> Self {
        match error {
            UpdateCheckServiceError::CheckFailed => Self {
                status: StatusCode::BAD_GATEWAY,
                code: "update_check_failed",
                message: error.to_string(),
            },
            UpdateCheckServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

    /// 由定时任务契约错误构建 HTTP 错误。
    pub fn from_cron_error(error: CronTaskServiceError) -> Self {
        match error {
            CronTaskServiceError::TaskNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "cron_task_not_found",
                message: error.to_string(),
            },
            CronTaskServiceError::InvalidInput => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_cron_task",
                message: error.to_string(),
            },
            CronTaskServiceError::StorageFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "cron_task_storage_failed",
                message: error.to_string(),
            },
            CronTaskServiceError::ExecutionFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "cron_task_execution_failed",
                message: error.to_string(),
            },
            CronTaskServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "cron_task_operation_failed",
                message: error.to_string(),
            },
            CronTaskServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

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
            InstanceServiceError::InvalidInput => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_input",
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

    /// 由服务器进程管理契约错误构建 HTTP 错误。
    pub fn from_server_error(error: ServerServiceError) -> Self {
        match error {
            ServerServiceError::InstanceNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "instance_not_found",
                message: error.to_string(),
            },
            ServerServiceError::InvalidState => Self {
                status: StatusCode::CONFLICT,
                code: "server_invalid_state",
                message: error.to_string(),
            },
            ServerServiceError::InvalidInput => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_input",
                message: error.to_string(),
            },
            ServerServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "server_operation_failed",
                message: error.to_string(),
            },
            ServerServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

    /// 由设置信息服务契约错误构建 HTTP 错误。
    pub fn from_settings_error(error: SettingsServiceError) -> Self {
        match error {
            SettingsServiceError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "settings_not_found",
                message: error.to_string(),
            },
            SettingsServiceError::InvalidInput => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_settings",
                message: error.to_string(),
            },
            SettingsServiceError::StorageFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "settings_storage_failed",
                message: error.to_string(),
            },
            SettingsServiceError::Unavailable => Self {
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "settings_unavailable",
                message: error.to_string(),
            },
            SettingsServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "settings_operation_failed",
                message: error.to_string(),
            },
            SettingsServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

    /// 由系统资源信息服务契约错误构建 HTTP 错误。
    pub fn from_system_error(error: SystemServiceError) -> Self {
        match error {
            SystemServiceError::ProcessNotFound | SystemServiceError::PathNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "system_resource_not_found",
                message: error.to_string(),
            },
            SystemServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "system_operation_failed",
                message: error.to_string(),
            },
            SystemServiceError::DefaultRunPathUnresolved => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "default_run_path_unresolved",
                message: error.to_string(),
            },
            SystemServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

    /// 由服务器控制台日志契约错误构建 HTTP 错误。
    pub fn from_console_error(error: ConsoleServiceError) -> Self {
        match error {
            ConsoleServiceError::InstanceNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "instance_not_found",
                message: error.to_string(),
            },
            ConsoleServiceError::InvalidInput => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_input",
                message: error.to_string(),
            },
            ConsoleServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "console_operation_failed",
                message: error.to_string(),
            },
            ConsoleServiceError::Unsupported => Self {
                status: StatusCode::NOT_IMPLEMENTED,
                code: "operation_unsupported",
                message: error.to_string(),
            },
        }
    }

    /// 由下载任务管理契约错误构建 HTTP 错误。
    pub fn from_download_error(error: DownloadServiceError) -> Self {
        match error {
            DownloadServiceError::TaskNotFound => Self {
                status: StatusCode::NOT_FOUND,
                code: "download_task_not_found",
                message: error.to_string(),
            },
            DownloadServiceError::InvalidInput => Self {
                status: StatusCode::BAD_REQUEST,
                code: "invalid_input",
                message: error.to_string(),
            },
            DownloadServiceError::OperationFailed => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "download_operation_failed",
                message: error.to_string(),
            },
            DownloadServiceError::Unsupported => Self {
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

impl From<CronTaskServiceError> for HttpError {
    fn from(error: CronTaskServiceError) -> Self {
        Self::from_cron_error(error)
    }
}

impl From<ServerServiceError> for HttpError {
    fn from(error: ServerServiceError) -> Self {
        Self::from_server_error(error)
    }
}

impl From<ConsoleServiceError> for HttpError {
    fn from(error: ConsoleServiceError) -> Self {
        Self::from_console_error(error)
    }
}

impl From<SettingsServiceError> for HttpError {
    fn from(error: SettingsServiceError) -> Self {
        Self::from_settings_error(error)
    }
}

impl From<SystemServiceError> for HttpError {
    fn from(error: SystemServiceError) -> Self {
        Self::from_system_error(error)
    }
}

impl From<DownloadServiceError> for HttpError {
    fn from(error: DownloadServiceError) -> Self {
        Self::from_download_error(error)
    }
}

impl From<UpdateCheckServiceError> for HttpError {
    fn from(error: UpdateCheckServiceError) -> Self {
        Self::from_update_error(error)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let body = HttpErrorBody { code: self.code, message: self.message };
        (self.status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn update_check_failure_has_stable_http_response() {
        let response = HttpError::from(UpdateCheckServiceError::CheckFailed).into_response();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .expect("read error response");
        let value: serde_json::Value = serde_json::from_slice(&body).expect("parse error response");
        assert_eq!(value["code"], "update_check_failed");
        assert_eq!(value["message"], "update check failed");
    }

    #[test]
    fn unsupported_update_check_maps_to_not_implemented() {
        let error = HttpError::from(UpdateCheckServiceError::Unsupported);

        assert_eq!(error.status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(error.code, "operation_unsupported");
    }

    #[test]
    fn settings_errors_have_stable_http_classification() {
        let invalid = HttpError::from(SettingsServiceError::InvalidInput);
        assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
        assert_eq!(invalid.code, "invalid_settings");

        let storage = HttpError::from(SettingsServiceError::StorageFailed);
        assert_eq!(storage.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(storage.code, "settings_storage_failed");

        let unavailable = HttpError::from(SettingsServiceError::Unavailable);
        assert_eq!(unavailable.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(unavailable.code, "settings_unavailable");
    }
}
