import type { App } from "vue";

export * from "@components/common";

export * from "@components/layout";

export * from "@components/plugin";

export type { TabItem } from "@components/common/SLTabs.vue";

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
} from "@components/common";

import { AppHeader, AppLayout, AppSidebar } from "@components/layout";

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
