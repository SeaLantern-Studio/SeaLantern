//! 玩家实时位置服务实现。
//!
//! 走 RCON：直接问运行中的服务器「玩家在哪」，不依赖任何服务端插件，
//! 也不用去扒 `playerdata`（那份数据只在存档时落盘，玩家跑动时根本不更新）。
//!
//! 一次查询只发两条命令：坐标一条、朝向一条，都是对「该维度全部玩家」生效，
//! 所以玩家数量增加不会线性拖慢刷新。
//!
//! 刻意不复用 TCP 连接：RCON 握手在回环上只有几毫秒，而复用连接要处理
//! 服务器重启后句柄失效、并发请求争抢同一 socket 这些麻烦事。按秒级轮询的
//! 频率算，每次新建连接的代价完全可以忽略。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use sealantern_contract::server::ServerState;
use sealantern_contract::{Dimension, PlayerPosition, PlayerPositionServiceError, PlayerPositions};
use sealantern_core::instance::InstanceId;
use sealantern_feature::config::ServerPropertiesManager;
use sealantern_feature::rcon::{
    RconClient, RconError, build_positions_command, build_rotation_command, is_no_target_response,
    parse_positions, parse_rotations,
};

use crate::port::{InstanceService, PlayerPositionService, ServerService};
use crate::service::{CoreInstanceService, CoreServerService};

/// 建立 TCP 连接并完成认证的上限。
///
/// 服务器没在运行时连回环端口会被立刻拒绝，这个超时主要是防防火墙吞包。
const CONNECT_TIMEOUT: Duration = Duration::from_millis(1_500);

/// 读取 `server.properties` 得到的 RCON 连接参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RconSettings {
    pub(crate) port: u16,
    pub(crate) password: String,
}

/// 让服务器把内存里的区块刷进 `.mca`。
///
/// 地图只读磁盘，而服务端把改过的区块攒在内存里等自动保存。不主动落盘的话，
/// 玩家刚走过、刚放过方块的地方在地图上还是旧画面——这是「我明明玩过了，
/// 地图上却没有」最直接的成因。
///
/// 服务器没运行、没开 RCON、密码不对，都按失败返回，由调用方决定要不要忽略。
pub(crate) async fn flush_world(server_dir: PathBuf) -> Result<(), PlayerPositionServiceError> {
    let settings = tokio::task::spawn_blocking(move || read_rcon_settings(&server_dir))
        .await
        .map_err(|_| PlayerPositionServiceError::OperationFailed)??;

    let addr = format!("127.0.0.1:{}", settings.port);
    let mut client = RconClient::connect(&addr, &settings.password, CONNECT_TIMEOUT)
        .await
        .map_err(|error| to_service_error(&error))?;
    client
        .command("save-all flush")
        .await
        .map_err(|error| to_service_error(&error))?;
    client.close().await;
    Ok(())
}

/// 玩家实时位置服务。
pub struct CorePlayerPositionService {
    instance: Arc<CoreInstanceService>,
    /// 只用来判断「占着同一个 RCON 端口的那个实例是不是正在运行」。
    server: Arc<CoreServerService>,
}

impl CorePlayerPositionService {
    /// 用实例服务与服务器进程服务句柄构造。
    pub fn new(instance: Arc<CoreInstanceService>, server: Arc<CoreServerService>) -> Self {
        Self { instance, server }
    }
}

/// 同一个 RCON 端口是不是被**另一个正在运行的实例**占着。
///
/// 端口是各实例自己在 `server.properties` 里配的，默认都是 25575。多开时
/// 谁先起来谁占着端口，此时向另一个实例发查询，连上的是别人家的服务器 ——
/// 密码一样的话，拿到的玩家坐标就是别的服的，`save-all` 刷的也是别人的世界。
///
/// 判据必须是「对方正在运行」：两个实例配了同一个端口、但对方停着，那是
/// 完全正常的场景（复制服务器目录时最容易出现），端口空着，我们的服务器
/// 才占得上，没理由报错。
async fn port_owner(
    instance: &CoreInstanceService,
    server: &CoreServerService,
    own: &InstanceId,
    port: u16,
) -> Option<String> {
    let instances = match instance.list().await {
        Ok(instances) => instances,
        // 列不出实例就不做判断：宁可照常查询，也不要因为读一次注册表失败
        // 就让整个地图的玩家图层挂掉
        Err(error) => {
            tracing::debug!(
                target: "sealantern.application.player_position",
                error = %error,
                "读取实例列表失败，跳过 RCON 端口冲突检查"
            );
            return None;
        }
    };

    // 先把这个实例的所有候选收集齐，再统一判断 —— 判定逻辑抽成纯函数，
    // 才能不装配真实进程就测到「同端口但对方停着不算冲突」这条
    let mut candidates: Vec<(String, u16, bool)> = Vec::new();
    for other in instances {
        if other.id == *own {
            continue;
        }
        let directory = other.directory.clone();
        let settings = tokio::task::spawn_blocking(move || read_rcon_settings(&directory))
            .await
            .ok()
            .and_then(Result::ok);
        let Some(settings) = settings else {
            continue;
        };
        let running = server
            .status(&other.id)
            .await
            .map(|snapshot| {
                matches!(
                    snapshot.state,
                    ServerState::Starting | ServerState::Running | ServerState::Stopping
                )
            })
            .unwrap_or(false);
        candidates.push((other.name, settings.port, running));
    }

    conflicting_owner(port, candidates).map(|(name, _)| name)
}

/// 从「候选实例: (名字, 端口, 是否运行中)」里挑出真正会抢走查询的那个。
///
/// 只认「端口相同且正在运行」。对方停着的时候端口是空的，我们自己的服务器
/// 才能绑上去，这时候报冲突纯属添堵。
fn conflicting_owner(port: u16, candidates: Vec<(String, u16, bool)>) -> Option<(String, u16)> {
    candidates
        .into_iter()
        .find(|(_, candidate_port, running)| *candidate_port == port && *running)
        .map(|(name, candidate_port, _)| (name, candidate_port))
}

#[async_trait]
impl PlayerPositionService for CorePlayerPositionService {
    async fn positions(
        &self,
        instance_id: &str,
        dimension: Dimension,
    ) -> Result<PlayerPositions, PlayerPositionServiceError> {
        let id = InstanceId::new(instance_id)
            .map_err(|_| PlayerPositionServiceError::InstanceNotFound)?;
        let instance = self
            .instance
            .find(&id)
            .await
            .map_err(|error| {
                tracing::warn!(
                    target: "sealantern.application.player_position",
                    instance_id,
                    error = %error,
                    "读取实例记录失败"
                );
                PlayerPositionServiceError::OperationFailed
            })?
            .ok_or(PlayerPositionServiceError::InstanceNotFound)?;

        let directory = instance.directory.clone();
        let settings = tokio::task::spawn_blocking(move || read_rcon_settings(&directory))
            .await
            .map_err(|_| PlayerPositionServiceError::OperationFailed)??;

        // 端口冲突时连过去看到的是别人的服，宁可报错也不返回错的数据
        if let Some(owner) = port_owner(&self.instance, &self.server, &id, settings.port).await {
            tracing::error!(
                target: "sealantern.application.player_position",
                instance_id,
                port = settings.port,
                owner = %owner,
                "RCON 端口被另一个正在运行的实例占用，拒绝查询"
            );
            return Err(PlayerPositionServiceError::RconPortConflict);
        }

        let addr = format!("127.0.0.1:{}", settings.port);
        let mut client = RconClient::connect(&addr, &settings.password, CONNECT_TIMEOUT)
            .await
            .map_err(|error| {
                log_rcon_failure(instance_id, "连接 RCON 失败", &error);
                to_service_error(&error)
            })?;

        let positions_lines = client
            .command(&build_positions_command(dimension))
            .await
            .map_err(|error| {
                log_rcon_failure(instance_id, "查询玩家坐标失败", &error);
                to_service_error(&error)
            })?;
        let positions = parse_positions(&positions_lines);

        // 命令有回显却一条都解析不出来，说明服务端输出格式和预期不一致，
        // 不能当成「没人在线」悄悄返回空列表。但「选择器没匹配到目标」是个例外：
        // 玩家都在主世界时查下界，服务端回的就是 `No player was found`，
        // 这是空列表，不是格式错误
        if positions.is_empty()
            && !positions_lines.is_empty()
            && !is_no_target_response(&positions_lines)
        {
            tracing::warn!(
                target: "sealantern.application.player_position",
                instance_id,
                sample = positions_lines.first().map(String::as_str).unwrap_or(""),
                "玩家坐标回显无法解析"
            );
            client.close().await;
            return Err(PlayerPositionServiceError::MalformedResponse);
        }

        // 没人就不用再问一次朝向
        let rotations = if positions.is_empty() {
            Vec::new()
        } else {
            let lines = client
                .command(&build_rotation_command(dimension))
                .await
                .map_err(|error| {
                    log_rcon_failure(instance_id, "查询玩家朝向失败", &error);
                    to_service_error(&error)
                })?;
            parse_rotations(&lines)
        };
        client.close().await;

        let players = positions
            .into_iter()
            .map(|(name, [x, y, z])| {
                // 朝向拿不到不算错误：没有箭头也好过整张图报错
                let yaw = rotations
                    .iter()
                    .find(|(candidate, _)| *candidate == name)
                    .map(|(_, yaw)| *yaw)
                    .unwrap_or(0.0);
                PlayerPosition { name, x, y, z, yaw, dimension }
            })
            .collect();

        Ok(PlayerPositions {
            instance_id: instance_id.to_string(),
            dimension,
            captured_at_unix_secs: unix_secs(),
            players,
        })
    }
}

/// 读取 `server.properties` 里的 RCON 配置。
///
/// `enable-rcon` 不是 `true`、或者密码为空，都算「没配」。这是刻意的：
/// RCON 端口开着但密码为空时，服务端本身会拒绝认证，报 `RconDisabled`
/// 比报认证失败更容易让用户明白该去改哪一行。
pub(crate) fn read_rcon_settings(
    server_dir: &Path,
) -> Result<RconSettings, PlayerPositionServiceError> {
    let properties = ServerPropertiesManager::new(server_dir)
        .read()
        .map_err(|_| PlayerPositionServiceError::RconDisabled)?;

    let enabled = properties
        .raw
        .get("enable-rcon")
        .map(|value| value.trim().eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !enabled {
        return Err(PlayerPositionServiceError::RconDisabled);
    }

    let password = properties
        .raw
        .get("rcon.password")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or(PlayerPositionServiceError::RconDisabled)?;

    let port = properties
        .raw
        .get("rcon.port")
        .and_then(|value| value.trim().parse::<u16>().ok())
        .unwrap_or(25575);

    Ok(RconSettings { port, password })
}

/// RCON 错误收敛成契约错误。
fn to_service_error(error: &RconError) -> PlayerPositionServiceError {
    match error {
        RconError::Unauthorized => PlayerPositionServiceError::RconUnauthorized,
        RconError::Protocol(_) => PlayerPositionServiceError::MalformedResponse,
        // 连不上、超时、连接中途断开：本质上都是「问不到活着的服务器」
        RconError::Connect(_) | RconError::Io(_) | RconError::Timeout => {
            PlayerPositionServiceError::RconUnreachable
        }
    }
}

fn log_rcon_failure(instance_id: &str, what: &str, error: &RconError) {
    tracing::warn!(
        target: "sealantern.application.player_position",
        instance_id,
        error = %error,
        "{what}"
    );
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_properties(content: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::write(dir.path().join("server.properties"), content).expect("写入配置");
        dir
    }

    #[test]
    fn reads_rcon_settings() {
        let dir = write_properties("enable-rcon=true\nrcon.port=25575\nrcon.password=s3cret\n");
        let settings = read_rcon_settings(dir.path()).expect("应读到配置");
        assert_eq!(
            settings,
            RconSettings {
                port: 25575,
                password: "s3cret".to_string()
            }
        );
    }

    #[test]
    fn disabled_rcon_is_rejected() {
        let dir = write_properties("enable-rcon=false\nrcon.password=s3cret\n");
        assert_eq!(read_rcon_settings(dir.path()), Err(PlayerPositionServiceError::RconDisabled));
    }

    #[test]
    fn empty_password_is_rejected() {
        let dir = write_properties("enable-rcon=true\nrcon.password=\n");
        assert_eq!(read_rcon_settings(dir.path()), Err(PlayerPositionServiceError::RconDisabled));
    }

    #[test]
    fn missing_properties_file_is_rejected() {
        let dir = tempfile::tempdir().expect("临时目录");
        assert_eq!(read_rcon_settings(dir.path()), Err(PlayerPositionServiceError::RconDisabled));
    }

    #[test]
    fn port_falls_back_to_default() {
        let dir = write_properties("enable-rcon=true\nrcon.password=p\nrcon.port=abc\n");
        let settings = read_rcon_settings(dir.path()).expect("应读到配置");
        assert_eq!(settings.port, 25575);
    }

    #[test]
    fn enable_flag_is_case_insensitive_and_trimmed() {
        let dir = write_properties("enable-rcon= TRUE \nrcon.password=p\nrcon.port=25566\n");
        let settings = read_rcon_settings(dir.path()).expect("应读到配置");
        assert_eq!(settings.port, 25566);
    }

    #[tokio::test]
    async fn flush_world_needs_rcon_config() {
        let dir = tempfile::tempdir().expect("临时目录");
        assert_eq!(
            flush_world(dir.path().to_path_buf()).await,
            Err(PlayerPositionServiceError::RconDisabled)
        );
    }

    /// 端口撞了要看对方在不在跑：在跑就会连到别人家，停着只是配置重复。
    #[test]
    fn conflicting_owner_only_counts_running_instances() {
        let candidates = vec![
            ("停着的服".to_string(), 25575u16, false),
            ("在跑的服".to_string(), 25575u16, true),
            ("端口不同的服".to_string(), 25576u16, true),
        ];

        assert_eq!(
            conflicting_owner(25575, candidates.clone()),
            Some(("在跑的服".to_string(), 25575))
        );
        // 所有人都停着 → 端口是空的，不算冲突
        assert_eq!(conflicting_owner(25575, vec![("停着的服".to_string(), 25575u16, false)]), None);
        // 端口都没撞上
        assert_eq!(conflicting_owner(25577, candidates), None);
    }

    #[test]
    fn maps_rcon_errors_to_contract_errors() {
        assert_eq!(
            to_service_error(&RconError::Unauthorized),
            PlayerPositionServiceError::RconUnauthorized
        );
        assert_eq!(
            to_service_error(&RconError::Timeout),
            PlayerPositionServiceError::RconUnreachable
        );
        assert_eq!(
            to_service_error(&RconError::Protocol("bad".to_string())),
            PlayerPositionServiceError::MalformedResponse
        );
    }
}
