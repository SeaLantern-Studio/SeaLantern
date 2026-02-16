import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import pinia from "./stores";
import "./style.css";
// `setupTray` 在浏览器环境中会引用 Tauri API，改为运行时按需导入

const app = createApp(App);

// 全局错误处理（仅在开发环境）
if (import.meta.env.DEV) {
  app.config.errorHandler = (err, instance, info) => {
    console.error("App Error:", err, "Info:", info, "Instance:", instance);
  };

  window.addEventListener("unhandledrejection", (event) => {
    console.error("Unhandled Promise:", event.reason);
  });
}

app.use(pinia);
app.use(router);

// 挂载应用
app.mount("#app");
// 当应用准备就绪后显示主窗口
window.addEventListener("load", async () => {
  try {
    // 尝试动态导入 Tauri API
    const tauriApi = await import("@tauri-apps/api/window");
    if (tauriApi && tauriApi.getCurrent) {
      const mainWindow = tauriApi.getCurrent();
      await mainWindow.show();
      // 如果有启动画面，可以在这里隐藏它
      if (tauriApi.getByLabel) {
        const splashWindow = tauriApi.getByLabel("splashscreen");
        if (splashWindow) {
          await splashWindow.hide();
          await splashWindow.close();
        }
      }
    }
    // 在 Tauri 环境下按需加载并设置托盘
    try {
      const { setupTray } = await import("./utils/tray");
      if (typeof setupTray === "function") {
        await setupTray();
      }
    } catch (trayErr) {
      console.warn("Failed to set up tray, tray functionality will be unavailable:", trayErr);
    }
  } catch (e) {
    console.log("Tauri API not available, running in non-Tauri environment:", e);
  }
});
