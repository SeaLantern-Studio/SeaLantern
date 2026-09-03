//! 下载任务契约模型。
//!
//! 定义宿主消费的下载任务信息模型，全部可序列化，供跨传输面传递。

/// 下载任务状态（对齐前端 `TaskStatus` 形状）。
///
/// 使用稳定的判别联合，避免让展示文本同时承担协议字段的职责。
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadTaskStatus {
    /// 排队中（尚未开始）。
    Pending,
    /// 下载中。
    Downloading,
    /// 已完成。
    Completed,
    /// 失败（携带错误信息）。
    Error(String),
}

impl serde::Serialize for DownloadTaskStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        match self {
            Self::Pending => {
                let mut state = serializer.serialize_struct("TaskStatus", 2)?;
                state.serialize_field("kind", "simple")?;
                state.serialize_field("message", "Pending")?;
                state.end()
            }
            Self::Downloading => {
                let mut state = serializer.serialize_struct("TaskStatus", 2)?;
                state.serialize_field("kind", "simple")?;
                state.serialize_field("message", "Downloading")?;
                state.end()
            }
            Self::Completed => {
                let mut state = serializer.serialize_struct("TaskStatus", 2)?;
                state.serialize_field("kind", "simple")?;
                state.serialize_field("message", "Completed")?;
                state.end()
            }
            Self::Error(message) => {
                let mut state = serializer.serialize_struct("TaskStatus", 2)?;
                state.serialize_field("kind", "error")?;
                state.serialize_field("message", message)?;
                state.end()
            }
        }
    }
}

/// 下载任务信息（宿主消费的契约模型，对齐前端 `DownloadTaskInfo`）。
///
/// `rename_all = "snake_case"` 显式声明契约字段命名：序列化输出
/// `total_size` / `is_finished` 等 snake_case 字段名，与其余契约模型
/// 保持一致（前端 `src/api/downloader.ts` 同步消费 snake_case 形状）。
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DownloadTaskInfo {
    /// 任务标识。
    pub id: String,
    /// 文件总大小（字节）。
    pub total_size: u64,
    /// 已下载字节数。
    pub downloaded: u64,
    /// 进度百分比（0.0 - 100.0）。
    pub progress: f64,
    /// 任务状态。
    pub status: DownloadTaskStatus,
    /// 是否已结束（完成、出错或取消）。
    pub is_finished: bool,
}

/// 创建下载任务的输入。
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    /// 下载 URL。
    pub url: String,
    /// 本地保存路径。
    pub save_path: String,
    /// 下载线程数。
    pub thread_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_serializes_as_frontend_shape() {
        // 状态使用稳定的 kind/message 判别联合。
        assert_eq!(
            serde_json::to_string(&DownloadTaskStatus::Pending).unwrap(),
            r#"{"kind":"simple","message":"Pending"}"#
        );
        assert_eq!(
            serde_json::to_string(&DownloadTaskStatus::Downloading).unwrap(),
            r#"{"kind":"simple","message":"Downloading"}"#
        );
        assert_eq!(
            serde_json::to_string(&DownloadTaskStatus::Completed).unwrap(),
            r#"{"kind":"simple","message":"Completed"}"#
        );
        // 错误状态同样保持 kind/message 结构。
        assert_eq!(
            serde_json::to_string(&DownloadTaskStatus::Error("boom".into())).unwrap(),
            r#"{"kind":"error","message":"boom"}"#
        );
    }

    #[test]
    fn task_info_serializes_snake_case_fields() {
        // 契约统一使用 snake_case（total_size/is_finished），与其余模型一致。
        let info = DownloadTaskInfo {
            id: "abc".into(),
            total_size: 100,
            downloaded: 60,
            progress: 60.0,
            status: DownloadTaskStatus::Downloading,
            is_finished: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"total_size\":100"), "missing total_size: {json}");
        assert!(json.contains("\"is_finished\":false"), "missing is_finished: {json}");
        assert!(!json.contains("totalSize"), "camelCase leaked: {json}");
        assert!(!json.contains("isFinished"), "camelCase leaked: {json}");
    }
}
