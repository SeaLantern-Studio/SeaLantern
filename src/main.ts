import { createApp } from "vue";
import { invoke } from "@tauri-apps/api/core";
import App from "@src/App.vue";
import router from "@src/router";
import pinia from "@src/stores";
import "@src/style.css";
import VueECharts from "vue-echarts";
import { use } from "echarts/core";
import { PieChart, LineChart } from "echarts/charts";
import { GridComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";

// 注册 ECharts 必要的组件
use([GridComponent, PieChart, LineChart, CanvasRenderer]);

const HEARTBEAT_INTERVAL = 5000;

function startHeartbeat() {
  // 在普通浏览器环境下，Tauri 后端不存在，调用会直接失败，这里静默忽略错误
  setInterval(() => {
    invoke("frontend_heartbeat").catch(() => {
      // 后端可能已退出或当前不在 Tauri 环境中
    });
  }, HEARTBEAT_INTERVAL);
}

const app = createApp(App);
// 全局注册 vue-echarts
app.component("v-chart", VueECharts);

if (import.meta.env.DEV) {
  app.config.errorHandler = (err, instance, info) => {
    console.error("App Error:", err, "Info:", info, "Instance:", instance);
  };

  window.addEventListener("unhandledrejection", (event) => {
    console.error("Unhandled Promise:", event.reason);
  });

  // DEV 模式下将 invoke 挂载到 window，方便在浏览器控制台手动调用 Tauri 命令。
  // 例如触发崩溃报告测试：await window.__invoke("debug_panic")
  // 注意：此挂载仅在开发模式下存在，生产包中不会包含。
  (window as any).__invoke = invoke;
}

app.use(pinia);
app.use(router);
app.mount("#app");

startHeartbeat();
