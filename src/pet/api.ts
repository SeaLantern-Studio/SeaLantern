import { invoke } from "@api/invoke";
import { serverApi, type ServerStatusInfo } from "@api/server";
import { isBrowserEnv } from "@api/tauri";

export interface PetServerStatus {
  id: string;
  name: string;
  status: ServerStatusInfo["status"];
  uptime: number | null;
  demo?: boolean;
}

const PREVIEW_SERVERS: PetServerStatus[] = [
  {
    id: "preview-survival",
    name: "海风生存服",
    status: "Running",
    uptime: 3 * 3600 + 26 * 60,
    demo: true,
  },
  {
    id: "preview-creative",
    name: "创造试验场",
    status: "Starting",
    uptime: null,
    demo: true,
  },
  {
    id: "preview-modpack",
    name: "整合包周末局",
    status: "Stopped",
    uptime: null,
    demo: true,
  },
];

export const petApi = {
  isPreview(): boolean {
    return isBrowserEnv();
  },

  async frontendReady(): Promise<void> {
    if (isBrowserEnv()) return;
    await invoke("pet_frontend_ready");
  },

  async hide(): Promise<void> {
    if (isBrowserEnv()) return;
    await invoke("pet_hide");
  },

  async openMainWindow(): Promise<void> {
    if (isBrowserEnv()) return;
    await invoke("pet_open_main_window");
  },

  async setClickThrough(ignore: boolean): Promise<void> {
    if (isBrowserEnv()) return;
    await invoke("pet_set_click_through", { ignore });
  },

  async startDesktopJelly(): Promise<void> {
    if (isBrowserEnv()) return;
    await invoke("desktop_jelly_start");
  },

  async stopDesktopJelly(): Promise<void> {
    if (isBrowserEnv()) return;
    await invoke("desktop_jelly_stop");
  },

  async loadServerStatuses(): Promise<PetServerStatus[]> {
    // 浏览器预览没有 Tauri 后端，使用明确的演示数据展示气泡布局。
    if (isBrowserEnv()) {
      return PREVIEW_SERVERS;
    }

    const servers = await serverApi.getList();
    const statuses = await Promise.all(
      servers.map(async (server) => {
        try {
          const status = await serverApi.getStatus(server.id);
          return {
            id: server.id,
            name: server.name,
            status: status.status,
            uptime: status.uptime,
          } satisfies PetServerStatus;
        } catch (error) {
          console.warn(`桌宠读取服务器 ${server.name} 状态失败:`, error);
          return {
            id: server.id,
            name: server.name,
            status: "Unknown",
            uptime: null,
          } satisfies PetServerStatus;
        }
      }),
    );
    return statuses;
  },
};
