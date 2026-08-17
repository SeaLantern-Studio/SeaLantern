import { createApp } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
  Cmz_Markdown,
  Cmz_Modal,
  Cmz_Progress,
  Cmz_Select,
  Cmz_Spinner,
  Cmz_Switch,
  Cmz_TabBar,
  Cmz_Textarea,
  Cmz_Toast,
  Cmz_Toggle,
  Cmz_Tooltip,
} from "cmzya-modern-ui";

// ECharts 已改为按需懒加载,见 src/components/views/home/SystemStatsCard.vue

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
app.component("cmz-markdown", Cmz_Markdown);
app.component("cmz-progress", Cmz_Progress);
app.component("cmz-select", Cmz_Select);
app.component("cmz-spinner", Cmz_Spinner);
app.component("cmz-switch", Cmz_Switch);
app.component("cmz-tab-bar", Cmz_TabBar);
app.component("cmz-textarea", Cmz_Textarea);
app.component("cmz-toast", Cmz_Toast);
app.component("cmz-toggle", Cmz_Toggle);
app.component("cmz-tooltip", Cmz_Tooltip);

if (import.meta.env.DEV) {
  app.config.errorHandler = (err, instance, info) => {
    console.error("App Error:", err, "Info:", info, "Instance:", instance);
  };

  window.addEventListener("unhandledrejection", (event) => {
    console.error("Unhandled Promise:", event.reason);
  });

  // DEV 模式下将 invoke 与 listen 挂载到 window，方便在浏览器控制台手动调用 Tauri 命令，并监听 Tauri 事件。
  // 例如触发崩溃报告测试：await window.__invoke("debug_panic")
  // 例如监听服务器日志事件：
  // let unlisten = await __listen("server-log-line", (event) => {console.log(event)});
  // 使用 await unlisten(); 以取消监听。
  // 注意：此挂载仅在开发模式下存在，生产包中不会包含。
  (window as any).__invoke = invoke;
  (window as any).__listen = listen;
}

app.use(pinia);
app.use(router);
app.mount("#app");
