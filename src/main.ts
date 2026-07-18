import { createApp } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { tauriInvoke } from "@api/tauri";
import App from "@src/App.vue";
import router from "@src/router";
import pinia from "@src/stores";
import "cmzya-modern-ui/style.css";
import "@src/style.css";
import {
  Cmz_Badge,
  Cmz_Button,
  Cmz_Card,
  Cmz_Checkbox,
  Cmz_Console,
  Cmz_Divider,
  Cmz_Dropzone,
  Cmz_FormField,
  Cmz_Input,
  Cmz_Modal,
  Cmz_Progress,
  Cmz_Select,
  Cmz_Spinner,
  Cmz_Switch,
  Cmz_TabBar,
  Cmz_Textarea,
  Cmz_Toggle,
  Cmz_Tooltip,
} from "cmzya-modern-ui";
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
    tauriInvoke("frontend_heartbeat", undefined, { silent: true }).catch(() => {
      // 后端可能已退出或当前不在 Tauri 环境中
    });
  }, HEARTBEAT_INTERVAL);
}

const app = createApp(App);

// 全局注册 CmzYa Modern UI 组件 (kebab-case)
app.component("cmz-badge", Cmz_Badge);
app.component("cmz-button", Cmz_Button);
app.component("cmz-card", Cmz_Card);
app.component("cmz-checkbox", Cmz_Checkbox);
app.component("cmz-console", Cmz_Console);
app.component("cmz-divider", Cmz_Divider);
app.component("cmz-dropzone", Cmz_Dropzone);
app.component("cmz-form-field", Cmz_FormField);
app.component("cmz-input", Cmz_Input);
app.component("cmz-modal", Cmz_Modal);
app.component("cmz-progress", Cmz_Progress);
app.component("cmz-select", Cmz_Select);
app.component("cmz-spinner", Cmz_Spinner);
app.component("cmz-switch", Cmz_Switch);
app.component("cmz-tab-bar", Cmz_TabBar);
app.component("cmz-textarea", Cmz_Textarea);
app.component("cmz-toggle", Cmz_Toggle);
app.component("cmz-tooltip", Cmz_Tooltip);

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
