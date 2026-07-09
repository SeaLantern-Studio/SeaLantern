import { tauriInvoke } from "@api/tauri";

export type ServerExtensionEntryKind = "plugin" | "mod" | "other";

export interface ServerExtensionEntrySummary {
  name: string;
  relative_path: string;
  size_bytes: number;
  modified_at: string | null;
  kind: ServerExtensionEntryKind;
}

export interface ServerExtensionsSummary {
  has_plugins_dir: boolean;
  has_mods_dir: boolean;
  plugin_entries: ServerExtensionEntrySummary[];
  mod_entries: ServerExtensionEntrySummary[];
  other_entries: ServerExtensionEntrySummary[];
}

export const serverExtensionsApi = {
  async getServerExtensionsSummary(serverPath: string): Promise<ServerExtensionsSummary> {
    return tauriInvoke("get_server_extensions_summary", { serverPath });
  },
};
