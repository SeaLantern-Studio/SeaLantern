<script setup lang="ts">
import { computed } from "vue";
import { House, Puzzle, Server, Settings2 } from "@lucide/vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import { AUTH_ROUTE_NAME } from "@router/authRoute";
import { useAuthStore } from "@stores/authStore";
import NextSidebarHostItems from "../components/host/NextSidebarHostItems.vue";
import type { NextProtectedPageKind, NextShellPage } from "../contracts/page";
import type { NextShellNavItem } from "../contracts/shell";
import { useNextHostRuntime } from "../host/runtime";
import NextWorkbenchLayout from "../layout/NextWorkbenchLayout.vue";
import { getNextRoutePageMeta, NEXT_PROTECTED_ROUTES } from "../router/pageMeta";

interface Props {
  pageSubtitle?: string;
  railLocked?: boolean;
}

const props = defineProps<Props>();

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const nextHostRuntime = useNextHostRuntime();

const pageMeta = computed(() => getNextRoutePageMeta(route.meta as Record<string, unknown>));
const currentPageKind = computed<NextProtectedPageKind>(() => {
  return pageMeta.value.pageKind === "auth" ? "home" : pageMeta.value.pageKind;
});

const navIconByKind = {
  home: House,
  servers: Server,
  plugins: Puzzle,
  settings: Settings2,
} as const;

const hostSidebarItems = computed(() => nextHostRuntime.sidebarItems.value);
const shellPage = computed<NextShellPage>(() => ({
  kind: currentPageKind.value,
  title: i18n.t(pageMeta.value.titleKey),
  subtitle: props.pageSubtitle ?? "",
}));
const navItems = computed<NextShellNavItem[]>(() =>
  NEXT_PROTECTED_ROUTES.map((page) => ({
    id: page.meta.pageKind,
    label: i18n.t(page.meta.navLabelKey ?? page.meta.titleKey),
    icon: navIconByKind[page.meta.pageKind],
    to: { name: page.name },
    active: page.meta.pageKind === currentPageKind.value,
    disabled: false,
  })),
);

async function handleLogout(): Promise<void> {
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
    :nav-items="navItems"
    :rail-locked="props.railLocked"
    @logout="handleLogout"
  >
    <template #sidebar-primary>
      <NextSidebarHostItems :items="hostSidebarItems" />
    </template>

    <template #header-primary-actions>
      <slot name="header-primary-actions" />
    </template>

    <template #page-header-actions>
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
