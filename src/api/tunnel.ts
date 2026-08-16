import { tauriInvoke } from "@api/tauri";
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
  logs: string[];
  host_port: number;
  join_port: number;
  last_ticket: string | null;
  relay_url: string | null;
}

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

/** 后端状态转前端，后端不支持的字段给默认值 */
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
    logs: [],
    host_port: 0,
    join_port: 0,
    last_ticket: null,
    relay_url: null,
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
