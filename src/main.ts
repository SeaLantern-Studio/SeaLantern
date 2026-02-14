import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import pinia from "./stores";
import i18n from "./i18n";
import "./style.css";

const app = createApp(App);

// 全局错误处理（仅在开发环境）
if (import.meta.env.DEV) {
  app.config.errorHandler = (err, instance, info) => {
    console.error('App Error:', err);
  };

  window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled Promise:', event.reason);
  });
}

app.use(pinia);
app.use(router);
app.use(i18n);
app.mount("#app");

