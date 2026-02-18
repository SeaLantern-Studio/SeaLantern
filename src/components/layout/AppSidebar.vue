<script setup lang="ts">
import { computed, ref, nextTick, watch, onMounted, onBeforeUnmount } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "../../stores/uiStore";
import { useServerStore } from "../../stores/serverStore";
import { i18n } from "../../locales";
import {
  Listbox,
  ListboxButton,
  ListboxOptions,
  ListboxOption,
  Disclosure,
  DisclosureButton,
  DisclosurePanel,
  Portal,
} from "@headlessui/vue";
import {
  Home,
  Plus,
  Terminal,
  Settings,
  Users,
  Sliders,
  Palette,
  Info,
  Server,
  ChevronRight,
} from "lucide-vue-next";

const router = useRouter();
const route = useRoute();
const ui = useUiStore();
const serverStore = useServerStore();
const navIndicator = ref<HTMLElement | null>(null);

interface NavItem {
  name: string;
  path: string;
  icon: string;
  labelKey: string;
  label: string;
  group: string;
}

const navItems: NavItem[] = [
  {
    name: "home",
    path: "/",
    icon: "home",
    labelKey: "common.home",
    label: i18n.t("common.home"),
    group: "main",
  },
  {
    name: "create",
    path: "/create",
    icon: "plus",
    labelKey: "common.create_server",
    label: i18n.t("common.create_server"),
    group: "main",
  },
  {
    name: "console",
    path: "/console",
    icon: "terminal",
    labelKey: "common.console",
    label: i18n.t("common.console"),
    group: "server",
  },
  {
    name: "config",
    path: "/config",
    icon: "sliders",
    labelKey: "common.config_edit",
    label: i18n.t("common.config_edit"),
    group: "server",
  },
  {
    name: "players",
    path: "/players",
    icon: "users",
    labelKey: "common.player_manage",
    label: i18n.t("common.player_manage"),
    group: "server",
  },
  {
    name: "paint",
    path: "/paint",
    icon: "paint",
    labelKey: "common.personalize",
    label: i18n.t("common.personalize"),
    group: "system",
  },
  {
    name: "settings",
    path: "/settings",
    icon: "settings",
    labelKey: "common.settings",
    label: i18n.t("common.settings"),
    group: "system",
  },
];

function navigateTo(path: string) {
  router.push(path);
}

// 服务器选择由 Headless UI 的 Listbox 管理

// 更新导航指示器位置
function updateNavIndicator() {
  nextTick(() => {
    if (!navIndicator.value) return;

    const activeNavItem = document.querySelector(".nav-item.active");
    if (activeNavItem && navIndicator.value.parentElement) {
      // 使用 getBoundingClientRect 来获取相对于父元素的正确位置
      const navItemRect = activeNavItem.getBoundingClientRect();
      const navRect = navIndicator.value.parentElement.getBoundingClientRect();
      const top = navItemRect.top - navRect.top + (navItemRect.height - 16) / 2;

      // 确保导航指示器可见
      navIndicator.value.style.display = "block";

      // 强制触发重排，确保动画能够正确执行
      void navIndicator.value.offsetHeight; // 触发重排

      // 使用 requestAnimationFrame 确保动画在正确的时机执行
      requestAnimationFrame(() => {
        navIndicator.value!.style.top = `${top}px`;
      });
    }
  });
}

// 监听侧边栏折叠状态变化，更新指示器位置
watch(
  () => ui.sidebarCollapsed,
  () => {
    // 延迟更新，确保动画完成后再计算位置
    setTimeout(() => {
      updateNavIndicator();
      // 在侧边栏折叠/展开后同时更新弹出列表位置
      updateOptionsPosition();
    }, 350); // 等待350ms，确保CSS过渡动画完全完成
  },
);

// 监听路由变化，更新指示器位置
watch(
  () => route.path,
  () => {
    // 使用 nextTick 确保 DOM 已经更新
    nextTick(() => {
      updateNavIndicator();
      // 路由变化时也更新弹出列表位置（若正在打开）
      updateOptionsPosition();
    });
  },
);

// 组件挂载后初始化指示器位置和服务器列表
onMounted(async () => {
  // 加载服务器列表
  await serverStore.refreshList();

  // 等待服务器列表加载完成后再更新指示器位置
  nextTick(() => {
    updateNavIndicator();
    // 初始化 ListboxOptions 的位置，确保弹出在合适的位置
    updateOptionsPosition();
  });

  // 不再需要手动外部点击处理，Listbox 会负责焦点/键盘可访问性
});

function handleServerChange(value: string | number) {
  serverStore.setCurrentServer(String(value));
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

// 用于把 ListboxOptions 渲染到 body，并在侧边栏收起时调整到侧边栏右侧
const listboxButton = ref<HTMLElement | null>(null);
const optionsStyle = ref<Record<string, string | number>>({});

function updateOptionsPosition() {
  nextTick(() => {
    // listboxButton 可能是 DOM 元素，也可能是组件实例（有 $el）
    let btnEl: HTMLElement | null = null;
    const raw = listboxButton.value as any;
    if (!raw) return;
    if (raw instanceof HTMLElement) {
      btnEl = raw;
    } else if (raw.$el && raw.$el instanceof HTMLElement) {
      btnEl = raw.$el as HTMLElement;
    } else if (raw.$el && raw.$el.$el && raw.$el.$el instanceof HTMLElement) {
      // 处理嵌套组件暴露的情况
      btnEl = raw.$el.$el as HTMLElement;
    }
    if (!btnEl) return;

    const btnRect = btnEl.getBoundingClientRect();
    const sidebarEl = document.querySelector(".sidebar") as HTMLElement | null;
    const sidebarRect = sidebarEl ? sidebarEl.getBoundingClientRect() : null;

    // 默认宽度与样式
    const width = sidebarRect && !ui.sidebarCollapsed ? Math.max(200, btnRect.width) : 200;

    // 计算固定定位的 top/left（相对于视口）
    let top = Math.round(btnRect.bottom);
    let left = Math.round(btnRect.left);

    // 如果侧边栏收起，将列表显示在侧边栏右侧
    if (sidebarRect && ui.sidebarCollapsed) {
      left = Math.round(sidebarRect.right + 8);
      // 让列表垂直居中于按钮（或靠近按钮）
      top = Math.round(btnRect.top + (btnRect.height - 40) / 2);
    }

    optionsStyle.value = {
      position: "fixed",
      top: `${top}px`,
      left: `${left}px`,
      width: `${width}px`,
    };
  });
}

// 更新位置：窗口尺寸变动或滚动时
function onWindowChange() {
  updateOptionsPosition();
}

window.addEventListener("resize", onWindowChange);
window.addEventListener("scroll", onWindowChange, true);

onBeforeUnmount(() => {
  window.removeEventListener("resize", onWindowChange);
  window.removeEventListener("scroll", onWindowChange, true);
});

// 服务器选项
const serverOptions = computed(() => {
  return serverStore.servers.map((s) => ({
    label: s.name,
    value: s.id,
  }));
});

// 使用本地 ref 作为 Listbox 的 v-model，保持和 store 同步
const currentServerRef = ref<string | undefined>(serverStore.currentServerId ?? undefined);

// 当 store 改变时同步到本地 ref
watch(
  () => serverStore.currentServerId,
  (v) => {
    currentServerRef.value = v ?? undefined;
  },
);

// 当本地 ref 改变时触发处理逻辑（会更新 store）
watch(
  () => currentServerRef.value,
  (v, old) => {
    if (v != null && v !== old) {
      handleServerChange(v);
    }
  },
);

// 监听服务器列表变化，更新指示器位置
watch(
  () => serverOptions.value.length,
  () => {
    updateNavIndicator();
  },
);

// 监听当前服务器变化（本地 ref），更新指示器位置
watch(
  () => currentServerRef.value,
  () => {
    updateNavIndicator();
  },
);

// 便捷计算当前服务器标签
const getCurrentServerLabel = computed(() => {
  const cur = serverOptions.value.find((o) => o.value === currentServerRef.value);
  return cur ? cur.label : i18n.t("common.select_server");
});

function isActive(path: string): boolean {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
}

// 图标已按需导入，模板中直接使用组件标签替代映射表
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
      <!-- 服务器选择（Headless UI Listbox） -->
      <Listbox v-if="serverOptions.length > 0" v-model="currentServerRef" class="server-selector" horizontal>
        <div>
          <ListboxButton
            ref="listboxButton"
            class="server-selector-button"
            :aria-label="i18n.t('common.select_server')"
            @click="updateOptionsPosition"
            @focus="updateOptionsPosition"
          >
            <Server :size="20" :stroke-width="1.8" class="server-icon" />
            <template v-if="!ui.sidebarCollapsed">
              <div class="server-select-box">{{ getCurrentServerLabel }}</div>
            </template>
          </ListboxButton>

          <!-- 将 ListboxOptions 渲染到 body（Portal），并使用固定定位样式 -->
          <Portal>
            <transition name="bubble">
              <ListboxOptions
                class="server-select-bubble-content-portal"
                :style="optionsStyle"
              >
                <div class="server-select-bubble-body">
                  <ListboxOption
                    v-for="option in serverOptions"
                    :key="option.value"
                    :value="option.value"
                    v-slot="{ selected }"
                  >
                    <div
                      :class="['server-select-option', { active: option.value === currentServerRef }]"
                    >
                      {{ option.label }}
                    </div>
                  </ListboxOption>
                </div>
              </ListboxOptions>
            </transition>
          </Portal>
        </div>
      </Listbox>

      <!-- 导航激活指示器 -->
      <div class="nav-active-indicator" ref="navIndicator"></div>

      <!-- 主菜单组 -->
      <div class="nav-group">
        <div v-if="serverOptions.length > 0" class="nav-group-label"></div>
        <div>
          <div
            v-for="item in navItems.filter((i) => i.group === 'main')"
            :key="item.name"
            class="nav-item"
            :class="{ active: isActive(item.path) }"
            @click="navigateTo(item.path)"
            :title="ui.sidebarCollapsed ? item.label : ''"
          >
            <Home v-if="item.icon === 'home'" class="nav-icon" :size="20" :stroke-width="1.8" />
            <Plus
              v-else-if="item.icon === 'plus'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Terminal
              v-else-if="item.icon === 'terminal'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Settings
              v-else-if="item.icon === 'settings'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Users
              v-else-if="item.icon === 'users'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Sliders
              v-else-if="item.icon === 'sliders'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Palette
              v-else-if="item.icon === 'paint'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Info v-else class="nav-icon" :size="20" :stroke-width="1.8" />
            <transition name="fade">
              <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t(item.labelKey) }}</span>
            </transition>
          </div>
        </div>
      </div>

      <!-- 服务器菜单组 -->
      <div v-if="serverOptions.length > 0" class="nav-group">
        <div class="nav-group-label"></div>
        <div>
          <div
            v-for="item in navItems.filter((i) => i.group === 'server')"
            :key="item.name"
            class="nav-item"
            :class="{ active: isActive(item.path) }"
            @click="navigateTo(item.path)"
            :title="ui.sidebarCollapsed ? item.label : ''"
          >
            <Home v-if="item.icon === 'home'" class="nav-icon" :size="20" :stroke-width="1.8" />
            <Plus
              v-else-if="item.icon === 'plus'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Terminal
              v-else-if="item.icon === 'terminal'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Settings
              v-else-if="item.icon === 'settings'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Users
              v-else-if="item.icon === 'users'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Sliders
              v-else-if="item.icon === 'sliders'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Palette
              v-else-if="item.icon === 'paint'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Info v-else class="nav-icon" :size="20" :stroke-width="1.8" />
            <transition name="fade">
              <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t(item.labelKey) }}</span>
            </transition>
          </div>
        </div>
      </div>

      <!-- 系统菜单组 -->
      <div class="nav-group">
        <div class="nav-group-label"></div>
        <div>
          <div
            v-for="item in navItems.filter((i) => i.group === 'system')"
            :key="item.name"
            class="nav-item"
            :class="{ active: isActive(item.path) }"
            @click="navigateTo(item.path)"
            :title="ui.sidebarCollapsed ? item.label : ''"
          >
            <Home v-if="item.icon === 'home'" class="nav-icon" :size="20" :stroke-width="1.8" />
            <Plus
              v-else-if="item.icon === 'plus'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Terminal
              v-else-if="item.icon === 'terminal'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Settings
              v-else-if="item.icon === 'settings'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Users
              v-else-if="item.icon === 'users'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Sliders
              v-else-if="item.icon === 'sliders'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Palette
              v-else-if="item.icon === 'paint'"
              class="nav-icon"
              :size="20"
              :stroke-width="1.8"
            />
            <Info v-else class="nav-icon" :size="20" :stroke-width="1.8" />
            <transition name="fade">
              <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t(item.labelKey) }}</span>
            </transition>
          </div>
        </div>
      </div>
    </nav>

    <!-- 弹出服务器选择由 Listbox 管理（原手动气泡已移除） -->

    <div class="sidebar-footer">
      <div class="nav-item" @click="navigateTo('/about')" :title="ui.sidebarCollapsed ? i18n.t('common.about') : ''">
        <Info class="nav-icon" :size="20" :stroke-width="1.8" />
        <transition name="fade">
          <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t("common.about") }}</span>
        </transition>
      </div>
      <div class="nav-item collapse-btn" @click="ui.toggleSidebar()">
        <ChevronRight
          class="nav-icon"
          :style="{ transform: ui.sidebarCollapsed ? 'rotate(180deg)' : '' }"
          :size="20"
          :stroke-width="1.8"
        />
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
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: width;
  transform: translateZ(0);
  backface-visibility: hidden;
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
  justify-content: center;
  position: relative;
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

.server-selector-button {
  width: 100%;
  padding: 8px 8px;
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  background-color: var(--sl-surface);
  transition: all var(--sl-transition-fast);
  cursor: pointer;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: var(--sl-space-sm);
  min-height: 40px;
  white-space: nowrap;
  margin-top: 5px;
}

.server-select-box {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--sl-text-secondary);
  transition: color var(--sl-transition-fast);
}

.sidebar.collapsed .server-selector-button {
  width: 40px;
  height: 40px;
  align-items: center;
  justify-content: center;
  padding: var(--sl-space-xs);
  border: none;
  background-color: transparent;
  min-height: 40px;
  gap: 0;
}

.server-icon {
  color: var(--sl-text-secondary);
  transition: color var(--sl-transition-fast);
}

.server-selector-button:hover {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.server-selector-button:hover .server-icon {
  color: var(--sl-primary);
}

.server-selector-button:hover .server-select-box {
  color: var(--sl-primary);
}



.server-selector-icon {
  padding: 8px;
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 36px;
}

.server-selector-icon:hover {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

/* 弹出的服务器选择气泡 */
.server-select-bubble-content {
  pointer-events: auto;
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 8px;
  z-index: 9999;
}

/* Portal 渲染时使用固定定位（相对于视口） */
.server-select-bubble-content-portal {
  position: fixed;
  pointer-events: auto;
  z-index: 9999;
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-sm);
  box-shadow: var(--sl-shadow-lg);
}

/* 气泡动画 */
.bubble-enter-active,
.bubble-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.bubble-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.bubble-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.9);
}

.bubble-enter-to,
.bubble-leave-from {
  opacity: 1;
  transform: translateY(0) scale(1);
}

.server-select-bubble-content {
  background: var(--sl-surface);
  border: 1px solid var(--sl-border);
  border-radius: var(--sl-radius-md);
  padding: var(--sl-space-sm);
  width: 200px;
  box-shadow: var(--sl-shadow-lg);
}

.server-select-bubble-header h3 {
  color: var(--sl-text-primary);
}

.server-select-option {
  color: var(--sl-text-secondary);
}

.server-select-option:hover {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.server-select-option.active {
  background-color: var(--sl-primary-bg);
  color: var(--sl-primary);
}

.bubble-close {
  color: var(--sl-text-tertiary);
}

.bubble-close:hover {
  color: var(--sl-text-primary);
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
  padding: 10px;
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
  justify-content: flex-start;
  gap: var(--sl-space-sm);
  padding: 8px 12px;
  border-radius: var(--sl-radius-md);
  cursor: pointer;
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  position: relative;
  white-space: nowrap;
  margin-top: 5px;
  min-height: 36px;
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
  will-change: top;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.nav-icon {
  flex-shrink: 0;
  transition: transform var(--sl-transition-normal);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
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
