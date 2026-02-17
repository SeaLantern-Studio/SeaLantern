import { tauriInvoke } from "./tauri";
import type { ServerInstance } from "../types/server";

export interface ServerStatusInfo {
  id: string;
  status: "Stopped" | "Starting" | "Running" | "Stopping" | "Error";
  pid: number | null;
  uptime: number | null;
}

export const serverApi = {
  async create(params: {
    name: string;
    coreType: string;
    mcVersion: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    javaPath: string;
    jarPath: string;
    startupMode?: "jar" | "bat" | "sh";
  }): Promise<ServerInstance> {
    return tauriInvoke("create_server", {
      name: params.name,
      coreType: params.coreType,
      mcVersion: params.mcVersion,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      javaPath: params.javaPath,
      jarPath: params.jarPath,
      startupMode: params.startupMode ?? "jar",
    });
  },

  async importServer(params: {
    name: string;
    jarPath: string;
    startupMode: "jar" | "bat" | "sh";
    javaPath: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    onlineMode: boolean;
  }): Promise<ServerInstance> {
    return tauriInvoke("import_server", {
      name: params.name,
      jarPath: params.jarPath,
      startupMode: params.startupMode,
      javaPath: params.javaPath,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      onlineMode: params.onlineMode,
    });
  },

  async importModpack(params: {
    name: string;
    modpackPath: string;
    javaPath: string;
    maxMemory: number;
    minMemory: number;
    port: number;
  }): Promise<ServerInstance> {
    return tauriInvoke("import_modpack", {
      name: params.name,
      modpackPath: params.modpackPath,
      javaPath: params.javaPath,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
    });
  },

  async start(id: string): Promise<void> {
    return tauriInvoke("start_server", { id });
  },

  async stop(id: string): Promise<void> {
    return tauriInvoke("stop_server", { id });
  },

  async forceStopAll(): Promise<void> {
    return tauriInvoke("force_stop_all_servers");
  },

  async sendCommand(id: string, command: string): Promise<void> {
    return tauriInvoke("send_command", { id, command });
  },

  async getList(): Promise<ServerInstance[]> {
    return tauriInvoke("get_server_list");
  },

  async getStatus(id: string): Promise<ServerStatusInfo> {
    return tauriInvoke("get_server_status", { id });
  },

  async deleteServer(id: string): Promise<void> {
    return tauriInvoke("delete_server", { id });
  },

  async getLogs(id: string, since: number): Promise<string[]> {
    return tauriInvoke("get_server_logs", { id, since });
  },

  async addServerCommand(id: string, name: string, command: string): Promise<void> {
    return tauriInvoke("add_server_command", { id, name, command });
  },

  async updateServerCommand(
    id: string,
    commandId: string,
    name: string,
    command: string,
  ): Promise<void> {
    return tauriInvoke("update_server_command", { id, commandId, name, command });
  },

  async deleteServerCommand(id: string, commandId: string): Promise<void> {
    return tauriInvoke("delete_server_command", { id, commandId });
  },

  async updateServerName(id: string, name: string): Promise<void> {
    return tauriInvoke("update_server_name", { id, name });
  },
};
