<script setup lang="ts">
import { computed, useSlots } from "vue";
import { Download, House, Info, Link2, Palette, Puzzle, Server, Settings2, Wrench } from "@lucide/vue";
import { isBrowserEnv } from "@api/tauri";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import { AUTH_ROUTE_NAME } from "@router/authRoute";
import { useAuthStore } from "@stores/authStore";
import { useSettingsStore } from "@stores/settingsStore";
import NextSidebarHostItems from "../components/host/NextSidebarHostItems.vue";
import type { NextProtectedPageKind, NextShellPage } from "../contracts/page";
import type { NextShellNavItem } from "../contracts/shell";
import { useNextShellNavigationTransition } from "../composables/useNextShellNavigationTransition";
import { useNextHostRuntime } from "../host/runtime";
import NextWorkbenchLayout from "../layout/NextWorkbenchLayout.vue";
import { getNextRoutePageMeta, NEXT_PROTECTED_ROUTES } from "../router/pageMeta";

interface Props {
  pageSubtitle?: string;
  railLocked?: boolean;
  shellHeaderMode?: "title" | "hidden";
}

const props = withDefaults(defineProps<Props>(), {
  pageSubtitle: "",
  railLocked: false,
  shellHeaderMode: "title",
});

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const settingsStore = useSettingsStore();
const nextHostRuntime = useNextHostRuntime();
const isBrowserMode = isBrowserEnv();
const slots = useSlots();

const pageMeta = computed(() => getNextRoutePageMeta(route.meta as Record<string, unknown>));
const currentPageKind = computed<NextProtectedPageKind>(() => {
  return pageMeta.value.pageKind === "auth" ? "home" : pageMeta.value.pageKind;
});
const {
  handlePageTransitionSettled,
  pageTransitionClass,
  railExpanded,
  setFocusWithinRail,
  setPointerInsideRail,
} = useNextShellNavigationTransition(currentPageKind.value);
const effectiveRailExpanded = computed(() => !props.railLocked && railExpanded.value);

const navIconByKind = {
  home: House,
  servers: Server,
  downloads: Download,
  tunnel: Link2,
  plugins: Puzzle,
  paint: Palette,
  developer: Wrench,
  settings: Settings2,
  about: Info,
} as const;

const hostSidebarItems = computed(() => nextHostRuntime.sidebarItems.value);
const shellPage = computed<NextShellPage>(() => ({
  kind: currentPageKind.value,
  title: i18n.t(pageMeta.value.titleKey),
  subtitle: props.pageSubtitle ?? "",
}));
const navItems = computed<NextShellNavItem[]>(() =>
  NEXT_PROTECTED_ROUTES.filter((page) => {
    return page.meta.pageKind !== "developer" || settingsStore.settings.developer_mode;
  }).map((page) => ({
    id: page.meta.pageKind,
    label: i18n.t(page.meta.navLabelKey ?? page.meta.titleKey),
    icon: navIconByKind[page.meta.pageKind],
    to: { name: page.name },
    active: page.meta.pageKind === currentPageKind.value,
    disabled: false,
  })),
);

async function handleLogout(): Promise<void> {
  if (!isBrowserMode) {
    await router.replace({ path: "/" });
    return;
  }

  authStore.logout();
  await router.replace({ name: AUTH_ROUTE_NAME });
}
</script>

<template>
  <NextWorkbenchLayout
    brand="SeaLantern"
    :rail-label="i18n.t('shell.rail_label')"
    :page="shellPage"
    :logout-label="i18n.t('shell.logout')"
    :show-logout="isBrowserMode"
    :nav-items="navItems"
    :rail-locked="props.railLocked"
    :rail-expanded="effectiveRailExpanded"
    :page-transition-class="pageTransitionClass"
    :shell-header-mode="props.shellHeaderMode"
    @logout="handleLogout"
    @page-transition-settled="handlePageTransitionSettled"
    @rail-focus-within-change="setFocusWithinRail"
    @rail-pointer-inside-change="setPointerInsideRail"
  >
    <template #sidebar-primary>
      <NextSidebarHostItems :items="hostSidebarItems" />
    </template>

    <template #header-primary-actions>
      <slot v-if="slots['header-primary-actions']" name="header-primary-actions" />
    </template>

    <template v-if="slots['page-header-actions']" #page-header-actions>
      <div class="next-protected-page-view__page-header-actions">
        <slot name="page-header-actions" />
      </div>
    </template>

    <slot />
  </NextWorkbenchLayout>
</template>

<style scoped>
.next-protected-page-view__page-header-actions {
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 10px;
}

@media (max-width: 767px) {
  .next-protected-page-view__page-header-actions {
    justify-content: stretch;
  }
}
</style>
