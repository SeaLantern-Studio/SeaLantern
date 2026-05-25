# sculk

P2P tunnel core library for Minecraft multiplayer, built on [iroh](https://crates.io/crates/iroh)/QUIC.

Provides tunnel establishment, ticket protocol, connection management, and event streaming for CLI/TUI or third-party client integration.

## Key Types

- `IrohTunnel::host` / `IrohTunnel::join` — Create or join a tunnel
- `Ticket` — Connection ticket in `sculk://<EndpointId>?relay=<url>` format
- `HostConfig` / `JoinConfig` — Per-side tunnel configuration (password, max players, reconnection, etc.)
- `TunnelEvent` — Runtime event stream (player join/leave, path changes, reconnection, etc.)
- `PeerId` — Peer identity in events and snapshots
- `ConnectionSnapshot` — Connection snapshot (direct/relay, RTT, traffic stats)

## Feature Flags

| Feature     | Default | Description                                       |
| ----------- | ------- | ------------------------------------------------- |
| `persist`   | No      | Secret key persistence + profile.toml preferences |
| `clipboard` | No      | System clipboard write support                    |

## Example

```rust
use sculk::tunnel::{IrohTunnel, HostConfig, JoinConfig};

// Host
let (tunnel, ticket, mut rx) = IrohTunnel::host(
    25565, None, None, HostConfig::default(),
).await?;

// Join
let (tunnel, mut rx) = IrohTunnel::join(
    &ticket, 30000, JoinConfig::default(),
).await?;
```

---

## 中文说明

面向 Minecraft 联机的 P2P 隧道核心库，基于 [iroh](https://crates.io/crates/iroh)/QUIC。

提供隧道建立、票据协议、连接管理与事件流能力，供 CLI/TUI 或第三方客户端集成。

### 主要类型

- `IrohTunnel::host` / `IrohTunnel::join` — 创建/加入隧道
- `Ticket` — 连接票据，格式为 `sculk://<EndpointId>?relay=<url>`
- `HostConfig` / `JoinConfig` — 分端隧道配置（密码、人数上限、重连策略等）
- `TunnelEvent` — 运行时事件推送（玩家加入/离开、路径变化、重连等）
- `PeerId` — 事件与快照中的对端标识
- `ConnectionSnapshot` — 连接快照（直连/中继、RTT、流量统计）

### Feature Flags

| Feature     | 默认 | 说明                               |
| ----------- | ---- | ---------------------------------- |
| `persist`   | 否   | 密钥持久化 + profile.toml 偏好配置 |
| `clipboard` | 否   | 系统剪贴板写入                     |

更多信息见[项目主页](https://github.com/KercyDing/sculk)。
