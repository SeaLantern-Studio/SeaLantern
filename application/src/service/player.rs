use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use sealantern_contract::{
    BanEntryDto, OpEntryDto, PlayerAdminError, PlayerEntryDto, PlayerListError, PlayerLookupError,
    PlayerProfile,
};
use sealantern_core::instance::InstanceId;

use crate::port::{InstanceService, PlayerAdminService, PlayerListService, PlayerLookupService};
use crate::service::{CoreInstanceService, CoreServerService, capture_command_output};

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
    /// 服务器进程管理服务：仅在线玩家列表需要它（向运行中的服务器发 `list` 命令）。
    ///
    /// 白名单 / 封禁 / OP 三个列表改为直接读取服务器目录下的配置文件，
    /// 不依赖服务器运行状态，因此不需要该服务。
    server_svc: Arc<CoreServerService>,
}

impl CorePlayerService {
    pub fn new(instance_svc: Arc<CoreInstanceService>, server_svc: Arc<CoreServerService>) -> Self {
        Self { instance_svc, server_svc }
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

    /// 读取服务器目录下的一个 JSON 列表文件并反序列化为 DTO 列表。
    ///
    /// 与「发控制台命令捕获回显」不同，读文件不要求服务器处于运行状态，
    /// 因此服务器未启动时打开玩家管理页不会产生任何错误。
    ///
    /// - 文件不存在 → 返回空列表（服务器尚未生成过该文件，属正常状态）
    /// - 内容为空或 `[]` → 返回空列表
    /// - 读取或解析失败 → [`PlayerListError::ServiceUnavailable`]
    async fn read_json_list<T>(
        &self,
        server_id: &str,
        filename: &str,
    ) -> Result<Vec<T>, PlayerListError>
    where
        T: serde::de::DeserializeOwned,
    {
        let directory = self.resolve_directory(server_id).await?;
        let path = Path::new(&directory).join(filename);

        let content = match tokio::fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(PlayerListError::ServiceUnavailable),
        };

        let trimmed = content.trim();
        if trimmed.is_empty() || trimmed == "[]" {
            return Ok(Vec::new());
        }
        serde_json::from_str(trimmed).map_err(|_| PlayerListError::ServiceUnavailable)
    }

    /// 向运行中的服务器发送一条玩家管理命令，等待其处理完成。
    ///
    /// 只负责"确认命令已送达且服务器已处理完"（以捕获回显的静默窗口判定），
    /// **不**把捕获到的回显行当作成功判定：
    ///
    /// - 并发捕获会收到同一实例的不相关日志行，仅凭"收到行"证明不了命令成功；
    /// - 命令本身也可能被服务器拒绝（玩家不存在、目标不在线等），此时服务器
    ///   仍会输出一条回显。
    ///
    /// 因此成功与否由各写方法在命令执行后对**最终状态**（配置文件 / 在线列表）
    /// 做验证，见 `assert_list_contains` 与各写方法
    /// （code review：命令被拒绝 / 并发无关行不应被误判为成功）。
    async fn run_admin_command(
        &self,
        server_id: &str,
        command: &str,
    ) -> Result<(), PlayerAdminError> {
        capture_command_output(&self.server_svc, server_id, command, Duration::from_secs(6))
            .await?;
        Ok(())
    }

    /// 断言服务器目录下的名单文件（`whitelist.json` / `banned-players.json` /
    /// `ops.json`）是否包含指定玩家（忽略大小写）。
    ///
    /// 服务器拒绝命令（玩家不存在、目标不在线等）时不会把改动写进名单文件，
    /// 据此把"命令发出但没生效"识别为 [`PlayerAdminError::OperationFailed`]；
    /// 同时也天然免疫并发场景下的无关回显行——这里的验证读的是文件而非捕获日志。
    async fn assert_list_contains(
        &self,
        server_id: &str,
        filename: &str,
        name: &str,
        contained: bool,
    ) -> Result<(), PlayerAdminError> {
        let list = self
            .read_json_list::<PlayerEntryDto>(server_id, filename)
            .await?;
        let found = list.iter().any(|e| e.name.eq_ignore_ascii_case(name));
        if found != contained {
            return Err(PlayerAdminError::OperationFailed);
        }
        Ok(())
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

impl From<crate::service::CaptureError> for PlayerAdminError {
    fn from(err: crate::service::CaptureError) -> Self {
        match err {
            crate::service::CaptureError::InvalidInput => PlayerAdminError::InvalidInput,
            crate::service::CaptureError::ServerNotRunning => PlayerAdminError::ServerNotRunning,
            crate::service::CaptureError::Unavailable => PlayerAdminError::ServiceUnavailable,
            crate::service::CaptureError::NoResponse => PlayerAdminError::CaptureFailed,
        }
    }
}

/// Minecraft 玩家名校验：3-16 字符，仅允许 ASCII 字母、数字与下划线。
///
/// 与 `server.properties` 的 `enforce-whitelist` / Mojang 账户命名规则一致；
/// 同时因为只允许 ASCII，也顺带杜绝了把命令分隔符 / 空格注入命令名的可能。
fn validate_player_name(name: &str) -> Result<&str, PlayerAdminError> {
    let name = name.trim();
    if !(3..=16).contains(&name.len()) {
        return Err(PlayerAdminError::InvalidInput);
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(PlayerAdminError::InvalidInput);
    }
    Ok(name)
}

/// 清理封禁 / 踢出原因里的多余空白（含换行符）。
///
/// 原因会作为命令参数拼进控制台命令行；若允许 `\n` / `\r` 存在，就可能把
/// 一条命令拆成多条写进服务器 stdin，因此统一把所有空白折叠为单个空格，
/// 并去掉首尾空白。
fn sanitize_reason(reason: &str) -> String {
    reason.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 把 Minecraft 配置文件里的 UUID 规范化为无连字符形式（契约约定）。
///
/// `whitelist.json` / `banned-players.json` / `ops.json` 存的是
/// 8-4-4-4-12 带连字符格式，而契约对外统一返回 32 位 hex。
fn normalize_uuid(uuid: &mut String) {
    if uuid.contains('-') {
        *uuid = uuid.replace('-', "");
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
        // 在线玩家只能从运行中的服务器获取：发 `list` 命令并解析回显。
        let lines =
            capture_command_output(&self.server_svc, &server_id, "list", Duration::from_secs(6))
                .await?;
        Ok(parse_online_names(&lines))
    }

    async fn get_whitelist(
        &self,
        server_id: String,
    ) -> Result<Vec<PlayerEntryDto>, PlayerListError> {
        // 读服务器目录下的 whitelist.json：不要求服务器运行，UUID 直接来自文件。
        let mut entries: Vec<PlayerEntryDto> =
            self.read_json_list(&server_id, "whitelist.json").await?;
        for entry in &mut entries {
            normalize_uuid(&mut entry.uuid);
        }
        Ok(entries)
    }

    async fn get_banned_players(
        &self,
        server_id: String,
    ) -> Result<Vec<BanEntryDto>, PlayerListError> {
        // 读服务器目录下的 banned-players.json：不要求服务器运行，且保留
        // reason/source/created/expires 等完整封禁信息。
        let mut entries: Vec<BanEntryDto> = self
            .read_json_list(&server_id, "banned-players.json")
            .await?;
        for entry in &mut entries {
            normalize_uuid(&mut entry.uuid);
        }
        Ok(entries)
    }

    async fn get_ops(&self, server_id: String) -> Result<Vec<OpEntryDto>, PlayerListError> {
        // 读服务器目录下的 ops.json：不要求服务器运行，且包含离线 OP 与真实等级。
        //
        // 注意：不能用 `list` 命令的 `*` 前缀代替——那只覆盖在线 OP。
        let mut entries: Vec<OpEntryDto> = self.read_json_list(&server_id, "ops.json").await?;
        for entry in &mut entries {
            normalize_uuid(&mut entry.uuid);
        }
        Ok(entries)
    }
}

#[async_trait]
impl PlayerAdminService for CorePlayerService {
    async fn add_to_whitelist(
        &self,
        server_id: String,
        name: String,
    ) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let command = format!("whitelist add {name}");
        self.run_admin_command(&server_id, &command).await?;
        // 玩家名无效时服务端不会写入白名单，据此识别命令被拒绝。
        self.assert_list_contains(&server_id, "whitelist.json", name, true)
            .await?;
        Ok(command)
    }

    async fn remove_from_whitelist(
        &self,
        server_id: String,
        name: String,
    ) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let command = format!("whitelist remove {name}");
        self.run_admin_command(&server_id, &command).await?;
        self.assert_list_contains(&server_id, "whitelist.json", name, false)
            .await?;
        Ok(command)
    }

    async fn ban_player(
        &self,
        server_id: String,
        name: String,
        reason: String,
    ) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let reason = sanitize_reason(&reason);
        let command = if reason.is_empty() {
            format!("ban {name}")
        } else {
            format!("ban {name} {reason}")
        };
        self.run_admin_command(&server_id, &command).await?;
        // 封禁不存在的玩家会被服务端拒绝，banned-players.json 不会出现该玩家。
        self.assert_list_contains(&server_id, "banned-players.json", name, true)
            .await?;
        Ok(command)
    }

    async fn unban_player(
        &self,
        server_id: String,
        name: String,
    ) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let command = format!("pardon {name}");
        self.run_admin_command(&server_id, &command).await?;
        self.assert_list_contains(&server_id, "banned-players.json", name, false)
            .await?;
        Ok(command)
    }

    async fn add_op(&self, server_id: String, name: String) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let command = format!("op {name}");
        self.run_admin_command(&server_id, &command).await?;
        self.assert_list_contains(&server_id, "ops.json", name, true)
            .await?;
        Ok(command)
    }

    async fn remove_op(&self, server_id: String, name: String) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let command = format!("deop {name}");
        self.run_admin_command(&server_id, &command).await?;
        self.assert_list_contains(&server_id, "ops.json", name, false)
            .await?;
        Ok(command)
    }

    async fn kick_player(
        &self,
        server_id: String,
        name: String,
        reason: String,
    ) -> Result<String, PlayerAdminError> {
        let name = validate_player_name(&name)?;
        let reason = sanitize_reason(&reason);
        // 服务端会拒绝"踢不在线的玩家"；kick 不写任何配置文件，只能靠在线列表
        // 判断，因此先确认目标在线，避免把"不在线"误判为踢出成功。
        let online_before = self.get_online_players(server_id.clone()).await?;
        if !online_before.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            return Err(PlayerAdminError::OperationFailed);
        }
        let command = if reason.is_empty() {
            format!("kick {name}")
        } else {
            format!("kick {name} {reason}")
        };
        self.run_admin_command(&server_id, &command).await?;
        // 执行后确认其已下线。
        let online_after = self.get_online_players(server_id).await?;
        if online_after.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            return Err(PlayerAdminError::OperationFailed);
        }
        Ok(command)
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── DTO 反序列化：直接对应 Minecraft 配置文件格式 ──────────────

    #[test]
    fn deserialize_whitelist_entry() {
        let raw = r#"[{"uuid":"069a79f4-44e9-4726-a5be-fca90e38aaf5","name":"Notch"}]"#;
        let entries: Vec<PlayerEntryDto> = serde_json::from_str(raw).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Notch");
        assert_eq!(entries[0].uuid, "069a79f4-44e9-4726-a5be-fca90e38aaf5");
    }

    #[test]
    fn deserialize_ban_entry_keeps_full_fields() {
        let raw = r#"[{
            "uuid":"069a79f4-44e9-4726-a5be-fca90e38aaf5",
            "name":"griefer",
            "created":"2026-01-01 00:00:00 +0800",
            "source":"Server",
            "expires":"forever",
            "reason":"griefing"
        }]"#;
        let entries: Vec<BanEntryDto> = serde_json::from_str(raw).unwrap();
        assert_eq!(entries[0].name, "griefer");
        assert_eq!(entries[0].reason, "griefing");
        assert_eq!(entries[0].source, "Server");
        assert_eq!(entries[0].created, "2026-01-01 00:00:00 +0800");
        assert_eq!(entries[0].expires, "forever");
    }

    #[test]
    fn deserialize_op_entry_accepts_camel_case_bypass_field() {
        let raw = r#"[{
            "uuid":"069a79f4-44e9-4726-a5be-fca90e38aaf5",
            "name":"Notch",
            "level":4,
            "bypassesPlayerLimit":true
        }]"#;
        let entries: Vec<OpEntryDto> = serde_json::from_str(raw).unwrap();
        assert_eq!(entries[0].name, "Notch");
        assert_eq!(entries[0].level, 4);
        assert!(entries[0].bypasses_player_limit);
    }

    #[test]
    fn deserialize_entry_tolerates_missing_optional_fields() {
        // 老版本服务器可能不写 source/created/expires，缺失时取默认值。
        let raw = r#"[{"uuid":"069a79f4-44e9-4726-a5be-fca90e38aaf5","name":"Notch"}]"#;
        let entries: Vec<BanEntryDto> = serde_json::from_str(raw).unwrap();
        assert_eq!(entries[0].reason, "");
        assert_eq!(entries[0].created, "");
        assert_eq!(entries[0].expires, "");
    }

    #[test]
    fn normalize_uuid_strips_hyphens_only_when_present() {
        let mut dashed = "069a79f4-44e9-4726-a5be-fca90e38aaf5".to_string();
        normalize_uuid(&mut dashed);
        assert_eq!(dashed, "069a79f444e94726a5befca90e38aaf5");

        let mut plain = "069a79f444e94726a5befca90e38aaf5".to_string();
        normalize_uuid(&mut plain);
        assert_eq!(plain, "069a79f444e94726a5befca90e38aaf5");
    }

    // ── 在线玩家：`list` 回显解析 ────────────────────────────────

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

    // ── 写入操作的玩家名校验 ──────────────────────────────────

    #[test]
    fn validate_player_name_accepts_valid_names() {
        assert_eq!(validate_player_name("Notch").unwrap(), "Notch");
        // 前后空白会被裁剪。
        assert_eq!(validate_player_name("  jeb_  ").unwrap(), "jeb_");
        // 长度边界：3 与 16。
        assert_eq!(validate_player_name("abc").unwrap(), "abc");
        assert_eq!(validate_player_name("a123456789012345").unwrap(), "a123456789012345");
    }

    #[test]
    fn validate_player_name_rejects_invalid_names() {
        // 空与超短。
        assert!(validate_player_name("").is_err());
        assert!(validate_player_name("ab").is_err());
        // 超长（17 字符）。
        assert!(validate_player_name("a1234567890123456").is_err());
        // 非法字符：空格、连字符、命令分隔符、非 ASCII。
        assert!(validate_player_name("has space").is_err());
        assert!(validate_player_name("bad-name").is_err());
        assert!(validate_player_name("inject;stop").is_err());
        assert!(validate_player_name("中文名").is_err());
    }

    #[test]
    fn sanitize_reason_collapses_newlines_into_spaces() {
        assert_eq!(sanitize_reason("griefing"), "griefing");
        assert_eq!(sanitize_reason("  spam  "), "spam");
        // 换行不能穿透成第二条控制台命令。
        assert_eq!(sanitize_reason("griefing\nstop"), "griefing stop");
        assert_eq!(sanitize_reason("a\r\nb"), "a b");
        assert_eq!(sanitize_reason(""), "");
    }
}
