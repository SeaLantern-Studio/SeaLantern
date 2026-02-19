import { tauriInvoke } from "./tauri";

export type ModLoader = "fabric" | "forge" | "quilt" | "neoforge";

export interface ModInfo {
  id: string;
  name: string;
  summary: string;
  download_url: string;
  file_name: string;
  source: string;
}

export const modsApi = {
  async searchMods(params: {
    query: string;
    gameVersion: string;
    loader: ModLoader;
  }): Promise<ModInfo[]> {
    return tauriInvoke("search_mods", {
      query: params.query,
      gameVersion: params.gameVersion,
      loader: params.loader,
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
