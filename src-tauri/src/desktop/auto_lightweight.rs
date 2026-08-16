//! 自动进入轻量模式的宿主侧延时任务。

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct AutoLightweightState {
    minutes: AtomicU32,
    generation: Arc<AtomicU64>,
}

impl AutoLightweightState {
    pub fn new() -> Self {
        Self {
            minutes: AtomicU32::new(0),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn configure(&self, minutes: Option<u32>) {
        self.minutes.store(minutes.unwrap_or(0), Ordering::Release);
        self.cancel();
    }

    pub fn delay(&self) -> Option<Duration> {
        let minutes = self.minutes.load(Ordering::Acquire);
        (minutes > 0).then(|| Duration::from_secs(u64::from(minutes) * 60))
    }

    pub fn cancel(&self) {
        self.advance();
    }

    pub fn schedule(
        &self,
        delay: Duration,
        action: impl FnOnce(AutoLightweightTicket) + Send + 'static,
    ) {
        let ticket = AutoLightweightTicket {
            generation: self.advance(),
            current: Arc::clone(&self.generation),
        };
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(delay).await;
            if ticket.is_current() {
                action(ticket);
            }
        });
    }

    fn advance(&self) -> u64 {
        self.generation
            .fetch_add(1, Ordering::AcqRel)
            .wrapping_add(1)
    }
}

impl Default for AutoLightweightState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AutoLightweightTicket {
    generation: u64,
    current: Arc<AtomicU64>,
}

impl AutoLightweightTicket {
    pub fn is_current(&self) -> bool {
        self.current.load(Ordering::Acquire) == self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_sets_delay_and_invalidates_old_ticket() {
        let state = AutoLightweightState::new();
        let ticket = AutoLightweightTicket {
            generation: state.advance(),
            current: Arc::clone(&state.generation),
        };

        state.configure(Some(3));

        assert_eq!(state.delay(), Some(Duration::from_secs(180)));
        assert!(!ticket.is_current());
        state.configure(None);
        assert_eq!(state.delay(), None);
    }
}
