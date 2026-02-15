<script setup lang="ts">
import { computed, ref, nextTick, watch, onMounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "../../stores/uiStore";
import { useServerStore } from "../../stores/serverStore";
import { i18n } from "../../locales";
import SLSelect from "../../components/common/SLSelect.vue";

const router = useRouter();
const route = useRoute();
const ui = useUiStore();
const serverStore = useServerStore();
const navIndicator = ref<HTMLElement | null>(null);
const showServerBubble = ref(false);

interface NavItem {
  name: string;
  path: string;
  icon: string;
  labelKey: string;
  group: string;
}

const navItems: NavItem[] = [
  { name: "home", path: "/", icon: "home", labelKey: "common.home", group: "main" },
  {
    name: "create",
    path: "/create",
    icon: "plus",
    labelKey: "common.create_server",
    group: "main",
  },
  {
    name: "console",
    path: "/console",
    icon: "terminal",
    labelKey: "common.console",
    group: "server",
  },
  {
    name: "config",
    path: "/config",
    icon: "settings",
    labelKey: "common.config_edit",
    group: "server",
  },
  {
    name: "players",
    path: "/players",
    icon: "users",
    labelKey: "common.player_manage",
    group: "server",
  },
  {
    name: "settings",
    path: "/settings",
    icon: "sliders",
    labelKey: "common.settings",
    group: "system",
  },
  { name: "about", path: "/about", icon: "info", labelKey: "common.about", group: "system" },
];

const groups = [
  { key: "main", labelKey: "sidebar.groups.main" },
  { key: "server", labelKey: "sidebar.groups.server" },
  { key: "system", labelKey: "sidebar.groups.system" },
];

function navigateTo(path: string) {
  router.push(path);
}

// 切换服务器选择气泡的显示/隐藏
function toggleServerBubble() {
  showServerBubble.value = !showServerBubble.value;
}

// 选择服务器并关闭气泡
function selectServer(serverId: string) {
  handleServerChange(serverId);
  showServerBubble.value = false;
}

// 更新导航指示器位置
function updateNavIndicator() {
  nextTick(() => {
    if (!navIndicator.value) return;
    
    const activeNavItem = document.querySelector('.nav-item.active');
    if (activeNavItem) {
      const { offsetTop, offsetHeight } = activeNavItem as HTMLElement;
      navIndicator.value.style.top = `${offsetTop + (offsetHeight - 16) / 2}px`;
    }
  });
}

// 监听侧边栏折叠状态变化，更新指示器位置
watch(() => ui.sidebarCollapsed, () => {
  // 延迟更新，确保动画完成后再计算位置
  setTimeout(() => {
    updateNavIndicator();
  }, 300); // 等待300ms，确保CSS过渡动画完成
});

// 监听路由变化，更新指示器位置
watch(() => route.path, () => {
  updateNavIndicator();
});

// 组件挂载后初始化指示器位置
onMounted(() => {
  updateNavIndicator();
  
  // 添加全局点击事件监听器，点击外部关闭气泡
  document.addEventListener('click', handleClickOutside);
});

// 点击外部关闭服务器选择气泡
function handleClickOutside(event: MouseEvent) {
  if (showServerBubble.value) {
    const bubble = document.querySelector('.server-select-bubble');
    const trigger = document.querySelector('.server-selector-icon');
    
    if (bubble && trigger) {
      const bubbleRect = bubble.getBoundingClientRect();
      const triggerRect = trigger.getBoundingClientRect();
      
      // 检查点击是否在气泡或触发按钮之外
      const clickedInsideBubble = event.clientX >= bubbleRect.left && 
                                event.clientX <= bubbleRect.right && 
                                event.clientY >= bubbleRect.top && 
                                event.clientY <= bubbleRect.bottom;
      
      const clickedInsideTrigger = event.clientX >= triggerRect.left && 
                                 event.clientX <= triggerRect.right && 
                                 event.clientY >= triggerRect.top && 
                                 event.clientY <= triggerRect.bottom;
      
      if (!clickedInsideBubble && !clickedInsideTrigger) {
        showServerBubble.value = false;
      }
    }
  }
}

function handleServerChange(value: string) {
  serverStore.setCurrentServer(value);
  // 如果当前在服务器相关页面，更新路由
  if (
    route.path.startsWith("/console") ||
    route.path.startsWith("/config") ||
    route.path.startsWith("/players")
  ) {
    const currentPath = route.path.split("/")[1];
    router.push(`/${currentPath}/${value}`);
  }
}

// 服务器选项
const serverOptions = computed(() => {
  return serverStore.servers.map((s) => ({
    label: s.name + " (" + s.id.substring(0, 8) + ")",
    value: s.id,
  }));
});

// 当前选中的服务器
const currentServerId = computed(() => {
  return serverStore.currentServerId;
});

function isActive(path: string): boolean {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
}

const iconMap: Record<string, string> = {
  home: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-4 0a1 1 0 01-1-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 01-1 1h-3m-4 0a1 1 0 01-1-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 01-1 1H5z",
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
        <img src="../../assets/logo.svg" :alt="i18n.t('common.app_name')" width="28" height="28" />
      </div>
      <transition name="fade">
        <span v-if="!ui.sidebarCollapsed" class="logo-text">{{ i18n.t("common.app_name") }}</span>
      </transition>
    </div>

    <nav class="sidebar-nav">
      <!-- 服务器选择 -->
      <div v-if="serverOptions.length > 0" class="server-selector">
        <template v-if="!ui.sidebarCollapsed">
          <div class="server-selector-label">服务器</div>
          <SLSelect
            :options="serverOptions"
            :modelValue="currentServerId"
            @update:modelValue="handleServerChange"
            :placeholder="i18n.t('common.select_server')"
            size="sm"
          />
        </template>
        <template v-else>
          <div class="server-selector-icon" @click="toggleServerBubble">
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <!-- 服务器图标（与侧边栏其他图标风格一致） -->
              <rect x="4" y="6" width="10" height="12" rx="1" />
              <rect x="12" y="10" width="6" height="8" rx="1" />
              <line x1="6" y1="10" x2="10" y2="10" />
              <line x1="6" y1="14" x2="10" y2="14" />
              <line x1="14" y1="12" x2="16" y2="12" />
              <line x1="14" y1="16" x2="16" y2="16" />
            </svg>
          </div>
        </template>
      </div>

      <!-- 导航激活指示器 -->
      <div class="nav-active-indicator" ref="navIndicator"></div>

      <div v-for="group in groups" :key="group.key" class="nav-group">
        <div class="nav-group-label"></div>
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
            <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t(item.labelKey) }}</span>
          </transition>
        </div>
      </div>
    </nav>

    <!-- 弹出的服务器选择气泡 -->
    <Transition name="bubble">
      <div v-if="showServerBubble && ui.sidebarCollapsed" class="server-select-bubble" @click.stop>
        <div class="server-select-bubble-content">
          <div class="server-select-bubble-header">
            <h3>选择服务器</h3>
          </div>
          <div class="server-select-bubble-body">
            <div 
              v-for="option in serverOptions" 
              :key="option.value"
              class="server-select-option"
              :class="{ active: option.value === currentServerId }"
              @click="selectServer(option.value)"
            >
              {{ option.label }}
            </div>
          </div>

        </div>
      </div>
    </Transition>

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
          <span v-if="!ui.sidebarCollapsed" class="nav-label">{{
            i18n.t("sidebar.collapse_btn")
          }}</span>
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
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
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
  position: relative;
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
  border-bottom: 1px solid var(--sl-border);
  margin-bottom: var(--sl-space-xs);
}

/* 服务器选择器样式 */
.server-selector {
  padding: var(--sl-space-sm);
  margin-bottom: var(--sl-space-sm);
  display: flex;
  align-items: center;
  justify-content: center;
}

.server-selector-label {
  padding: var(--sl-space-xs) var(--sl-space-sm);
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--sl-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  white-space: nowrap;
  margin-bottom: var(--sl-space-xs);
}

.server-selector :deep(.sl-select) {
  width: 100%;
}

.server-selector :deep(.sl-select__input) {
  font-size: 0.8125rem;
}

.server-selector-icon {
  padding: 8px;
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
}

.server-selector-icon:hover {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

/* 弹出的服务器选择气泡 */
.server-select-bubble {
  position: fixed;
  left: var(--sl-sidebar-collapsed-width, 60px);
  top: 60px;
  z-index: 9999;
  pointer-events: none;
}

.server-select-bubble-content {
  pointer-events: auto;
}

/* 气泡动画 */
.bubble-enter-active,
.bubble-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.bubble-enter-from {
  opacity: 0;
  transform: translateX(-10px) scale(0.95);
}

.bubble-leave-to {
  opacity: 0;
  transform: translateX(-10px) scale(0.9);
}

.bubble-enter-to,
.bubble-leave-from {
  opacity: 1;
  transform: translateX(0) scale(1);
}

.server-select-bubble-content {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: var(--sl-radius-lg);
  padding: var(--sl-space-lg);
  width: 300px;
  box-shadow: var(--sl-shadow-lg);
  position: relative;
}

.server-select-bubble-header h3 {
  color: #1f2937;
}

.server-select-option {
  color: #4b5563;
}

.server-select-option:hover {
  background-color: #f3f4f6;
  color: #1f2937;
}

.server-select-option.active {
  background-color: #e0f2fe;
  color: #0284c7;
}

.bubble-close {
  color: #6b7280;
}

.bubble-close:hover {
  color: #1f2937;
}

.server-select-bubble-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--sl-space-md);
}

.server-select-bubble-header h3 {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0;
}

.bubble-close {
  background: none;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  color: var(--sl-text-tertiary);
  transition: color var(--sl-transition-fast);
}

.bubble-close:hover {
  color: var(--sl-text-primary);
}

.server-select-bubble-body {
  max-height: 300px;
  overflow-y: auto;
}



.server-select-option {
  padding: 10px 14px;
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  margin-bottom: 4px;
}

.server-select-option:hover {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.server-select-option.active {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
  font-weight: 500;
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
  top: 0;
  width: 3px;
  height: 16px;
  background-color: var(--sl-primary);
  border-radius: var(--sl-radius-full);
  transition: top 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 10;
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
