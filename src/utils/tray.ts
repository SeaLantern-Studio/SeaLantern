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
// 存储托盘图标实例，防止重复创建
let trayIconInstance: any = null;

/**
 * 设置托盘图标
 * 确保只创建一个托盘图标实例
 */
export async function setupTray() {
  // 如果已经创建了托盘图标实例，直接返回
  if (trayIconInstance) {
    return;
  }

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
    menuOnLeftClick: false, // 禁用左键显示菜单，改为点击打开主界面
  };

  // 创建托盘图标实例并存储
  trayIconInstance = await TrayIcon.new(options as any);

  // 添加左键点击事件处理
  if (trayIconInstance && typeof trayIconInstance.onClick === "function") {
    trayIconInstance.onClick(async () => {
      const w = getCurrentWindow();
      try {
        const isVisible = await w.isVisible();
        if (isVisible) {
          await w.hide();
        } else {
          await w.show();
          await w.setFocus();
        }
      } catch (e) {
        console.warn("Failed to toggle window visibility:", e);
        await w.show();
        await w.setFocus();
      }
    });
  }
}
