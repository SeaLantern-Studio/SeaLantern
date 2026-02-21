import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import pinia from "./stores";
import "./style.css";
import VueECharts from "vue-echarts";
import { use } from 'echarts/core'
import { PieChart, LineChart } from 'echarts/charts'
import { GridComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

// 注册 ECharts 必要的组件
use([GridComponent, PieChart, LineChart, CanvasRenderer]);

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
}

app.use(pinia);
app.use(router);
app.mount("#app");
