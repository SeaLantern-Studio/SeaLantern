<script setup lang="ts">
import { computed } from "vue";
import { Download, House, Info, Link2, Palette, Puzzle, Server, Settings2, Wrench } from "@lucide/vue";
import { isBrowserEnv } from "@api/tauri";
import { i18n } from "@language";
import { AUTH_ROUTE_NAME } from "@router/authRoute";
import { useAuthStore } from "@stores/authStore";
import { useSettingsStore } from "@stores/settingsStore";
import { RouterView, useRoute, useRouter } from "vue-router";
import NextSidebarHostItems from "@src/components/host/NextSidebarHostItems.vue";
import type { NextProtectedPageKind, NextShellPage } from "@src/contracts/page";
import type { NextShellNavItem } from "@src/contracts/shell";
import { useNextShellNavigationTransition } from "@src/composables/useNextShellNavigationTransition";
import { provideNextProtectedShellController } from "@src/composables/useNextProtectedShell";
import { useNextHostRuntime } from "@src/host/runtime";
import NextWorkbenchLayout from "@src/layout/NextWorkbenchLayout.vue";
import { getNextRoutePageMeta, NEXT_PROTECTED_ROUTES } from "@src/router/pageMeta";

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const settingsStore = useSettingsStore();
const nextHostRuntime = useNextHostRuntime();
const shellController = provideNextProtectedShellController();
const isBrowserMode = isBrowserEnv();

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
} = useNextShellNavigationTransition(currentPageKind);

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
const effectiveRailExpanded = computed(
  () => !shellController.renderState.value.railLocked && railExpanded.value,
);
const headerPrimaryActionsRenderer = computed(() => {
  const render = shellController.headerPrimaryActions.value;
  return render ? { render } : null;
});
const shellPage = computed<NextShellPage>(() => ({
  kind: currentPageKind.value,
  title: i18n.t(pageMeta.value.titleKey),
  subtitle: "",
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
    :rail-locked="shellController.renderState.value.railLocked"
    :rail-expanded="effectiveRailExpanded"
    :page-transition-class="pageTransitionClass"
    :shell-header-mode="shellController.renderState.value.shellHeaderMode"
    @logout="handleLogout"
    @page-transition-settled="handlePageTransitionSettled"
    @rail-focus-within-change="setFocusWithinRail"
    @rail-pointer-inside-change="setPointerInsideRail"
  >
    <template #sidebar-primary>
      <NextSidebarHostItems :items="hostSidebarItems" />
    </template>

    <template #header-primary-actions>
      <component :is="headerPrimaryActionsRenderer" v-if="headerPrimaryActionsRenderer !== null" />
    </template>

    <RouterView />
  </NextWorkbenchLayout>
</template>
