//! host 侧玩家会话索引，保证同一玩家重连时不重复触发 `PlayerJoined`。

use std::collections::HashMap;

use super::*;

/// 单条会话记录。
pub(super) struct SessionEntry {
    pub(super) generation: u64,
    pub(super) conn: Option<Connection>,
}

/// Host 侧会话索引，用于同一玩家重连去重。
#[derive(Default)]
pub(super) struct HostSessions {
    by_id: HashMap<EndpointId, SessionEntry>,
}

impl HostSessions {
    pub(super) fn active_players(&self) -> usize {
        self.by_id.len()
    }

    pub(super) fn contains(&self, endpoint_id: &EndpointId) -> bool {
        self.by_id.contains_key(endpoint_id)
    }

    /// 插入或更新会话，返回 `(generation, is_reconnect, old_conn)`。
    pub(super) fn upsert(
        &mut self,
        endpoint_id: EndpointId,
        conn: Connection,
    ) -> (u64, bool, Option<Connection>) {
        match self.by_id.get_mut(&endpoint_id) {
            Some(entry) => {
                entry.generation = entry.generation.saturating_add(1);
                let generation = entry.generation;
                let old_conn = entry.conn.replace(conn);
                (generation, true, old_conn)
            }
            None => {
                self.by_id.insert(
                    endpoint_id,
                    SessionEntry {
                        generation: 1,
                        conn: Some(conn),
                    },
                );
                (1, false, None)
            }
        }
    }

    /// 仅在 `generation` 匹配当前会话时移除。
    pub(super) fn remove_if_current(&mut self, endpoint_id: &EndpointId, generation: u64) -> bool {
        let is_current = self
            .by_id
            .get(endpoint_id)
            .is_some_and(|entry| entry.generation == generation);
        if is_current {
            self.by_id.remove(endpoint_id);
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub(super) fn insert_for_test(&mut self, endpoint_id: EndpointId, generation: u64) {
        self.by_id.insert(
            endpoint_id,
            SessionEntry {
                generation,
                conn: None,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_endpoint_id() -> EndpointId {
        let bytes: [u8; 32] = rand::random();
        SecretKey::from_bytes(&bytes).public()
    }

    #[test]
    fn session_generation_guards_player_left() {
        let endpoint_id = test_endpoint_id();
        let mut sessions = HostSessions::default();
        sessions.insert_for_test(endpoint_id, 2);

        assert!(!sessions.remove_if_current(&endpoint_id, 1));
        assert_eq!(sessions.active_players(), 1);

        assert!(sessions.remove_if_current(&endpoint_id, 2));
        assert_eq!(sessions.active_players(), 0);
    }
}
