import { tauriInvoke } from "./tauri";

export type ModLoader = "fabric" | "forge" | "quilt" | "neoforge";

export interface ModInfo {
  id: string;
  name: string;
  summary: string;
  download_url: string;
  file_name: string;
  source: string;
  icon_url?: string | null;
  downloads: number;
}

export interface SearchModsResult {
  items: ModInfo[];
  total: number;
  offset: number;
  limit: number;
}

export const modsApi = {
  async searchMods(params: {
    query: string;
    gameVersion: string;
    loader: ModLoader;
    page: number;
    pageSize: number;
  }): Promise<SearchModsResult> {
    return tauriInvoke("search_mods", {
      query: params.query,
      gameVersion: params.gameVersion,
      loader: params.loader,
      page: params.page,
      pageSize: params.pageSize,
    });
  },

  async installMod(params: {
    serverId: string;
    downloadUrl: string;
    fileName: string;
  }): Promise<void> {
    return tauriInvoke("install_mod", {
      serverId: params.serverId,
      downloadUrl: params.downloadUrl,
      fileName: params.fileName,
    });
  },
};
