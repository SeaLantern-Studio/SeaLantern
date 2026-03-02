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
  SLContextMenu,
  SLFormField,
  SLInput,
  SLModal,
  SLProgress,
  SLSelect,
  SLSpinner,
  SLSwitch,
  SLTabs,
  SLTextarea,
} from "@components/common";

import { AppHeader, AppLayout, AppSidebar } from "@components/layout";

const components: Record<string, ReturnType<typeof import("vue").defineComponent>> = {
  SLBadge,
  SLButton,
  SLCard,
  SLCheckbox,
  SLContextMenu,
  SLFormField,
  SLInput,
  SLModal,
  SLProgress,
  SLSelect,
  SLSpinner,
  SLSwitch,
  SLTabs,
  SLTextarea,
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
