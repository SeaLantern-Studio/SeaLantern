import { createRouter, createWebHistory } from "vue-router";
import { onPageChanged } from "@api/plugin";

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
    path: "/plugins",
    name: "plugins",
    component: () => import("@views/PluginsPageView.vue"),
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
    path: "/download-file",
    name: "download-file",
    component: () => import("../views/DownloadFileView.vue"),
    meta: { titleKey: "common.download-file", icon: "info" },
  },
];
const router = createRouter({
  history: createWebHistory(),
  routes,
});

let pageChangedTimers: number[] = [];

router.afterEach((to) => {
  for (const t of pageChangedTimers) {
    clearTimeout(t);
  }
  pageChangedTimers = [];

  pageChangedTimers.push(
    window.setTimeout(() => {
      onPageChanged(to.path).catch(() => {});
    }, 250),
  );
  pageChangedTimers.push(
    window.setTimeout(() => {
      onPageChanged(to.path).catch(() => {});
    }, 900),
  );
});

export default router;
