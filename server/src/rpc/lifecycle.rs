//! RPC 调用的协作式取消与截止时间控制。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 可由传输适配器或任务协调器取消的调用令牌。
///
/// 取消是协作式的：它不会强制终止线程、future 或子进程。RPC 调度器会在方法调用前后
/// 检查该令牌；耗时方法应在自己的循环和 I/O 边界调用 [`crate::rpc::RpcContext::check_active`]。
#[derive(Clone, Debug, Default)]
pub struct RpcCancellationToken(Arc<AtomicBool>);

impl RpcCancellationToken {
    /// 创建尚未取消的令牌。
    pub fn new() -> Self {
        Self::default()
    }

    /// 请求取消当前调用。
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// 判断调用是否已被请求取消。
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// 单调时钟上的 RPC 调用截止时间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RpcDeadline(Instant);

impl RpcDeadline {
    /// 使用调用方已计算的单调时钟时间点创建截止时间。
    pub const fn at(instant: Instant) -> Self {
        Self(instant)
    }

    /// 判断是否已经超过截止时间。
    pub fn has_expired(self) -> bool {
        Instant::now() >= self.0
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn cancellation_is_visible_to_all_token_clones() {
        let token = RpcCancellationToken::new();
        let observer = token.clone();

        token.cancel();

        assert!(observer.is_cancelled());
    }

    #[test]
    fn recognizes_an_elapsed_deadline() {
        let deadline = RpcDeadline::at(Instant::now() - Duration::from_secs(1));

        assert!(deadline.has_expired());
    }
}
