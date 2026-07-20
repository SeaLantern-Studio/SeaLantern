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

// 路由切换后通知插件页面变化;原实现注册 250ms/900ms 两个定时器冗余触发,改为单次延时
let pageChangedTimer: number | null = null;

router.afterEach((to) => {
  if (pageChangedTimer !== null) {
    clearTimeout(pageChangedTimer);
  }
  // 300ms 等待页面渲染稳定后再通知,避免插件在过渡动画期间收到事件
  pageChangedTimer = window.setTimeout(() => {
    pageChangedTimer = null;
    onPageChanged(to.path).catch(() => {});
  }, 300);
});

export default router;
