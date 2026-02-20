import type { App } from "vue";

export * from "./common";

export * from "./layout";

export * from "./plugin";

export type { TabItem } from "./common/SLTabs.vue";

import {
  SLBadge,
  SLButton,
  SLCard,
  SLCheckbox,
  SLCloseDialog,
  SLContextMenu,
  SLFormField,
  SLInput,
  SLModal,
  SLNotification,
  SLProgress,
  SLSelect,
  SLSpinner,
  SLSwitch,
  SLTabPanel,
  SLTabs,
  SLTextarea,
  SLToast,
  SLTooltip,
} from "./common";

import { AppHeader, AppLayout, AppSidebar } from "./layout";

const components: Record<string, ReturnType<typeof import("vue").defineComponent>> = {
  SLBadge,
  SLButton,
  SLCard,
  SLCheckbox,
  SLCloseDialog,
  SLContextMenu,
  SLFormField,
  SLInput,
  SLModal,
  SLNotification,
  SLProgress,
  SLSelect,
  SLSpinner,
  SLSwitch,
  SLTabPanel,
  SLTabs,
  SLTextarea,
  SLToast,
  SLTooltip,
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
