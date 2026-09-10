//! 服务器控制台日志契约模型。
//!
//! 定义宿主消费的控制台日志行，全部可序列化，供跨传输面传递。

/// 服务器控制台日志行（宿主消费的契约模型）。
///
/// `sequence` 为单调递增的行号游标，宿主可将其作为增量读取的
/// `since` 参数继续拉取后续日志。
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ConsoleLogLine {
    /// 行号（单调递增游标，用于增量读取）。
    pub sequence: i64,
    /// 写入时刻（Unix 秒）。
    pub timestamp: i64,
    /// 日志来源标识（`sealantern` / `server`）。
    pub source: String,
    /// 日志行文本。
    pub line: String,
}
