import { createApp } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { isBrowserEnv, tauriInvoke } from "@api/tauri";
import App from "@src/App.vue";
import router from "@src/router";
import pinia from "@src/stores";
import "@src/style.css";

const HEARTBEAT_INTERVAL = 5000;

function startHeartbeat() {
  if (isBrowserEnv()) return;

  setInterval(() => {
    tauriInvoke("frontend_heartbeat", undefined, { silent: true }).catch(() => {
      // 后端可能已退出或当前不在 Tauri 环境中
    });
  }, HEARTBEAT_INTERVAL);
}

export async function mountClassicApp(): Promise<void> {
  const app = createApp(App);

  if (import.meta.env.DEV) {
    app.config.errorHandler = (err, instance, info) => {
      console.error("App Error:", err, "Info:", info, "Instance:", instance);
    };

    window.addEventListener("unhandledrejection", (event) => {
      console.error("Unhandled Promise:", event.reason);
    });

    (window as Window & { __invoke?: typeof invoke }).__invoke = invoke;
  }

  app.use(pinia);
  app.use(router);
  app.mount("#app");

  startHeartbeat();
}
