<script setup lang="ts">
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "../../stores/uiStore";

const router = useRouter();
const route = useRoute();
const ui = useUiStore();

interface NavItem {
  name: string;
  path: string;
  icon: string;
  label: string;
  group: string;
}

const navItems: NavItem[] = [
  { name: "home", path: "/", icon: "home", label: "首页", group: "main" },
  { name: "create", path: "/create", icon: "plus", label: "创建服务器", group: "main" },
  { name: "console", path: "/console", icon: "terminal", label: "控制台", group: "server" },
  { name: "config", path: "/config", icon: "settings", label: "配置编辑", group: "server" },
  { name: "players", path: "/players", icon: "users", label: "玩家管理", group: "server" },
  { name: "settings", path: "/settings", icon: "sliders", label: "设置", group: "system" },
  { name: "about", path: "/about", icon: "info", label: "关于", group: "system" },
];

const groups = [
  { key: "main", label: "通用" },
  { key: "server", label: "服务器" },
  { key: "system", label: "系统" },
];

function navigateTo(path: string) {
  router.push(path);
}

function isActive(path: string): boolean {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
}

const iconMap: Record<string, string> = {
  home: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-4 0a1 1 0 01-1-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 01-1 1",
  plus: "M12 4v16m8-8H4",
  terminal: "M4 17l6-6-6-6m8 14h8",
  settings:
    "M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4",
  users:
    "M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z",
  sliders:
    "M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4",
  info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
  chevron: "M15 19l-7-7 7-7",
};
</script>

<template>
  <aside class="sidebar glass-strong" :class="{ collapsed: ui.sidebarCollapsed }">
    <div class="sidebar-logo" @click="navigateTo('/')">
      <div class="logo-icon">
        <img src="../../assets/logo.svg" alt="Sea Lantern" width="28" height="28" />
      </div>
      <transition name="fade">
        <span v-if="!ui.sidebarCollapsed" class="logo-text">Sea Lantern</span>
      </transition>
    </div>

    <nav class="sidebar-nav">
      <div v-for="group in groups" :key="group.key" class="nav-group">
        <transition name="fade">
          <div v-if="!ui.sidebarCollapsed" class="nav-group-label">{{ group.label }}</div>
        </transition>
        <div
          v-for="item in navItems.filter((i) => i.group === group.key)"
          :key="item.name"
          class="nav-item"
          :class="{ active: isActive(item.path) }"
          @click="navigateTo(item.path)"
          :title="ui.sidebarCollapsed ? item.label : ''"
        >
          <svg
            class="nav-icon"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path :d="iconMap[item.icon] || iconMap.info" />
          </svg>
          <transition name="fade">
            <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ item.label }}</span>
          </transition>
          <div v-if="isActive(item.path)" class="nav-active-indicator" />
        </div>
      </div>
    </nav>

    <div class="sidebar-footer">
      <div class="nav-item collapse-btn" @click="ui.toggleSidebar()">
        <svg
          class="nav-icon"
          :style="{ transform: ui.sidebarCollapsed ? 'rotate(180deg)' : '' }"
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path :d="iconMap.chevron" />
        </svg>
        <transition name="fade">
          <span v-if="!ui.sidebarCollapsed" class="nav-label">收起侧栏</span>
        </transition>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  position: fixed;
  top: 0;
  left: 0;
  width: var(--sl-sidebar-width);
  height: 100vh;
  display: flex;
  flex-direction: column;
  z-index: 100;
  border-right: 1px solid var(--sl-border-light);
  transition: width var(--sl-transition-normal);
  overflow: hidden;
}

.sidebar.collapsed {
  width: var(--sl-sidebar-collapsed-width);
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: var(--sl-space-md);
  height: 60px;
  cursor: pointer;
  flex-shrink: 0;
}

.logo-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.logo-text {
  font-size: 1.125rem;
  font-weight: 700;
  white-space: nowrap;
  letter-spacing: -0.01em;
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--sl-space-sm);
}

.nav-group {
  margin-bottom: var(--sl-space-sm);
}

.nav-group-label {
  padding: var(--sl-space-xs) var(--sl-space-sm);
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--sl-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  white-space: nowrap;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm);
  padding: 8px 12px;
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  position: relative;
  white-space: nowrap;
  margin-top: 5px;
}

.nav-item:hover {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.nav-item.active {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-weight: 500;
}

.nav-active-indicator {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 16px;
  background-color: var(--sl-primary);
  border-radius: var(--sl-radius-full);
}

.nav-icon {
  flex-shrink: 0;
  transition: transform var(--sl-transition-normal);
}

.nav-label {
  font-size: 0.875rem;
  white-space: nowrap;
}

.sidebar-footer {
  flex-shrink: 0;
  padding: var(--sl-space-sm);
  border-top: 1px solid var(--sl-border-light);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
