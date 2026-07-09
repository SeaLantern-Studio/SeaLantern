import {
  createRouter,
  createWebHistory,
  type RouteLocationNormalized,
  type RouteMeta,
  type RouteRecordRaw,
} from "vue-router";
import { isBrowserEnv, exchangeNextBridgeToken } from "@api/tauri";
import pinia from "@src/stores";
import { useAuthStore } from "@stores/authStore";
import { useSettingsStore } from "@stores/settingsStore";
import { AUTH_ROUTE_NAME, buildRedirectQuery, sanitizeRedirectPath } from "@router/authRoute";
import {
  createNextProtectedPageMeta,
  NEXT_AUTH_ROUTE,
  NEXT_ABOUT_ROUTE,
  NEXT_HOME_ROUTE,
  NEXT_HOME_ROUTE_NAME,
  NEXT_SERVERS_ROUTE,
  NEXT_DOWNLOADS_ROUTE,
  NEXT_TUNNEL_ROUTE,
  NEXT_PLUGINS_ROUTE,
  NEXT_PAINT_ROUTE,
  NEXT_PLUGIN_CATEGORY_ROUTE,
  NEXT_PLUGIN_CATEGORY_ROUTE_NAME,
  NEXT_PLUGIN_DETAIL_ROUTE,
  NEXT_PLUGIN_DETAIL_ROUTE_NAME,
  NEXT_PLUGIN_MARKET_ROUTE,
  NEXT_PLUGIN_MARKET_ROUTE_NAME,
  NEXT_DEVELOPER_ROUTE,
  NEXT_SETTINGS_ROUTE,
  NEXT_DOWNLOADS_ROUTE_NAME,
  NEXT_DEVELOPER_ROUTE_NAME,
  NEXT_PAINT_ROUTE_NAME,
  NEXT_SERVER_CREATE_ROUTE,
  NEXT_SERVER_IMPORT_ROUTE,
  NEXT_SETTINGS_ROUTE_NAME,
  NEXT_SERVERS_ROUTE_NAME,
  NEXT_TUNNEL_ROUTE_NAME,
  NEXT_SERVER_CREATE_ROUTE_NAME,
  NEXT_ABOUT_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
  NEXT_SERVER_IMPORT_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_CONSOLE_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_ROUTE_NAME,
  NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME,
  NEXT_LEGACY_CREATE_COMPAT_PATH,
  NEXT_LEGACY_ADD_EXISTING_COMPAT_PATH,
  NEXT_LEGACY_DOWNLOAD_COMPAT_PATH,
  NEXT_LEGACY_CONFIG_COMPAT_PATH,
  NEXT_LEGACY_PLAYERS_COMPAT_PATH,
} from "./pageMeta";

const NEXT_BRIDGE_QUERY_KEY = "next_bridge_token";
const NEXT_LEGACY_CONSOLE_COMPAT_ROUTE_NAME = "next-legacy-console-compat";
const NEXT_LEGACY_CREATE_COMPAT_ROUTE_NAME = "next-legacy-create-compat";
const NEXT_LEGACY_ADD_EXISTING_COMPAT_ROUTE_NAME = "next-legacy-add-existing-compat";
const NEXT_LEGACY_DOWNLOAD_COMPAT_ROUTE_NAME = "next-legacy-download-compat";
const NEXT_LEGACY_CONFIG_COMPAT_ROUTE_NAME = "next-legacy-config-compat";
const NEXT_LEGACY_PLAYERS_COMPAT_ROUTE_NAME = "next-legacy-players-compat";
const NEXT_PROTECTED_ROUTE_HOST_NAME = "next-protected-route-host";

const protectedChildren: RouteRecordRaw[] = [
  {
    path: NEXT_HOME_ROUTE.path,
    name: NEXT_HOME_ROUTE.name,
    component: () => import("../views/WorkbenchView.vue"),
    meta: NEXT_HOME_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_SERVERS_ROUTE.path,
    name: NEXT_SERVERS_ROUTE.name,
    component: () => import("../views/ServersView.vue"),
    meta: NEXT_SERVERS_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_DOWNLOADS_ROUTE.path,
    name: NEXT_DOWNLOADS_ROUTE_NAME,
    component: () => import("../views/DownloadsView.vue"),
    meta: NEXT_DOWNLOADS_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_TUNNEL_ROUTE.path,
    name: NEXT_TUNNEL_ROUTE_NAME,
    component: () => import("../views/TunnelView.vue"),
    meta: NEXT_TUNNEL_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PLUGINS_ROUTE.path,
    name: NEXT_PLUGINS_ROUTE.name,
    component: () => import("../views/PluginsView.vue"),
    meta: NEXT_PLUGINS_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PLUGIN_MARKET_ROUTE.path,
    name: NEXT_PLUGIN_MARKET_ROUTE_NAME,
    component: () => import("../views/MarketView.vue"),
    meta: NEXT_PLUGIN_MARKET_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PLUGIN_DETAIL_ROUTE.path,
    name: NEXT_PLUGIN_DETAIL_ROUTE_NAME,
    component: () => import("../views/PluginDetailView.vue"),
    props: true,
    meta: NEXT_PLUGIN_DETAIL_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PLUGIN_CATEGORY_ROUTE.path,
    name: NEXT_PLUGIN_CATEGORY_ROUTE_NAME,
    component: () => import("../views/PluginCategoryView.vue"),
    props: true,
    meta: NEXT_PLUGIN_CATEGORY_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PAINT_ROUTE.path,
    name: NEXT_PAINT_ROUTE_NAME,
    component: () => import("../views/PaintView.vue"),
    meta: NEXT_PAINT_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_DEVELOPER_ROUTE.path,
    name: NEXT_DEVELOPER_ROUTE_NAME,
    component: () => import("../views/DeveloperView.vue"),
    meta: NEXT_DEVELOPER_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_SETTINGS_ROUTE.path,
    name: NEXT_SETTINGS_ROUTE.name,
    component: () => import("../views/SettingsView.vue"),
    meta: NEXT_SETTINGS_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_ABOUT_ROUTE.path,
    name: NEXT_ABOUT_ROUTE_NAME,
    component: () => import("../views/AboutView.vue"),
    meta: NEXT_ABOUT_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_SERVER_CREATE_ROUTE.path,
    name: NEXT_SERVER_CREATE_ROUTE_NAME,
    component: () => import("../views/ServerCreateView.vue"),
    meta: NEXT_SERVER_CREATE_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_SERVER_IMPORT_ROUTE.path,
    name: NEXT_SERVER_IMPORT_ROUTE_NAME,
    component: () => import("../views/ServerImportView.vue"),
    meta: NEXT_SERVER_IMPORT_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: "/console/:serverId?",
    name: NEXT_LEGACY_CONSOLE_COMPAT_ROUTE_NAME,
    redirect: (to) => {
      const serverId = typeof to.params.serverId === "string" ? to.params.serverId : "";

      if (!serverId) {
        return { name: NEXT_SERVERS_ROUTE_NAME };
      }

      return {
        name: NEXT_SERVER_INSTANCE_CONSOLE_ROUTE_NAME,
        params: { serverId },
      };
    },
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.console",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: NEXT_LEGACY_CREATE_COMPAT_PATH,
    name: NEXT_LEGACY_CREATE_COMPAT_ROUTE_NAME,
    redirect: { name: NEXT_SERVER_CREATE_ROUTE_NAME },
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.create_server",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: NEXT_LEGACY_ADD_EXISTING_COMPAT_PATH,
    name: NEXT_LEGACY_ADD_EXISTING_COMPAT_ROUTE_NAME,
    redirect: { name: NEXT_SERVER_IMPORT_ROUTE_NAME },
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.add_existing_server",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: NEXT_LEGACY_DOWNLOAD_COMPAT_PATH,
    name: NEXT_LEGACY_DOWNLOAD_COMPAT_ROUTE_NAME,
    redirect: { name: NEXT_DOWNLOADS_ROUTE_NAME },
    meta: createNextProtectedPageMeta({
      pageKind: "downloads",
      titleKey: "common.download",
      navLabelKey: "shell.nav_downloads",
    }) as unknown as RouteMeta,
  },
  {
    path: NEXT_LEGACY_CONFIG_COMPAT_PATH,
    name: NEXT_LEGACY_CONFIG_COMPAT_ROUTE_NAME,
    redirect: (to) => {
      const serverId = typeof to.params.serverId === "string" ? to.params.serverId : "";

      if (!serverId) {
        return { name: NEXT_SERVERS_ROUTE_NAME };
      }

      return {
        name: NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
        params: { serverId },
      };
    },
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.config_edit",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: NEXT_LEGACY_PLAYERS_COMPAT_PATH,
    name: NEXT_LEGACY_PLAYERS_COMPAT_ROUTE_NAME,
    redirect: (to) => {
      const serverId = typeof to.params.serverId === "string" ? to.params.serverId : "";

      if (!serverId) {
        return { name: NEXT_SERVERS_ROUTE_NAME };
      }

      return {
        name: NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
        params: { serverId },
      };
    },
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.player_manage",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
];

const routes: RouteRecordRaw[] = [
  {
    path: NEXT_AUTH_ROUTE.path,
    name: NEXT_AUTH_ROUTE.name,
    component: () => import("../views/AuthEntryView.vue"),
    meta: NEXT_AUTH_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: "/",
    component: () => import("../views/NextProtectedRouteView.vue"),
    name: NEXT_PROTECTED_ROUTE_HOST_NAME,
    children: protectedChildren,
  },
  {
    path: "/servers/:serverId",
    name: NEXT_SERVER_INSTANCE_ROUTE_NAME,
    redirect: (to) => ({
      name: NEXT_SERVER_INSTANCE_CONSOLE_ROUTE_NAME,
      params: { serverId: to.params.serverId },
    }),
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.console",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: "/servers/:serverId/console",
    name: NEXT_SERVER_INSTANCE_CONSOLE_ROUTE_NAME,
    component: () => import("../views/ServerInstanceConsoleView.vue"),
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "common.console",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: "/servers/:serverId/players",
    name: NEXT_SERVER_INSTANCE_PLAYERS_ROUTE_NAME,
    component: () => import("../views/ServerInstancePlayersView.vue"),
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "servers.next.instance.sections.players",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: "/servers/:serverId/extensions",
    name: NEXT_SERVER_INSTANCE_EXTENSIONS_ROUTE_NAME,
    component: () => import("../views/ServerInstanceExtensionsView.vue"),
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "servers.next.instance.sections.extensions",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: "/servers/:serverId/config",
    name: NEXT_SERVER_INSTANCE_CONFIG_ROUTE_NAME,
    component: () => import("../views/ServerInstanceConfigView.vue"),
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "servers.next.instance.sections.config",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: "/servers/:serverId/world",
    name: NEXT_SERVER_INSTANCE_WORLD_ROUTE_NAME,
    component: () => import("../views/ServerInstanceWorldView.vue"),
    meta: createNextProtectedPageMeta({
      pageKind: "servers",
      titleKey: "servers.next.instance.sections.world",
      navLabelKey: "shell.nav_servers",
    }) as unknown as RouteMeta,
  },
  {
    path: "/:pathMatch(.*)*",
    redirect: { name: NEXT_HOME_ROUTE_NAME },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

const authStore = useAuthStore(pinia);
const settingsStore = useSettingsStore(pinia);
authStore.attachRouter(router);

function isNextPreviewRoute(to: {
  name?: unknown;
  meta: RouteMeta;
  query: Record<string, unknown>;
}): boolean {
  return typeof to.meta.pageKind === "string" && to.query.preview === "1";
}

function readBridgeTokenFromQuery(query: Record<string, unknown>): string | null {
  const rawValue = query[NEXT_BRIDGE_QUERY_KEY];
  const token = typeof rawValue === "string" ? rawValue.trim() : "";
  return token || null;
}

function buildRouteWithoutBridgeToken(to: RouteLocationNormalized) {
  const nextQuery = { ...to.query };
  delete nextQuery[NEXT_BRIDGE_QUERY_KEY];
  return {
    path: to.path,
    query: nextQuery,
    hash: to.hash,
  };
}

function buildRedirectWithoutBridgeToken(to: RouteLocationNormalized): string {
  return sanitizeRedirectPath(router.resolve(buildRouteWithoutBridgeToken(to)).fullPath);
}

router.beforeEach(async (to) => {
  if (!isBrowserEnv()) {
    if (to.meta.pageKind === "developer") {
      try {
        await settingsStore.ensureLoaded();
      } catch {
        // use current snapshot fallback below
      }

      if (!settingsStore.settings.developer_mode) {
        return {
          name: NEXT_SETTINGS_ROUTE_NAME,
          hash: "#developer-management",
        };
      }
    }

    if (to.name === AUTH_ROUTE_NAME) {
      return { path: "/" };
    }

    return true;
  }

  if (isNextPreviewRoute(to)) {
    return true;
  }

  const bridgeToken = readBridgeTokenFromQuery(to.query);
  if (bridgeToken && to.meta.authRequired === true) {
    try {
      const exchanged = await exchangeNextBridgeToken(bridgeToken);
      const accepted = await authStore.acceptSession(exchanged, false);

      if (accepted) {
        return buildRouteWithoutBridgeToken(to);
      }
    } catch {
      // fall through to existing auth fallback below
    }
  }

  if (to.meta.public) {
    if (to.name === AUTH_ROUTE_NAME && authStore.isAuthenticated) {
      return { path: sanitizeRedirectPath(to.query.redirect) };
    }

    return true;
  }

  if (to.meta.authRequired !== true) {
    return true;
  }

  if (!authStore.isAuthenticated) {
    const restored = await authStore.hydrate();
    if (!restored) {
      const redirect = bridgeToken
        ? buildRedirectWithoutBridgeToken(to)
        : sanitizeRedirectPath(buildRedirectQuery(to));
      return {
        name: AUTH_ROUTE_NAME,
        query: { redirect },
      };
    }
  }

  if (to.meta.pageKind === "developer") {
    try {
      await settingsStore.ensureLoaded();
    } catch {
      // keep fallback decision below based on current store snapshot
    }

    if (!settingsStore.settings.developer_mode) {
      return {
        name: NEXT_SETTINGS_ROUTE_NAME,
        hash: "#developer-management",
      };
    }
  }

  return true;
});

export default router;
