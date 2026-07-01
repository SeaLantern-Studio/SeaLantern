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
import { AUTH_ROUTE_NAME, buildRedirectQuery, sanitizeRedirectPath } from "@router/authRoute";
import { NEXT_AUTH_ROUTE, NEXT_PROTECTED_ROUTES } from "./pageMeta";

const NEXT_BRIDGE_QUERY_KEY = "next_bridge_token";

const routes: RouteRecordRaw[] = [
  {
    path: NEXT_AUTH_ROUTE.path,
    name: NEXT_AUTH_ROUTE.name,
    component: () => import("../views/AuthEntryView.vue"),
    meta: NEXT_AUTH_ROUTE.meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PROTECTED_ROUTES[0].path,
    name: NEXT_PROTECTED_ROUTES[0].name,
    component: () => import("../views/WorkbenchView.vue"),
    meta: NEXT_PROTECTED_ROUTES[0].meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PROTECTED_ROUTES[1].path,
    name: NEXT_PROTECTED_ROUTES[1].name,
    component: () => import("../views/ServersView.vue"),
    meta: NEXT_PROTECTED_ROUTES[1].meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PROTECTED_ROUTES[2].path,
    name: NEXT_PROTECTED_ROUTES[2].name,
    component: () => import("../views/PluginsView.vue"),
    meta: NEXT_PROTECTED_ROUTES[2].meta as unknown as RouteMeta,
  },
  {
    path: NEXT_PROTECTED_ROUTES[3].path,
    name: NEXT_PROTECTED_ROUTES[3].name,
    component: () => import("../views/SettingsView.vue"),
    meta: NEXT_PROTECTED_ROUTES[3].meta as unknown as RouteMeta,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

const authStore = useAuthStore(pinia);
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
      const accepted = await authStore.login(exchanged.token, false);

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

  return true;
});

export default router;
