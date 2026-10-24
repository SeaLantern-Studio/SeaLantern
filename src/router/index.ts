import { createRouter, createWebHistory } from "vue-router";

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
    path: "/resource-market",
    name: "resource-market",
    component: () => import("@views/ResourceMarketView.vue"),
    meta: { titleKey: "common.resource_market", icon: "store" },
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
  // 个性化已并入设置页,旧地址跳转保留
  {
    path: "/paint",
    redirect: "/settings",
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
  {
    path: "/backup/:id?",
    name: "backup",
    component: () => import("@views/BackupView.vue"),
    meta: { titleKey: "common.backup", icon: "archive" },
  },
  {
    path: "/help",
    name: "help",
    component: () => import("@views/HelpView.vue"),
    meta: { titleKey: "common.help", icon: "book" },
  },
  // 开发者测试工具:仅在开发者模式开启时侧栏展示,路由本身始终注册
  {
    path: "/dev-test",
    name: "dev-test",
    component: () => import("@views/DevTestView.vue"),
    meta: { titleKey: "common.dev_test", icon: "beaker" },
  },
  // 404 兜底:无效路径统一回到首页
  {
    path: "/:pathMatch(.*)*",
    name: "not-found",
    redirect: "/",
  },
];
const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
