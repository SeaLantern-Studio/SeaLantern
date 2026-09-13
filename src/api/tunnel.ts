import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isBrowserEnv } from "@api/tauri";
import { invoke } from "@api/invoke";

export interface TunnelConnection {
  remote_id: string;
  is_relay: boolean;
  rtt_ms: number;
  tx_bytes: number;
  rx_bytes: number;
  alive: boolean;
  elapsed_secs: number;
}

/** 隧道生命周期阶段（与后端 snake_case 对齐） */
export type OnlineTunnelPhase = "idle" | "starting" | "active" | "stopping";

/** 隧道错误的产品级分类（与后端 snake_case 对齐） */
export type OnlineTunnelErrorCategory =
  | "invalid_join_uri"
  | "invalid_endpoint"
  | "authorization_denied"
  | "host_unreachable"
  | "target_unavailable"
  | "local_port_unavailable"
  | "identity_unavailable"
  | "operation_conflict"
  | "resource_limit"
  | "invalid_configuration"
  | "internal";

export interface TunnelStatus {
  running: boolean;
  phase: OnlineTunnelPhase;
  mode: "host" | "join" | null;
  /** 后端已转换为用户侧分享链接（`https://ideaflash.cn/#v1/...`） */
  ticket: string | null;
  /** 加入方实际绑定的本地监听地址；房主或未运行时为 null */
  localAddress: string | null;
  connections: TunnelConnection[];
  lastError: OnlineTunnelErrorCategory | null;
}

/** 后端 online_tunnel_event 的事件负载（serde tag = "kind"） */
export type OnlineTunnelEvent =
  | { kind: "started"; mode: "host" | "join"; ticket: string | null }
  | { kind: "stopped"; mode: "host" | "join" }
  | { kind: "player_joined"; remote_id: string }
  | { kind: "player_left"; remote_id: string; reason: string }
  | { kind: "connected" }
  | { kind: "disconnected"; reason: string }
  | { kind: "path_changed"; remote_id: string; is_relay: boolean; rtt_ms: number }
  | { kind: "reconnecting"; attempt: number }
  | { kind: "reconnected" }
  | { kind: "authentication_failed"; remote_id: string }
  | { kind: "player_rejected"; remote_id: string; reason: string }
  | { kind: "token_rotated" }
  | { kind: "error"; category: OnlineTunnelErrorCategory; message: string }
  | { kind: "provider_message"; message: string };

export type TunnelLinkLifetime = "always" | "never" | "1h" | "3h" | "6h" | "12h" | "24h";

export const TUNNEL_LINK_LIFETIMES: readonly TunnelLinkLifetime[] = [
  "always",
  "never",
  "1h",
  "3h",
  "6h",
  "12h",
  "24h",
];

export interface TunnelHostParams {
  port: number;
  maxPlayers?: number;
  relayUrl?: string;
  linkLifetime: TunnelLinkLifetime;
}

export interface TunnelJoinParams {
  ticket: string;
  localPort: number;
}

/** 后端 OnlineTunnelStatus 原始结构，字段和前端 TunnelStatus 差异较大 */
interface TunnelStatusRaw {
  active: boolean;
  phase: OnlineTunnelPhase;
  mode: "host" | "join" | null;
  ticket: string | null;
  local_address: string | null;
  connections: TunnelConnectionRaw[];
  last_error: OnlineTunnelErrorCategory | null;
}

/** 后端连接信息用 elapsed_ms，前端用 elapsed_secs */
interface TunnelConnectionRaw {
  remote_id: string;
  is_relay: boolean;
  rtt_ms: number;
  tx_bytes: number;
  rx_bytes: number;
  alive: boolean;
  elapsed_ms: number;
}

/** 后端状态转前端（后端只返回 OnlineTunnelStatus 的字段） */
function toTunnelStatus(raw: TunnelStatusRaw): TunnelStatus {
  return {
    running: raw.active,
    phase: raw.phase,
    mode: raw.mode,
    ticket: raw.ticket,
    /** 后端未返回该字段（旧版本）时按无地址处理 */
    localAddress: raw.local_address ?? null,
    lastError: raw.last_error,
    connections: raw.connections.map((c) => ({
      remote_id: c.remote_id,
      is_relay: c.is_relay,
      rtt_ms: c.rtt_ms,
      tx_bytes: c.tx_bytes,
      rx_bytes: c.rx_bytes,
      alive: c.alive,
      // 后端毫秒，前端秒
      elapsed_secs: c.elapsed_ms / 1000,
    })),
  };
}

export const tunnelApi = {
  async host(params: TunnelHostParams): Promise<TunnelStatus> {
    // 后端 OnlineTunnelHostRequest 用 snake_case，port 映射为 minecraft_port
    const raw = await invoke<TunnelStatusRaw>("online_tunnel_host", {
      request: {
        minecraft_port: params.port,
        max_players: params.maxPlayers,
        relay_url: params.relayUrl,
        link_lifetime: params.linkLifetime,
      },
    });
    return toTunnelStatus(raw);
  },

  async join(params: TunnelJoinParams): Promise<TunnelStatus> {
    const raw = await invoke<TunnelStatusRaw>("online_tunnel_join", {
      request: {
        ticket: params.ticket,
        local_port: params.localPort,
      },
    });
    return toTunnelStatus(raw);
  },

  async stop(): Promise<TunnelStatus> {
    const raw = await invoke<TunnelStatusRaw>("online_tunnel_stop");
    return toTunnelStatus(raw);
  },

  async status(): Promise<TunnelStatus> {
    const raw = await invoke<TunnelStatusRaw>("online_tunnel_status");
    return toTunnelStatus(raw);
  },
};

/**
 * 订阅在线隧道运行事件。
 *
 * Tauri 模式下监听后端 `online_tunnel_event`；浏览器/Docker 模式后端没有该事件的
 * 转发通道（也没有可用的 SSE 端点），优雅降级为"不订阅"。
 *
 * 调用方需成对调用返回的 unlisten，避免重复订阅导致日志重复。
 */
export function onTunnelEvent(callback: (event: OnlineTunnelEvent) => void): Promise<UnlistenFn> {
  if (isBrowserEnv()) {
    return Promise.resolve(() => {});
  }
  return listen<OnlineTunnelEvent>("online_tunnel_event", (event) => {
    callback(event.payload);
  });
}
