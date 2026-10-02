//! 服务端事件总线（协议无关）。
//!
//! 对 `tokio::sync::broadcast` 的薄封装：发布者与订阅者通过它解耦，
//! 传输适配器（[`ws`](super::ws)）只依赖本模块，不感知事件来源。

use tokio::sync::broadcast;

use super::ServerEvent;

/// 广播通道默认容量：订阅者最多可落后多少条事件。
///
/// 与 application 层日志广播保持一致，使两处的"落后窗口"语义相同。
pub const DEFAULT_EVENT_CAPACITY: usize = 1024;

/// server 进程内的事件总线。
///
/// 由宿主创建并注入（而非全局静态），便于测试隔离与观测订阅者数量：
/// 所有发布者共享同一实例，每个订阅者持有独立的接收端。
pub struct ServerEventBus {
    sender: broadcast::Sender<ServerEvent>,
}

impl ServerEventBus {
    /// 按指定容量创建总线。
    pub fn new(capacity: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(capacity);
        Self { sender }
    }

    /// 按默认容量创建总线。
    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_EVENT_CAPACITY)
    }

    /// 发布事件，返回接收者数量。
    ///
    /// 当前无订阅者时事件被丢弃（返回 `0`）——事件不保证送达，调用方无需
    /// 依赖该返回值，它仅用于观测。
    pub fn publish(&self, event: ServerEvent) -> usize {
        self.sender.send(event).unwrap_or(0)
    }

    /// 订阅事件流；每个连接应持有独立接收端。
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }

    /// 当前订阅者数量（观测与测试用）。
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sealantern_contract::console::ConsoleLogLine;

    fn sample_event(label: &str) -> ServerEvent {
        ServerEvent::log(
            "instance-a",
            ConsoleLogLine {
                sequence: 1,
                timestamp: 0,
                source: "server".to_owned(),
                line: format!("line from {label}"),
            },
        )
    }

    #[tokio::test]
    async fn publish_reaches_every_subscriber() {
        let bus = ServerEventBus::new(8);
        let mut first = bus.subscribe();
        let mut second = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);
        assert_eq!(bus.publish(sample_event("server")), 2);

        let first_event = first.recv().await.expect("第一个订阅者应收到事件");
        let second_event = second.recv().await.expect("第二个订阅者应收到事件");
        assert_eq!(first_event.instance_id, "instance-a");
        assert_eq!(second_event.instance_id, "instance-a");
    }

    #[tokio::test]
    async fn publishing_without_subscribers_is_ignored() {
        let bus = ServerEventBus::new(8);
        assert_eq!(bus.publish(sample_event("server")), 0);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn slow_subscriber_observes_lag_instead_of_blocking_publishers() {
        let bus = ServerEventBus::new(2);
        let mut slow = bus.subscribe();

        for index in 0..4 {
            bus.publish(sample_event(&format!("line-{index}")));
        }

        // 容量 2、已发 4 条：接收端应报告落后，而不是静默丢数据。
        assert!(matches!(slow.recv().await, Err(broadcast::error::RecvError::Lagged(_))));
    }

    #[tokio::test]
    async fn a_dropped_subscriber_is_no_longer_counted() {
        let bus = ServerEventBus::new(8);
        let subscriber = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        drop(subscriber);
        assert_eq!(bus.subscriber_count(), 0);
    }
}
