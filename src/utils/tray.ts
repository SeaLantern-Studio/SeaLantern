import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu } from "@tauri-apps/api/menu";
import { i18n } from "../locales";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { defaultWindowIcon } from "@tauri-apps/api/app";
import { exit } from "@tauri-apps/plugin-process";

// 菜单项ID常量
enum TrayMenuId {
  Show = "show",
  Minimize = "minimize",
  Quit = "quit",
}

// 存储托盘图标实例，防止重复创建
let trayIconInstance: TrayIcon | null = null;

/**
 * 设置托盘图标
 * 确保只创建一个托盘图标实例
 */
export async function setupTray() {
  // 如果已经创建了托盘图标实例，直接返回
  if (trayIconInstance) {
    console.log("Tray icon already exists, skipping creation");
    return;
  }

  try {
    const menu = await Menu.new({
      items: [
        {
          id: TrayMenuId.Show,
          text: i18n.t("tray.show"),
          action: async () => {
            const w = getCurrentWindow();
            await w.show();
            await w.unminimize();
            try {
              await w.setFocus();
            } catch (e) {
              console.warn("Failed to focus window after showing:", e);
            }
          },
        },
        {
          id: TrayMenuId.Minimize,
          text: i18n.t("tray.minimize"),
          action: async () => {
            const w = getCurrentWindow();
            await w.hide();
          },
        },
        {
          id: TrayMenuId.Quit,
          text: i18n.t("tray.quit"),
          action: async () => {
            try {
              // 调用后端停止所有服务器
              const { tauriInvoke } = await import("../api/tauri");
              await tauriInvoke("force_stop_all_servers");
            } catch (e) {
              console.warn("Failed to stop servers before quit:", e);
            }
            // 退出应用
            await exit(0);
          },
        },
      ],
    });

    const options = {
      icon: await defaultWindowIcon(),
      menu,
      menuOnLeftClick: false,
    };

    // 创建托盘图标实例并存储
    trayIconInstance = await TrayIcon.new(options as any);
    console.log("Tray icon created successfully");
  } catch (error) {
    console.error("Failed to create tray icon:", error);
    throw error;
  }
}

/**
 * 清理托盘图标
 */
export async function cleanupTray() {
  if (trayIconInstance) {
    try {
      await trayIconInstance.close();
      trayIconInstance = null;
      console.log("Tray icon cleaned up");
    } catch (e) {
      console.warn("Failed to cleanup tray icon:", e);
    }
  }
}
