//! 服务器日志实时事件转发（Tauri 宿主侧）。
//!
//! 订阅 application 层的全局日志事件广播（[`subscribe_log_events`]），
//! 将日志行转发为 Tauri 前端事件 `server-log-line`，使前端控制台能
//! 实时展示服务器输出，无需轮询。
//!
//! [`LogSenderState`] 负责转发任务的生命周期：
//! - `start` 惰性启动单个转发任务（幂等，重复调用不会叠加任务）；
//! - `stop` 通过 `watch` 通道发送停止信号，转发任务自主退出后等待回收；
//! - 任务内以 `tokio::select!` 同时监听停止信号与日志事件，避免
//!   阻塞在事件接收上无法响应停止请求。

use sealantern_application::service::subscribe_log_events;
use std::sync::Arc;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, async_runtime::spawn};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{Mutex, watch};

/// 服务器日志事件转发器的运行状态。
///
/// 持有 `watch` 停止信号与后台转发任务的句柄；作为 Tauri 托管状态
/// （`.manage()`）供 setup 启动、窗口销毁时停止。
pub struct LogSenderState {
    /// 运行标志（`watch` 发送端）：`start` 置 `true`，`stop` 置 `false`，
    /// 置值会唤醒转发任务中等待的 `changed()` 分支。
    running: watch::Sender<bool>,
    /// 后台转发任务的句柄；`None` 表示未启动或已停止。
    handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Default for LogSenderState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSenderState {
    /// 创建处于停止状态的转发器。
    pub fn new() -> LogSenderState {
        let (running, _) = watch::channel(false);
        Self {
            running,
            handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动日志事件转发任务（幂等）。
    ///
    /// 若已有转发任务在运行则直接返回，避免重复订阅广播产生重复推送。
    /// 任务订阅全局日志广播并转发为 `server-log-line` 事件；停止信号
    /// 到来或广播关闭时自行退出。
    pub async fn start(&self, app_handle: AppHandle) {
        if self.handle.lock().await.is_some() {
            return;
        }

        // 置运行标志并持有其接收端：`stop` 改值即可唤醒下方 `select`。
        let _ = self.running.send(true);
        let mut stop_signal = self.running.subscribe();

        let handle = spawn(async move {
            let mut receiver = subscribe_log_events();
            loop {
                tokio::select! {
                    // 停止信号：`stop()` 已调用，任务自行退出。
                    _ = stop_signal.changed() => {
                        break
                    }
                    // 日志事件：转发到前端。
                    event = receiver.recv() => match event {
                        Ok(event) => {
                            if let Err(e) = app_handle.emit("server-log-line", &event) {
                                tracing::error!(
                                    target: "sealantern.tauri.server_log",
                                    error = e.to_string(),
                                    "failed to emit server log event"
                                )
                            }
                        }
                        // 广播通道关闭：静态广播不会发生，防御性退出。
                        Err(RecvError::Closed) => {
                            break;
                        }
                        // 消费落后被跳过 n 条：预期背压行为，仅记录提示。
                        Err(RecvError::Lagged(n)) => {
                            tracing::warn!(target: "sealantern.tauri.server_log", skipped = n, "server log subscriber lagged")
                        }
                    }
                }
            }
        });
        *self.handle.lock().await = Some(handle);
    }

    /// 停止转发任务：发送停止信号并等待任务自行退出。
    ///
    /// 采用优雅停止（而非 `abort` 强杀），保证任务内正在进行的
    /// 事件转发不被中途打断；重复调用安全（句柄已取出）。
    pub async fn stop(&self) {
        let _ = self.running.send(false);
        if let Some(handle) = self.handle.lock().await.take() {
            let _ = handle.await;
        }
    }
}
