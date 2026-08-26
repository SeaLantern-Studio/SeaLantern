use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use sealantern_core::instance::InstanceId;
use sealantern_interface::{
    BanEntryDto, InstanceService, OpEntryDto, PlayerEntryDto, PlayerListError, PlayerListService,
    PlayerLookupError, PlayerLookupService, PlayerProfile,
};
use std::path::Path;

use crate::service::capture_command_output;
use crate::service::CoreInstanceService;

/// usercache.json 里每条记录的格式。
#[derive(serde::Deserialize)]
struct UserCacheEntry {
    name: String,
    uuid: String,
}

pub struct CorePlayerService {
    /// 实例注册表：用于由 server_id 解析出唯一可信的服务器目录，
    /// 而非信任前端传入的 server_path（见 code review：server_id 与
    /// server_path 分开信任）。
    instance_svc: Arc<CoreInstanceService>,
}

impl CorePlayerService {
    pub fn new(instance_svc: Arc<CoreInstanceService>) -> Self {
        Self { instance_svc }
    }

    /// 由 server_id 经实例注册表解析出唯一可信的服务器目录。
    ///
    /// 这是玩家子系统唯一允许获取目录的入口：不接收前端传入的 server_path，
    /// 避免 A 服的 list 回显与 B 服 usercache 拼出错误 UUID。
    async fn resolve_directory(&self, server_id: &str) -> Result<String, PlayerListError> {
        let id = InstanceId::new(server_id).map_err(|_| PlayerListError::InvalidInput)?;
        let instance = self
            .instance_svc
            .find(&id)
            .await
            .map_err(|_| PlayerListError::ServiceUnavailable)?
            .ok_or(PlayerListError::ServiceUnavailable)?;
        Ok(instance.directory.to_string_lossy().into_owned())
    }
}

impl From<crate::service::CaptureError> for PlayerListError {
    fn from(err: crate::service::CaptureError) -> Self {
        match err {
            crate::service::CaptureError::InvalidInput => PlayerListError::InvalidInput,
            crate::service::CaptureError::ServerNotRunning => PlayerListError::ServerNotRunning,
            crate::service::CaptureError::Unavailable => PlayerListError::ServiceUnavailable,
            crate::service::CaptureError::NoResponse => PlayerListError::CaptureFailed,
        }
    }
}

#[async_trait]
impl PlayerLookupService for CorePlayerService {
    async fn lookup(
        &self,
        server_path: String,
        username: String,
    ) -> Result<PlayerProfile, PlayerLookupError> {
        // 1. 校验用户名：不能空，只能字母数字下划线
        let username = username.trim();
        if username.is_empty() || !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PlayerLookupError::InvalidInput);
        }

        // 2. 校验服务器路径
        if server_path.trim().is_empty() {
            return Err(PlayerLookupError::ServerNotSelected);
        }

        // 3. 读 usercache.json
        let cache_path = Path::new(&server_path).join("usercache.json");
        let content = tokio::fs::read_to_string(&cache_path)
            .await
            .map_err(|_| PlayerLookupError::ServiceUnavailable)?;

        // 4. 解析 JSON 数组，按用户名查找（不区分大小写）
        let entries: Vec<UserCacheEntry> =
            serde_json::from_str(&content).map_err(|_| PlayerLookupError::ServiceUnavailable)?;

        let found = entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(username));

        match found {
            Some(entry) => {
                // usercache.json 的 UUID 是 8-4-4-4-12 带连字符格式
                // 去掉连字符，保持无连字符形式
                let uuid = entry.uuid.replace('-', "");
                Ok(PlayerProfile { name: entry.name.clone(), uuid })
            }
            None => Err(PlayerLookupError::NotFound),
        }
    }
}

#[async_trait]
impl PlayerListService for CorePlayerService {
    async fn get_online_players(&self, server_id: String) -> Result<Vec<String>, PlayerListError> {
        let lines = capture_command_output(&server_id, "list", Duration::from_secs(6)).await?;
        Ok(parse_online_names(&lines))
    }

    async fn get_whitelist(
        &self,
        server_id: String,
    ) -> Result<Vec<PlayerEntryDto>, PlayerListError> {
        let lines =
            capture_command_output(&server_id, "whitelist list", Duration::from_secs(6)).await?;
        let server_path = self.resolve_directory(&server_id).await?;
        let names = parse_whitelist_names(&lines);
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            let uuid = self
                .lookup(server_path.clone(), name.clone())
                .await
                .map(|p| p.uuid)
                .unwrap_or_default();
            out.push(PlayerEntryDto { uuid, name });
        }
        Ok(out)
    }

    async fn get_banned_players(
        &self,
        server_id: String,
    ) -> Result<Vec<BanEntryDto>, PlayerListError> {
        let lines = capture_command_output(&server_id, "banlist", Duration::from_secs(6)).await?;
        let server_path = self.resolve_directory(&server_id).await?;
        let bans = parse_ban_entries(&lines);
        let mut out = Vec::with_capacity(bans.len());
        for (name, reason) in bans {
            let uuid = self
                .lookup(server_path.clone(), name.clone())
                .await
                .map(|p| p.uuid)
                .unwrap_or_default();
            out.push(BanEntryDto { uuid, name, reason });
        }
        Ok(out)
    }

    async fn get_ops(
        &self,
        server_id: String,
    ) -> Result<Vec<OpEntryDto>, PlayerListError> {
        let lines = capture_command_output(&server_id, "list", Duration::from_secs(6)).await?;
        let server_path = self.resolve_directory(&server_id).await?;
        let names = parse_online_op_names(&lines);
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            let uuid = self
                .lookup(server_path.clone(), name.clone())
                .await
                .map(|p| p.uuid)
                .unwrap_or_default();
            // Minecraft Java 控制台不输出 OP 等级，统一按默认 4 处理。
            out.push(OpEntryDto { uuid, name, level: 4 });
        }
        Ok(out)
    }
}

// ── 控制台回显解析函数 ─────────────────────────────────────────

/// 解析 `list` 回显里的玩家名。
///
/// 兼容三种格式：
/// 1. `There are 0 of a max of 20 players online` —— 0 人，无名单
/// 2. `There are 1 of a max of 20 players online: a` —— 单行，冒号后逗号分隔
/// 3. `There are 1 of a max of 20 players online:\n a` —— 名单换行到下一行（Minecraft 1.21+）
fn parse_online_names(lines: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    let Some((idx, line)) = lines
        .iter()
        .enumerate()
        .find(|(_, l)| l.to_lowercase().contains("players online"))
    else {
        return names;
    };

    let lower = line.to_lowercase();
    let after = match lower.find("players online") {
        Some(pos) => &line[pos + "players online".len()..],
        None => return names,
    };

    // 单行格式：`... online: a, b, c`
    if let Some((_, list_part)) = after.split_once(':') {
        let inline: Vec<String> = list_part
            .split(',')
            .map(|n| n.trim().trim_start_matches('*').trim().to_string())
            .filter(|n| !n.is_empty())
            .collect();
        if !inline.is_empty() {
            return inline;
        }
    }

    // 多行格式：名单出现在下一行。
    for next in &lines[idx + 1..] {
        let trimmed = next.trim();
        if trimmed.is_empty() {
            continue;
        }
        for n in trimmed.split(',') {
            let n = n.trim().trim_start_matches('*').trim();
            if !n.is_empty() {
                names.push(n.to_string());
            }
        }
        break;
    }
    names
}

/// 解析 `list` 回显里的在线 OP 玩家（带 `*` 前缀的名字）。
fn parse_online_op_names(lines: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    let Some((idx, line)) = lines
        .iter()
        .enumerate()
        .find(|(_, l)| l.to_lowercase().contains("players online"))
    else {
        return names;
    };
    let lower = line.to_lowercase();
    let after = match lower.find("players online") {
        Some(pos) => &line[pos + "players online".len()..],
        None => return names,
    };

    let mut all = Vec::new();
    if let Some((_, list_part)) = after.split_once(':') {
        let inline: Vec<String> = list_part
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        all.extend(inline);
    }
    for next in &lines[idx + 1..] {
        let trimmed = next.trim();
        if trimmed.is_empty() {
            continue;
        }
        all.extend(
            trimmed
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
        break;
    }
    for n in all {
        // 仅收集以 `*` 开头的 OP 玩家，去掉 `*` 与空白后作为名字。
        if let Some(stripped) = n.strip_prefix('*') {
            let name = stripped.trim();
            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// 解析 `whitelist list` 回显里的玩家名。
fn parse_whitelist_names(lines: &[String]) -> Vec<String> {
    // 1) 命中"空集"声明 → 直接返回。
    if lines.iter().any(|l| {
        let lower = l.to_lowercase();
        lower.contains("no whitelisted players") || lower.contains("no players are whitelisted")
    }) {
        return Vec::new();
    }

    // 2) 找包含 "whitelisted player" 的那一行。
    let Some((idx, line)) = lines
        .iter()
        .enumerate()
        .find(|(_, l)| l.to_lowercase().contains("whitelisted player"))
    else {
        return Vec::new();
    };

    let lower = line.to_lowercase();
    let anchor = if let Some(pos) = lower.find("whitelisted players:") {
        Some((pos, "whitelisted players:".len()))
    } else {
        lower.find("whitelisted player:").map(|pos| (pos, 19))
    };

    let real_after = match anchor {
        Some((pos, anchor_len)) => &line[pos + anchor_len..],
        None => "",
    };

    // 2a) 单行格式：锚点后紧随名单。
    let inline: Vec<String> = real_after
        .split(',')
        .map(|n| n.trim().trim_start_matches(':').trim().to_string())
        .filter(|n| !n.is_empty())
        .collect();
    if !inline.is_empty() {
        return inline;
    }

    // 2b) 多行格式。
    let mut names = Vec::new();
    for next in &lines[idx + 1..] {
        let trimmed = next.trim();
        if trimmed.is_empty() {
            break;
        }
        for n in trimmed.split(',') {
            let n = n.trim();
            if !n.is_empty() {
                names.push(n.to_string());
            }
        }
    }
    names
}

/// 解析 `banlist` 回显：返回 (名字, 原因)。
fn parse_ban_entries(lines: &[String]) -> Vec<(String, String)> {
    // 0 个封禁快速返回。
    if lines.iter().any(|l| {
        let lower = l.to_lowercase();
        lower.contains("no bans")
            || lower.contains("0 bans")
            || lower.contains("no players are banned")
            || lower.contains("no banned players")
    }) {
        return Vec::new();
    }

    let mut bans: Vec<(String, String)> = Vec::new();
    let mut in_banlist = false;
    for line in lines {
        let lower = line.to_lowercase();
        let is_header = lower.contains("bans:")
            || lower.contains(" ban:")
            || lower.contains(" banned:")
            || lower.contains("banlist")
            || lower.contains("were banned");
        if is_header {
            in_banlist = true;
            if let Some((_, after)) = line.split_once(':') {
                push_ban_fragments(&mut bans, after);
            }
            continue;
        }
        if in_banlist {
            if is_non_ban_noise(&lower) {
                continue;
            }
            if let Some(entry) = extract_ban_name(line) {
                bans.push(entry);
            }
        }
    }
    bans
}

/// 判断一行是否明显是其他命令的回显。
fn is_non_ban_noise(lower: &str) -> bool {
    lower.contains("players online")
        || lower.contains("issued server command")
        || lower.contains("whitelisted")
        || lower.contains("whitelist list")
        || lower.contains("ops:")
        || lower.contains("op list")
        || lower.contains("[server")
}

/// 把 ban list 行内串拆成若干 `(name, reason)` 并 push。
fn push_ban_fragments(out: &mut Vec<(String, String)>, fragment: &str) {
    let trimmed = fragment.trim();
    if trimmed.is_empty() {
        return;
    }
    let mut current = trimmed.to_string();
    loop {
        let Some(close_rel) = current.find(')') else {
            let name = current.trim().trim_end_matches(',').trim().to_string();
            if !name.is_empty() {
                out.push((name, String::new()));
            }
            return;
        };
        let close = close_rel;
        let entry_part = &current[..=close];
        if let Some((name, reason)) = extract_ban_name(entry_part) {
            out.push((name, reason));
        }
        let after = current[close + 1..].trim();
        if after.is_empty() {
            return;
        }
        current = after.trim_start_matches(',').trim().to_string();
        if current.is_empty() {
            return;
        }
    }
}

/// 从一行里提取封禁名字与原因。
fn extract_ban_name(part: &str) -> Option<(String, String)> {
    let part = part.trim().trim_end_matches(',').trim();
    if part.is_empty() {
        return None;
    }
    if let Some(open) = part.find('(') {
        let head = part[..open].trim().trim_end_matches(',').trim().to_string();
        let reason = part[open + 1..]
            .trim()
            .trim_end_matches('.')
            .trim_end_matches(')')
            .trim_end_matches('.')
            .to_string();
        let name = head
            .split_whitespace()
            .take_while(|w| *w != "was")
            .collect::<Vec<&str>>()
            .join(" ");
        let name = if name.is_empty() { head } else { name };
        Some((name, reason))
    } else {
        Some((part.to_string(), String::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_online_names_legacy_zero_players_single_line() {
        let lines = vec!["There are 0 of a max of 20 players online".to_string()];
        assert_eq!(parse_online_names(&lines), Vec::<String>::new());
    }

    #[test]
    fn parse_online_names_single_line_inline_list() {
        let lines = vec!["There are 2 of a max of 20 players online: Notch, jeb".to_string()];
        assert_eq!(parse_online_names(&lines), vec!["Notch", "jeb"]);
    }

    #[test]
    fn parse_online_names_multi_line_list_minecraft_1_21() {
        let lines = vec![
            "There are 2 of a max of 20 players online:".to_string(),
            "Notch, jeb".to_string(),
        ];
        assert_eq!(parse_online_names(&lines), vec!["Notch", "jeb"]);
    }

    #[test]
    fn parse_online_names_strips_op_asterisk_prefix() {
        let lines = vec!["There are 2 of a max of 20 players online: * Notch, jeb".to_string()];
        assert_eq!(parse_online_names(&lines), vec!["Notch", "jeb"]);
    }

    #[test]
    fn parse_online_names_returns_empty_when_no_matching_line() {
        let lines = vec!["[Server] something unrelated".to_string()];
        assert_eq!(parse_online_names(&lines), Vec::<String>::new());
    }

    #[test]
    fn parse_whitelist_names_empty_declaration() {
        let lines = vec!["There are 0 whitelisted players".to_string()];
        assert_eq!(parse_whitelist_names(&lines), Vec::<String>::new());
    }

    #[test]
    fn parse_whitelist_names_multi_line_list() {
        let lines = vec![
            "There are 2 whitelisted players:".to_string(),
            "alice".to_string(),
            "bob".to_string(),
        ];
        assert_eq!(parse_whitelist_names(&lines), vec!["alice", "bob"]);
    }

    #[test]
    fn parse_whitelist_names_inline_list() {
        let lines = vec!["There are 2 whitelisted players: alice, bob".to_string()];
        assert_eq!(parse_whitelist_names(&lines), vec!["alice", "bob"]);
    }

    #[test]
    fn parse_op_names_marks_prefixed_entries() {
        let lines =
            vec!["There are 3 of a max of 20 players online: * Notch, jeb, dinnerbone".to_string()];
        assert_eq!(parse_online_op_names(&lines), vec!["Notch"]);
    }

    #[test]
    fn parse_op_names_multi_line_with_two_ops() {
        let lines = vec![
            "There are 3 of a max of 20 players online:".to_string(),
            "* Notch, * jeb, dinnerbone".to_string(),
        ];
        assert_eq!(parse_online_op_names(&lines), vec!["Notch", "jeb"]);
    }

    #[test]
    fn parse_ban_entries_inline_with_reasons() {
        let lines = vec![r#"There are 2 bans: alice (banned by Admin, reason: griefing), bob (banned by Mod, reason: spam)"#.to_string()];
        let bans = parse_ban_entries(&lines);
        assert_eq!(bans.len(), 2);
        assert_eq!(bans[0].0, "alice");
        assert_eq!(bans[1].0, "bob");
        assert!(bans[0].1.contains("griefing"));
    }

    #[test]
    fn parse_ban_entries_multi_line() {
        let lines = vec![
            "There are 2 bans:".to_string(),
            "alice was banned by Admin (reason: griefing)".to_string(),
            "bob was banned by Mod (reason: spam)".to_string(),
        ];
        let bans = parse_ban_entries(&lines);
        assert_eq!(bans.len(), 2);
        assert_eq!(bans[0].0, "alice");
        assert_eq!(bans[1].0, "bob");
    }

    #[test]
    fn parse_ban_entries_empty_does_not_swallow_unrelated_lines() {
        let lines = vec![
            "There are no bans".to_string(),
            "There are 1 of a max of 20 players online: hjcboar".to_string(),
        ];
        assert_eq!(parse_ban_entries(&lines), Vec::<(String, String)>::new());
    }

    #[test]
    fn parse_ban_entries_filters_noise_after_real_header() {
        let lines = vec![
            "There is 1 ban:".to_string(),
            "There are 1 of a max of 20 players online: hjcboar".to_string(),
            "alice was banned by Admin (reason: griefing)".to_string(),
            "There are 3 whitelisted players: bob".to_string(),
        ];
        let bans = parse_ban_entries(&lines);
        assert_eq!(bans.len(), 1);
        assert_eq!(bans[0].0, "alice");
        assert!(bans[0].1.contains("griefing"));
    }

    #[test]
    fn parse_ban_entries_singular_header_inline() {
        let lines = vec!["There is 1 ban: alice (banned by Admin, reason: griefing)".to_string()];
        let bans = parse_ban_entries(&lines);
        assert_eq!(bans.len(), 1);
        assert_eq!(bans[0].0, "alice");
        assert!(bans[0].1.contains("griefing"));
    }
}
