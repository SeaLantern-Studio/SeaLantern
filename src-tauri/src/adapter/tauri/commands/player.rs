/*
 * @Author: hjcba 1174368998@qq.com
 * @Date: 2026-08-20 20:10:28
 * @LastEditors: hjcba 1174368998@qq.com
 * @LastEditTime: 2026-08-22 16:00:00
 * @FilePath: \SeaLantern\src-tauri\src\adapter\tauri\commands\player.rs
 * @Description: 玩家查询 Tauri 命令。
 */
//! 玩家查询 Tauri 命令。
//!
//! 列表类数据（在线 / 白名单 / 封禁）不读服务器本地管理文件，而是像主流
//! 启动器那样：给运行中的服务器发控制台命令（`list` / `whitelist list` /
//! `banlist`），捕获回显后用名字去 `usercache.json` 反查 UUID。
//!
//! 错误处理：把 `capture_command_output` 的失败（如服务器进程已退出、
//! stdin 关闭、装配层未就绪等）原样以 `String` 透传到前端，便于排查。
//! 不再静默吞错 —— 之前的 `.unwrap_or_default()` 在玩家列表查不到时只
//! 返回空数组，无法区分"真的没玩家"与"命令根本没发出去"。
use std::sync::Arc;
use std::time::Duration;

use sealantern_application::service::capture_command_output;
use sealantern_application::services::AppServices;
use sealantern_interface::{PlayerLookupError, PlayerLookupService};

/// 单条玩家条目（含 UUID）。
#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerEntryDto {
    pub uuid: String,
    pub name: String,
}

/// 封禁条目。
#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BanEntryDto {
    pub uuid: String,
    pub name: String,
    pub reason: String,
}

/// OP 条目。
#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpEntryDto {
    pub uuid: String,
    pub name: String,
    pub level: i32,
}

/// 按用户名查询玩家档案（UUID），从服务器本地 usercache.json 读取。
#[tauri::command(rename_all = "snake_case")]
pub async fn lookup_player(
    server_path: String,
    username: String,
) -> Result<sealantern_interface::PlayerProfile, PlayerLookupError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerLookupError::ServiceUnavailable)?;
    service.lookup(server_path, username).await
}

/// 在线玩家：发 `list` 命令，捕获回显解析玩家名。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_online_players(server_id: String) -> Result<Vec<String>, String> {
    let lines = capture_command_output(&server_id, "list", Duration::from_secs(6))
        .await
        .map_err(|err| err.to_string())?;
    Ok(parse_online_names(&lines))
}

/// 白名单：发 `whitelist list`，解析名字后用 usercache 反查 UUID。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_whitelist(
    server_id: String,
    server_path: String,
) -> Result<Vec<PlayerEntryDto>, String> {
    let lines = capture_command_output(&server_id, "whitelist list", Duration::from_secs(6))
        .await
        .map_err(|err| err.to_string())?;
    let names = parse_whitelist_names(&lines);
    Ok(resolve_entries(&server_path, names).await)
}

/// 封禁列表：发 `banlist`，解析名字+原因，UUID 用 usercache 反查。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_banned_players(
    server_id: String,
    server_path: String,
) -> Result<Vec<BanEntryDto>, String> {
    let lines = capture_command_output(&server_id, "banlist", Duration::from_secs(6))
        .await
        .map_err(|err| err.to_string())?;
    let bans = parse_ban_entries(&lines);
    let svc = AppServices::player_service()
        .await
        .map_err(|err| err.to_string())?;
    let mut out = Vec::with_capacity(bans.len());
    for (name, reason) in bans {
        let uuid = lookup_uuid(&svc, &server_path, &name).await;
        out.push(BanEntryDto { uuid, name, reason });
    }
    Ok(out)
}

/// OP 列表：发 `list` 命令，解析带 `*` 前缀的在线玩家。
///
/// Minecraft Java 没有 `op list` 控制台命令，只有 `op <name>` / `deop <name>`。
/// 当前实现只能反映"在线的 OP"，离线 OP 需要读 `ops.json` 才能拿到 —— 这是
/// 现状下的折衷，与用户对"不读服务器本地文件"的诉求冲突，TODO 待与用户对齐。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_ops(
    server_id: String,
    server_path: String,
) -> Result<Vec<OpEntryDto>, String> {
    let lines = capture_command_output(&server_id, "list", Duration::from_secs(6))
        .await
        .map_err(|err| err.to_string())?;
    let names = parse_online_op_names(&lines);
    let svc = AppServices::player_service()
        .await
        .map_err(|err| err.to_string())?;
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let uuid = lookup_uuid(&svc, &server_path, &name).await;
        // Minecraft Java 控制台不输出 OP 等级，统一按默认 4 处理。
        out.push(OpEntryDto { uuid, name, level: 4 });
    }
    Ok(out)
}

/// 用 usercache.json 反查 UUID，查不到返回空串（不阻断列表展示）。
async fn lookup_uuid(
    svc: &Arc<sealantern_application::service::CorePlayerService>,
    server_path: &str,
    name: &str,
) -> String {
    match svc.lookup(server_path.to_string(), name.to_string()).await {
        Ok(profile) => profile.uuid,
        Err(_) => String::new(),
    }
}

/// 批量把玩家名解析为含 UUID 的条目。
async fn resolve_entries(server_path: &str, names: Vec<String>) -> Vec<PlayerEntryDto> {
    let svc = match AppServices::player_service().await {
        Ok(svc) => Some(svc),
        Err(_) => None,
    };
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let uuid = match &svc {
            Some(svc) => lookup_uuid(svc, server_path, &name).await,
            None => String::new(),
        };
        out.push(PlayerEntryDto { uuid, name });
    }
    out
}

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
///
/// 与 [`parse_online_names`] 共享定位逻辑；只收集带星号的条目。
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
///
/// 兼容多种版本：
/// - `There are 0 whitelisted players`（空集，旧版本）
/// - `No whitelisted players`（空集，新版本）
/// - `There are 1 whitelisted player: a`（单行，单数）
/// - `There are 2 whitelisted players: a, b`（单行，复数）
/// - `There are 2 whitelisted players:\n a\n b`（多行）
fn parse_whitelist_names(lines: &[String]) -> Vec<String> {
    // 1) 命中"空集"声明 → 直接返回。
    if lines.iter().any(|l| {
        let lower = l.to_lowercase();
        lower.contains("no whitelisted players")
            || lower.contains("no players are whitelisted")
    }) {
        return Vec::new();
    }

    // 2) 找包含 "whitelisted player" 的那一行（不限定单复数 / 冒号）。
    let Some((idx, line)) = lines.iter().enumerate().find(|(_, l)| {
        l.to_lowercase().contains("whitelisted player")
    }) else {
        return Vec::new();
    };

    let lower = line.to_lowercase();
    // 按出现位置挑最早的有效锚点（"whitelisted players:" 优先于"whitelisted player:"）。
    let anchor = if let Some(pos) = lower.find("whitelisted players:") {
        Some((pos, "whitelisted players:".len()))
    } else {
        lower.find("whitelisted player:").map(|pos| (pos, 19))
    };
    // `whitelisted players:` 也含 "whitelisted player"，上面用 `find` 拿到的是
    // 最早出现的位置（对单行单锚点场景更稳）。

    let real_after = match anchor {
        Some((pos, anchor_len)) => &line[pos + anchor_len..],
        // 不带冒号的"whitelisted player" 在原行也可能存在（极少见），跳过到下行。
        None => "",
    };

    // 2a) 单行格式：锚点后紧随名单（可能残留冒号）。
    let inline: Vec<String> = real_after
        .split(',')
        .map(|n| n.trim().trim_start_matches(':').trim().to_string())
        .filter(|n| !n.is_empty())
        .collect();
    if !inline.is_empty() {
        return inline;
    }

    // 2b) 多行格式：从锚点行往后的连续非空行都算名单（一行一个名字或逗号分隔）。
    let mut names = Vec::new();
    for next in &lines[idx + 1..] {
        let trimmed = next.trim();
        if trimmed.is_empty() {
            // 空行视为名单结束。
            break;
        }
        // 单行内的多个玩家：`alice` 或 `alice, bob`
        for n in trimmed.split(',') {
            let n = n.trim();
            if !n.is_empty() {
                names.push(n.to_string());
            }
        }
        // 继续读下一行：名单可能跨多行（每行一个或多个名字）。
    }
    names
}

/// 解析 `banlist` 回显：返回 (名字, 原因)。兼容单行与多行两种格式。
///
/// 收紧 header 匹配（必须含 `bans:` / `banned:` / `banlist` / `were banned`
/// 之一），避免 `There are no bans` 误命中后把后续 `players online` 等无关
/// 行吞成封禁条目。
fn parse_ban_entries(lines: &[String]) -> Vec<(String, String)> {
    // 0 个封禁快速返回（vanilla server 输出 `No bans` / `There are no bans` /
    // `There are 0 bans`）。放在最前面防止后续 header 误匹配。
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
        // 严格 header：必须含明确的复数 / 单数（带 ban 关键字）/ 命令名 /
        // 过去时动词。覆盖 vanilla 实际输出：
        //   `There are 2 bans: a (...), b (...)`
        //   `There is 1 ban: a (...)`
        //   `Banned players: ...`
        //   `banlist` 命令回显首行
        //   `X were banned: ...`
        let is_header = lower.contains("bans:")
            || lower.contains(" ban:")          // `There is 1 ban:`
            || lower.contains(" banned:")       // `Banned players:` / `Alice was banned:`
            || lower.contains("banlist")
            || lower.contains("were banned");
        if is_header {
            in_banlist = true;
            // 单行格式：`There are 2 bans: alice (banned by X, reason: Y), bob (...)`
            // —— 名单用 `), ` 分隔（含括号），先取首行名单。
            if let Some((_, after)) = line.split_once(':') {
                push_ban_fragments(&mut bans, after);
            }
            continue;
        }
        if in_banlist {
            // 过滤明显不是 ban 行的回显（其他命令的残留行混入 capture 通道时会被剥掉，
            // 但作为最后防线这里再过一道）。
            if is_non_ban_noise(&lower) {
                continue;
            }
            // 多行格式：每行一个 `name (banned by X, reason: Y)`
            if let Some(entry) = extract_ban_name(line) {
                bans.push(entry);
            }
        }
    }
    bans
}

/// 判断一行是否明显是其他命令的回显（或通用噪声），不应作为 ban 条目。
fn is_non_ban_noise(lower: &str) -> bool {
    lower.contains("players online")
        || lower.contains("issued server command")
        || lower.contains("whitelisted")
        || lower.contains("whitelist list")
        || lower.contains("ops:")
        || lower.contains("op list")
        || lower.contains("[server")
}

/// 把 ban list 行内串（`a (...), b (...)`）拆成若干 `(name, reason)` 并 push。
///
/// 拆分隔符用 `), `（关闭括号后跟逗号 / 空白）以避开括号内的逗号。
fn push_ban_fragments(out: &mut Vec<(String, String)>, fragment: &str) {
    let trimmed = fragment.trim();
    if trimmed.is_empty() {
        return;
    }
    // 形如 `a (b), c (d), e (f)`
    let mut current = trimmed.to_string();
    loop {
        // 关闭当前条目的方式：找到匹配的 `)` 并切到其后。
        let Some(close_rel) = current.find(')') else {
            // 没有括号，整段作为名字。
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
        // 跳过开头的逗号等。
        current = after.trim_start_matches(',').trim().to_string();
        if current.is_empty() {
            return;
        }
    }
}

/// 从一行里提取封禁名字与原因：`name (banned by X, reason: Y)` 或
/// `name was banned by X (reason: Y)` 等变体。括号是 reason 容器，
/// 括号之前（含 `was banned by` 之类连接词）按整段作为名字。
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
        let lines = vec![
            "There are 2 of a max of 20 players online: Notch, jeb".to_string(),
        ];
        assert_eq!(parse_online_names(&lines), vec!["Notch", "jeb"]);
    }

    #[test]
    fn parse_online_names_multi_line_list_minecraft_1_21() {
        // Minecraft 1.21+ 把名单换行到下行。
        let lines = vec![
            "There are 2 of a max of 20 players online:".to_string(),
            "Notch, jeb".to_string(),
        ];
        assert_eq!(parse_online_names(&lines), vec!["Notch", "jeb"]);
    }

    #[test]
    fn parse_online_names_strips_op_asterisk_prefix() {
        let lines = vec![
            "There are 2 of a max of 20 players online: * Notch, jeb".to_string(),
        ];
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
        let lines = vec![
            "There are 2 whitelisted players: alice, bob".to_string(),
        ];
        assert_eq!(parse_whitelist_names(&lines), vec!["alice", "bob"]);
    }

    #[test]
    fn parse_op_names_marks_prefixed_entries() {
        // 只有带 `*` 前缀的玩家是 OP。
        let lines = vec![
            "There are 3 of a max of 20 players online: * Notch, jeb, dinnerbone".to_string(),
        ];
        assert_eq!(parse_online_op_names(&lines), vec!["Notch"]);
    }

    #[test]
    fn parse_op_names_multi_line_with_two_ops() {
        // 1.21+ 多行格式下两个 OP。
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
        // 复现用户报告：banlist 输出 `There are no bans` 时，旧实现会因
        // `lower.contains("ban")` 命中 header 并把 in_banlist 拉成 true，
        // 接下来 `list` 命令残留的 `There are 1 ... online: hjcboar` 行被
        // extract_ban_name 当成 ban name 吞下。
        let lines = vec![
            "There are no bans".to_string(),
            "There are 1 of a max of 20 players online: hjcboar".to_string(),
        ];
        assert_eq!(parse_ban_entries(&lines), Vec::<(String, String)>::new());
    }

    #[test]
    fn parse_ban_entries_filters_noise_after_real_header() {
        // 真实 ban header 之后混入其他命令的回显，不应作为 ban 条目。
        // 同时验证单数 `There is 1 ban:` 也被识别为 header。
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
        // 单数 ban header + 同行名单。
        let lines = vec![
            "There is 1 ban: alice (banned by Admin, reason: griefing)".to_string(),
        ];
        let bans = parse_ban_entries(&lines);
        assert_eq!(bans.len(), 1);
        assert_eq!(bans[0].0, "alice");
        assert!(bans[0].1.contains("griefing"));
    }
}
