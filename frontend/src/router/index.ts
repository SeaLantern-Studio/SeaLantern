import { createRouter, createWebHistory } from "vue-router";
import { checkDeveloperMode } from "@api/logging";
import { notifyPluginPageLifecycle } from "@router/pluginPageLifecycle";

const routes = [
  {
    path: "/",
    name: "home",
    component: () => import("@views/HomeView.vue"),
    meta: { titleKey: "common.home", icon: "home" },
  },
  {
    path: "/create",
    name: "create-server",
    component: () => import("@views/CreateServerView.vue"),
    meta: { titleKey: "common.create_server", icon: "plus" },
  },
  {
    path: "/add-existing",
    name: "add-existing-server",
    component: () => import("@views/AddExistingServerView.vue"),
    meta: { titleKey: "common.add_existing_server", icon: "server-plus" },
  },
  {
    path: "/console/:id?",
    name: "console",
    component: () => import("@views/ConsoleView.vue"),
    meta: { titleKey: "common.console", icon: "terminal" },
  },
  {
    path: "/config/:id?",
    name: "config",
    component: () => import("@views/ConfigView.vue"),
    meta: { titleKey: "common.config_edit", icon: "settings" },
  },
  {
    path: "/players/:id?",
    name: "players",
    component: () => import("@views/PlayerView.vue"),
    meta: { titleKey: "common.player_manage", icon: "users" },
  },
  {
    path: "/tunnel",
    name: "tunnel",
    component: () => import("@views/TunnelView.vue"),
    meta: { titleKey: "common.tunnel", icon: "link2" },
  },
  {
    path: "/plugins",
    name: "plugins",
    component: () => import("@views/PluginsView.vue"),
    meta: { titleKey: "common.plugins", icon: "puzzle" },
  },
  {
    path: "/market",
    redirect: "/plugins?tab=market",
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@views/SettingsView.vue"),
    meta: { titleKey: "common.settings", icon: "sliders" },
  },
  {
    path: "/developer",
    name: "developer",
    component: () => import("@views/DeveloperView.vue"),
    meta: { titleKey: "common.developer_tools", icon: "sparkles" },
  },
  {
    path: "/paint",
    name: "paint",
    component: () => import("@views/PaintView.vue"),
    meta: { titleKey: "common.personalize", icon: "palette" },
  },
  {
    path: "/about",
    name: "about",
    component: () => import("@views/AboutView.vue"),
    meta: { titleKey: "common.about", icon: "info" },
  },
  {
    path: "/plugin/:pluginId",
    name: "plugin-page",
    component: () => import("@views/PluginPageView.vue"),
    props: true,
    meta: { titleKey: "plugins.plugin_settings", icon: "puzzle" },
  },
  {
    path: "/plugin-category/:pluginId",
    name: "plugin-category",
    component: () => import("@views/PluginCategoryView.vue"),
    props: true,
    meta: { titleKey: "plugins.plugin_category", icon: "folder" },
  },
  {
    path: "/download",
    name: "download",
    component: () => import("../views/DownloadView.vue"),
    meta: { titleKey: "common.download", icon: "download" },
  },
];
const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.beforeEach(async (to) => {
  if (to.name !== "developer") {
    return true;
  }

  const developerEnabled = await checkDeveloperMode().catch(() => false);
  if (!developerEnabled) {
    return { path: "/settings" };
  }

  return true;
});

router.afterEach((to) => {
  notifyPluginPageLifecycle(to.fullPath);
});

export default router;
