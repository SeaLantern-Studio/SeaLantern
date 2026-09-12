import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isBrowserEnv, tauriInvoke } from "@api/tauri";
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

export interface TunnelStatus {
  running: boolean;
  mode: "host" | "join" | null;
  ticket: string | null;
  connections: TunnelConnection[];
}

/** 后端 online_tunnel_event 的事件负载（serde tag = "kind"） */
export type OnlineTunnelEvent =
  | { kind: "started"; mode: "host" | "join" }
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
  | { kind: "error"; message: string }
  | { kind: "provider_message"; message: string };

export interface TunnelHostParams {
  port: number;
  password?: string;
  maxPlayers?: number;
  relayUrl?: string;
}

export interface TunnelJoinParams {
  ticket: string;
  localPort: number;
  password?: string;
}

/** 后端 OnlineTunnelStatus 原始结构，字段和前端 TunnelStatus 差异较大 */
interface TunnelStatusRaw {
  active: boolean;
  mode: "host" | "join" | null;
  ticket: string | null;
  connections: TunnelConnectionRaw[];
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
    mode: raw.mode,
    ticket: raw.ticket,
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
        password: params.password,
        max_players: params.maxPlayers,
        relay_url: params.relayUrl,
      },
    });
    return toTunnelStatus(raw);
  },

  async join(params: TunnelJoinParams): Promise<TunnelStatus> {
    const raw = await invoke<TunnelStatusRaw>("online_tunnel_join", {
      request: {
        ticket: params.ticket,
        local_port: params.localPort,
        password: params.password,
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

  // TODO(backend): 以下票据命令后端均未实现（tunnel_copy_ticket / tunnel_regenerate_ticket /
  // tunnel_generate_ticket 没有对应 Tauri 命令）。ticket 目前只在 host 成功后由
  // status.ticket 返回，UI 侧已禁用这些入口，待后端补齐票据能力后再接通。
  async copyTicket(): Promise<boolean> {
    return tauriInvoke("tunnel_copy_ticket");
  },

  async regenerateTicket(): Promise<TunnelStatus> {
    return tauriInvoke("tunnel_regenerate_ticket");
  },

  async generateTicket(): Promise<TunnelStatus> {
    return tauriInvoke("tunnel_generate_ticket");
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
