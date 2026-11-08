//! 玩家实时位置服务端口。

use async_trait::async_trait;
use sealantern_contract::{Dimension, PlayerPositionServiceError, PlayerPositions};

/// 查询服务器某个维度下所有在线玩家实时位置的宿主能力端口。
///
/// 与 [`crate::port::players`] 的区别：那个端口查的是磁盘上的玩家档案
/// （白名单、封禁、OP、在线名单），不需要服务器在运行；本端口问的是
/// 「此刻人在哪」，必须通过 RCON 连上活着的服务器，服务器没开就拿不到。
#[async_trait]
pub trait PlayerPositionService: Send + Sync {
    /// 取某个维度下在线玩家的位置快照。
    ///
    /// 没有玩家在线时返回空列表而不是错误；只有连不上服务器、RCON 未启用、
    /// 认证失败这类情况才返回 `Err`。
    async fn positions(
        &self,
        instance_id: &str,
        dimension: Dimension,
    ) -> Result<PlayerPositions, PlayerPositionServiceError>;
}
