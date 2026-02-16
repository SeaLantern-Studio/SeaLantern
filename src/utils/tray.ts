import { TrayIcon } from "@tauri-apps/api/tray";
import { Menu } from "@tauri-apps/api/menu";
import i18n from "../locales";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { defaultWindowIcon } from "@tauri-apps/api/app";

// 菜单项ID常量
enum TrayMenuId {
  Show = "show",
  Minimize = "minimize",
  Quit = "quit",
}

// 创建托盘菜单
export async function setupTray() {
  const menu = await Menu.new({
    items: [
      {
        id: TrayMenuId.Show,
        text: i18n.t("tray.show"),
        action: async () => {
          const w = getCurrentWindow();
          await w.show();
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
          // 后台最小化：隐藏窗口以便从托盘恢复
          try {
            // 尽量同时隐藏窗口并从任务栏移除，这样桌面不会保留图标
            await w.hide();
            try {
              await w.setSkipTaskbar(true);
            } catch (e) {
              console.warn("Failed to set skip taskbar:", e);
            }
          } catch (e) {
            console.warn("Failed to hide window for tray minimize:", e);
            await w.minimize();
          }
        },
      },
      {
        id: TrayMenuId.Quit,
        text: i18n.t("tray.quit"),
        action: async () => {
          const w = getCurrentWindow();
          try {
            w.close();
          } catch (e) {
            console.warn("Failed to close window:", e);
            await w.show();
          }
        },
      },
    ],
  });

  const options = {
    icon: await defaultWindowIcon(),
    menu,
    menuOnLeftClick: true,
  };

  await TrayIcon.new(options as any);
}
