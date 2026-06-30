import { createRouter, createWebHistory } from "vue-router";
import pinia from "@src/stores";
import { useAuthStore } from "@stores/authStore";
import { AUTH_ROUTE_NAME, buildRedirectQuery, sanitizeRedirectPath } from "@router/authRoute";

const routes = [
  {
    path: "/auth",
    name: AUTH_ROUTE_NAME,
    component: () => import("../views/AuthEntryView.vue"),
    meta: { public: true, pageKind: "auth" },
  },
  {
    path: "/",
    name: "next-home",
    component: () => import("../views/WorkbenchView.vue"),
    meta: { requiresAuth: true, pageKind: "home" },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

const authStore = useAuthStore(pinia);
authStore.attachRouter(router);

function isAuthPreviewRoute(to: { name?: unknown; query: Record<string, unknown> }): boolean {
  return to.name === AUTH_ROUTE_NAME && to.query.preview === "1";
}

function isHomePreviewRoute(to: { name?: unknown; query: Record<string, unknown> }): boolean {
  return to.name === "next-home" && to.query.preview === "1";
}

router.beforeEach(async (to) => {
  if (to.meta.public) {
    if (isAuthPreviewRoute(to)) {
      return true;
    }

    if (to.name === AUTH_ROUTE_NAME && authStore.isAuthenticated) {
      return { path: sanitizeRedirectPath(to.query.redirect) };
    }

    return true;
  }

  if (isHomePreviewRoute(to)) {
    return true;
  }

  if (!authStore.isAuthenticated) {
    const restored = await authStore.hydrate();
    if (!restored) {
      return {
        name: AUTH_ROUTE_NAME,
        query: { redirect: sanitizeRedirectPath(buildRedirectQuery(to)) },
      };
    }
  }

  return true;
});

export default router;
