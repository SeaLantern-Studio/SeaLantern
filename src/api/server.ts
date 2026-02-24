import { tauriInvoke } from "@api/tauri";
import type { ServerInstance } from "@type/server";

export interface ServerStatusInfo {
  id: string;
  status: "Stopped" | "Starting" | "Running" | "Stopping" | "Error";
  pid: number | null;
  uptime: number | null;
}

export interface ParsedServerCoreInfo {
  coreType: string;
  mainClass: string | null;
  jarPath: string | null;
}

export interface StartupCandidateItem {
  id: string;
  mode: "starter" | "jar" | "bat" | "sh" | "ps1";
  label: string;
  detail: string;
  path: string;
  recommended: number;
}

interface ParsedServerCoreInfoRaw {
  core_type: string;
  main_class: string | null;
  jar_path: string | null;
}

interface StartupCandidateItemRaw {
  id: string;
  mode: string;
  label: string;
  detail: string;
  path: string;
  recommended: number;
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
    startupMode?: "jar" | "bat" | "sh" | "ps1";
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
    startupMode: "jar" | "bat" | "sh" | "ps1";
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

  async parseServerCoreType(sourcePath: string): Promise<ParsedServerCoreInfo> {
    const result = await tauriInvoke<ParsedServerCoreInfoRaw>("parse_server_core_type", { sourcePath });
    return {
      coreType: result.core_type,
      mainClass: result.main_class,
      jarPath: result.jar_path,
    };
  },

  async scanStartupCandidates(
    sourcePath: string,
    sourceType: "archive" | "folder",
  ): Promise<StartupCandidateItem[]> {
    const result = await tauriInvoke<StartupCandidateItemRaw[]>("scan_startup_candidates", {
      sourcePath,
      sourceType,
    });

    return result.map((item) => ({
      id: item.id,
      mode: (item.mode as StartupCandidateItem["mode"]) ?? "jar",
      label: item.label,
      detail: item.detail,
      path: item.path,
      recommended: item.recommended,
    }));
  },

  async collectCopyConflicts(sourceDir: string, targetDir: string): Promise<string[]> {
    return tauriInvoke("collect_copy_conflicts", { sourceDir, targetDir });
  },

  async copyDirectoryContents(sourceDir: string, targetDir: string): Promise<void> {
    return tauriInvoke("copy_directory_contents", { sourceDir, targetDir });
  },

  async addExistingServer(params: {
    name: string;
    serverPath: string;
    javaPath: string;
    maxMemory: number;
    minMemory: number;
    port: number;
    startupMode: "jar" | "bat" | "sh" | "ps1";
    executablePath?: string;
  }): Promise<ServerInstance> {
    return tauriInvoke("add_existing_server", {
      name: params.name,
      serverPath: params.serverPath,
      javaPath: params.javaPath,
      maxMemory: params.maxMemory,
      minMemory: params.minMemory,
      port: params.port,
      startupMode: params.startupMode,
      executablePath: params.executablePath,
    });
  },

  async start(id: string): Promise<void> {
    return tauriInvoke("start_server", { id });
  },

  async stop(id: string): Promise<void> {
    return tauriInvoke("stop_server", { id });
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

  async updateServerName(id: string, name: string): Promise<void> {
    return tauriInvoke("update_server_name", { id, name });
  },
};
