import { createApp } from "vue";
import NextRoot from "./NextRoot.vue";
import nextRouter from "./router";
import pinia from "@src/stores";
import "@src/style.css";

export async function mountNextApp(): Promise<void> {
  const app = createApp(NextRoot);

  app.use(pinia);
  app.use(nextRouter);
  app.mount("#app");
}
