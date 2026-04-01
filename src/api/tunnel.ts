import { tauriInvoke } from "@api/tauri";

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

export const tunnelApi = {
  async host(params: TunnelHostParams): Promise<TunnelStatus> {
    return tauriInvoke("tunnel_host", {
      port: params.port,
      password: params.password,
      maxPlayers: params.maxPlayers,
      relayUrl: params.relayUrl,
    });
  },

  async join(params: TunnelJoinParams): Promise<TunnelStatus> {
    return tauriInvoke("tunnel_join", {
      ticket: params.ticket,
      localPort: params.localPort,
      password: params.password,
    });
  },

  async stop(): Promise<TunnelStatus> {
    return tauriInvoke("tunnel_stop");
  },

  async status(): Promise<TunnelStatus> {
    return tauriInvoke("tunnel_status");
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
