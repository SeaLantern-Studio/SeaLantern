//! 服务器控制台日志服务端口。

use async_trait::async_trait;
use sealantern_contract::{ConsoleLogLine, ConsoleServiceError};
use sealantern_core::instance::InstanceId;

/// 服务器控制台日志宿主能力端口。
///
/// 提供服务器进程输出的持久化日志读取：以行号游标增量拉取，并支持
/// "最近 N 行"滚动窗口。实现方组合 application 的日志存储能力，
/// 不依赖任何具体宿主。
#[async_trait]
pub trait ConsoleService: Send + Sync {
    /// 读取 `sequence` 大于 `since` 的控制台日志行，按行号升序返回。
    ///
    /// `recent_limit` 提供时，仅返回最近 `recent_limit` 行窗口内的匹配行。
    async fn logs(
        &self,
        id: &InstanceId,
        since: i64,
        recent_limit: Option<i64>,
    ) -> Result<Vec<ConsoleLogLine>, ConsoleServiceError>;
}
