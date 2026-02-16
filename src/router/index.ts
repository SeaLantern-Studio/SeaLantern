import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "home",
    component: () => import("../views/HomeView.vue"),
    meta: { titleKey: "common.home", icon: "home" },
  },
  {
    path: "/create",
    name: "create-server",
    component: () => import("../views/CreateServerView.vue"),
    meta: { titleKey: "common.create_server", icon: "plus" },
  },
  {
    path: "/console/:id?",
    name: "console",
    component: () => import("../views/ConsoleView.vue"),
    meta: { titleKey: "common.console", icon: "terminal" },
  },
  {
    path: "/config/:id?",
    name: "config",
    component: () => import("../views/ConfigView.vue"),
    meta: { titleKey: "common.config_edit", icon: "settings" },
  },
  {
    path: "/players/:id?",
    name: "players",
    component: () => import("../views/PlayerView.vue"),
    meta: { titleKey: "common.player_manage", icon: "users" },
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("../views/SettingsView.vue"),
    meta: { titleKey: "common.settings", icon: "sliders" },
  },
  {
    path: "/paint",
    name: "paint",
    component: () => import("../views/PaintView.vue"),
    meta: { titleKey: "common.personalize", icon: "paint" },
  },
  {
    path: "/about",
    name: "about",
    component: () => import("../views/AboutView.vue"),
    meta: { titleKey: "common.about", icon: "info" },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
