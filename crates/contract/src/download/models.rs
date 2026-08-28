//! 下载任务契约模型。
//!
//! 定义宿主消费的下载任务信息模型，全部可序列化，供跨传输面传递。

/// 下载任务状态（对齐前端 `TaskStatus` 形状）。
///
/// 手写序列化以匹配前端期望：unit 变体输出为字符串
/// （`"Pending"`/`"Downloading"`/`"Completed"`），错误变体输出为
/// `{ "Error": "..." }` 对象。
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
        use serde::ser::SerializeMap;
        match self {
            Self::Pending => serializer.serialize_str("Pending"),
            Self::Downloading => serializer.serialize_str("Downloading"),
            Self::Completed => serializer.serialize_str("Completed"),
            Self::Error(message) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("Error", message)?;
                map.end()
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
        // unit 变体输出为字符串（前端期望 "Pending"/"Downloading"/"Completed"）。
        assert_eq!(serde_json::to_string(&DownloadTaskStatus::Pending).unwrap(), "\"Pending\"");
        assert_eq!(
            serde_json::to_string(&DownloadTaskStatus::Downloading).unwrap(),
            "\"Downloading\""
        );
        assert_eq!(serde_json::to_string(&DownloadTaskStatus::Completed).unwrap(), "\"Completed\"");
        // 错误变体输出为 { "Error": "..." } 对象（前端 errorMessage 判定 "Error" in status）。
        assert_eq!(
            serde_json::to_string(&DownloadTaskStatus::Error("boom".into())).unwrap(),
            r#"{"Error":"boom"}"#
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
