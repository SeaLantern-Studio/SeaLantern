import { createRouter, createWebHistory } from "vue-router";
import { onPageChanged } from "@api/plugin";

const routes = [
  {
    path: "/",
    name: "home",
    component: () => import("@views/HomeView.vue"),
    meta: { title: "首页", icon: "home" },
  },
  {
    path: "/create",
    name: "create-server",
    component: () => import("@views/CreateServerView.vue"),
    meta: { title: "创建服务器", icon: "plus" },
  },
  {
    path: "/console/:id?",
    name: "console",
    component: () => import("@views/ConsoleView.vue"),
    meta: { title: "控制台", icon: "terminal" },
  },
  {
    path: "/config/:id?",
    name: "config",
    component: () => import("@views/ConfigView.vue"),
    meta: { title: "配置编辑", icon: "settings" },
  },
  {
    path: "/players/:id?",
    name: "players",
    component: () => import("@views/PlayerView.vue"),
    meta: { title: "玩家管理", icon: "users" },
  },
  {
    path: "/plugins",
    name: "plugins",
    component: () => import("@views/PluginsPageView.vue"),
    meta: { title: "插件", icon: "puzzle" },
  },
  {
    path: "/market",
    redirect: "/plugins?tab=market",
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@views/SettingsView.vue"),
    meta: { title: "设置", icon: "sliders" },
  },
  {
    path: "/paint",
    name: "paint",
    component: () => import("@views/PaintView.vue"),
    meta: { title: "个性化", icon: "palette" },
  },
  {
    path: "/about",
    name: "about",
    component: () => import("@views/AboutView.vue"),
    meta: { title: "关于", icon: "info" },
  },
  {
    path: "/plugin/:pluginId",
    name: "plugin-page",
    component: () => import("@views/PluginPageView.vue"),
    props: true,
    meta: { title: "插件设置", icon: "puzzle" },
  },
  {
    path: "/plugin-category/:pluginId",
    name: "plugin-category",
    component: () => import("@views/PluginCategoryView.vue"),
    props: true,
    meta: { title: "插件分类", icon: "folder" },
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
