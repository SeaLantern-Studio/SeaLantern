import type { App } from "vue";
import { SLContextMenu } from "@components/common";
import { AppHeader, AppLayout, AppSidebar } from "@components/layout";

const components: Record<string, any> = {
  SLContextMenu,
  AppHeader,
  AppLayout,
  AppSidebar,
};

export function install(app: App): void {
  for (const [name, component] of Object.entries(components)) {
    app.component(name, component);
  }
}

export default { install };
