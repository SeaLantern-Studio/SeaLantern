<script setup lang="ts">
import { computed, ref, nextTick, watch, onMounted, onUnmounted, onBeforeUnmount } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "../../stores/uiStore";
import { useServerStore } from "../../stores/serverStore";
import { usePluginStore } from "../../stores/pluginStore";
import { i18n } from "../../language";
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
  ChevronLeft,
  ChevronRight,
  Puzzle,
  Store,
  LayoutDashboard,
  BarChart2,
  Sparkles,
  type LucideIcon,
} from "lucide-vue-next";

const iconMap: Record<string, LucideIcon> = {
  home: Home,
  plus: Plus,
  terminal: Terminal,
  settings: Settings,
  users: Users,
  sliders: Sliders,
  palette: Palette,
  paint: Palette,
  info: Info,
  server: Server,
  puzzle: Puzzle,
  store: Store,
  "layout-dashboard": LayoutDashboard,
  chart: BarChart2,
  sparkles: Sparkles,
};

function getNavIcon(name: string): LucideIcon {
  return iconMap[name] ?? Info;
}

const router = useRouter();
const route = useRoute();
const ui = useUiStore();
const serverStore = useServerStore();
const pluginStore = usePluginStore();
const navIndicator = ref<HTMLElement | null>(null);

interface NavItem {
  name: string;
  path: string;
  icon: string;
  labelKey: string;
  label: string;
  group: string;
  isPlugin?: boolean;
  pluginId?: string;
  pluginIcon?: string;
  pluginName?: string;
  after?: string;
  children?: NavItem[];
}

const staticNavItems: NavItem[] = [
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
    name: "plugins",
    path: "/plugins",
    icon: "puzzle",
    labelKey: "common.plugins",
    label: i18n.t("common.plugins"),
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

const pluginNavItems = computed<NavItem[]>(() => {
  return pluginStore.navItems.map((item) => ({
    name: `plugin-${item.plugin_id}`,
    path: `/plugin/${item.plugin_id}`,
    icon: item.icon || "puzzle",
    labelKey: "",
    label: item.label,
    group: "plugins",
    isPlugin: true,
    pluginId: item.plugin_id,
  }));
});

function sidebarItemToNavItem(item: import("../../types/plugin").SidebarItem): NavItem {
  const path =
    item.mode === "category" ? `/plugin-category/${item.pluginId}` : `/plugin/${item.pluginId}`;
  const pluginManifest = pluginStore.plugins.find((p) => p.manifest.id === item.pluginId)?.manifest;
  return {
    name: `sidebar-${item.pluginId}`,
    path,
    icon: item.icon || "puzzle",
    labelKey: "",
    label: item.label,
    group: item.isDefault ? "plugins-default" : "plugins-custom",
    isPlugin: true,
    pluginId: item.pluginId,
    pluginIcon: pluginStore.icons[item.pluginId] || undefined,
    pluginName: pluginManifest?.name,
    after: item.after,
    children: item.children?.map(sidebarItemToNavItem),
  };
}

const navItems = computed<NavItem[]>(() => {
  const result: NavItem[] = [...staticNavItems];

  const positioned = pluginStore.sidebarItems
    .filter((i) => !i.isDefault && i.after)
    .map(sidebarItemToNavItem);

  for (const item of positioned) {
    const idx = result.findIndex((r) => r.name === item.after);
    if (idx !== -1) {
      result.splice(idx + 1, 0, item);
    } else {
      result.push(item);
    }
  }

  const unpositioned = pluginStore.sidebarItems
    .filter((i) => !i.isDefault && !i.after)
    .map(sidebarItemToNavItem);
  result.push(...unpositioned);

  const defaultItems = pluginStore.sidebarItems
    .filter((i) => i.isDefault)
    .map(sidebarItemToNavItem);
  result.push(...defaultItems);

  const handledPluginIds = new Set(pluginStore.sidebarItems.map((i) => i.pluginId));
  result.push(
    ...pluginNavItems.value.filter((i) => !i.pluginId || !handledPluginIds.has(i.pluginId)),
  );

  return result;
});

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

    // 默认宽度与样式：当侧边栏存在且未收起时允许更宽一些
    const width = sidebarRect && !ui.sidebarCollapsed ? Math.max(200, btnRect.width) : 200;

    // 计算固定定位的 top/left（相对于视口）
    let top = Math.round(btnRect.bottom);
    let left = Math.round(btnRect.left);

    // 如果存在侧边栏，无论收起或展开，都将列表显示在侧边栏右侧，避免被侧栏容器裁剪
    // 使用相同的垂直居中逻辑，确保展开与收起时起始位置一致
    if (sidebarRect) {
      left = Math.round(sidebarRect.right + 8);
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

// 监听窗口尺寸变化，更新选项位置
onMounted(() => {
  window.addEventListener("resize", updateNavIndicator);
});

onUnmounted(() => {
  window.removeEventListener("resize", updateNavIndicator);
});

// 便捷计算当前服务器标签
const getCurrentServerLabel = computed(() => {
  const cur = serverOptions.value.find((o) => o.value === currentServerRef.value);
  return cur ? cur.label : i18n.t("common.select_server");
});

function isActive(path: string): boolean {
  if (path === "/") return route.path === "/";
  return route.path.startsWith(path);
}

interface NavGroup {
  group: string;
  items: NavItem[];
}

const orderedNavGroups = computed<NavGroup[]>(() => {
  const groups: NavGroup[] = [];
  let currentGroup: NavGroup | null = null;

  for (const item of navItems.value) {
    if (item.group === "plugins-custom") {
      groups.push({ group: "plugins-custom", items: [item] });
      currentGroup = null;
      continue;
    }
    if (!currentGroup || currentGroup.group !== item.group) {
      currentGroup = { group: item.group, items: [] };
      groups.push(currentGroup);
    }
    currentGroup.items.push(item);
  }

  return groups;
});

// 图标已按需导入，模板中直接使用组件标签替代映射表
</script>

<template>
  <aside class="sidebar glass-strong" :class="{ collapsed: ui.sidebarCollapsed }">
    <div class="sidebar-logo" @click="navigateTo('/')">
      <div class="logo-icon">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 512 512"
          width="28"
          height="28"
          :aria-label="i18n.t('common.app_name')"
          role="img"
        >
          <defs>
            <linearGradient id="sl-logo-grad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" style="stop-color: #60a5fa" />
              <stop offset="100%" style="stop-color: #818cf8" />
            </linearGradient>
          </defs>
          <rect x="0" y="0" width="512" height="512" rx="128" fill="url(#sl-logo-grad)" />
          <rect x="176" y="176" width="160" height="160" rx="48" fill="white" fill-opacity="0.85" />
        </svg>
      </div>
      <transition name="fade">
        <span v-if="!ui.sidebarCollapsed" class="logo-text">{{ i18n.t("common.app_name") }}</span>
      </transition>
    </div>

    <!-- 导航激活指示器 -->
    <div class="nav-active-indicator" ref="navIndicator"></div>

    <nav class="sidebar-nav">
      <!-- 服务器选择（Headless UI Listbox） -->
      <Listbox
        v-if="serverOptions.length > 0"
        v-model="currentServerRef"
        class="server-selector"
        horizontal
      >
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
              <ListboxOptions class="server-select-bubble-content-portal" :style="optionsStyle">
                <div class="server-select-bubble-body">
                  <ListboxOption
                    v-for="option in serverOptions"
                    :key="option.value"
                    :value="option.value"
                    v-slot="{ selected }"
                  >
                    <div
                      :class="[
                        'server-select-option',
                        { active: option.value === currentServerRef },
                      ]"
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

      <!-- 按顺序渲染 -->
      <template v-for="(group, gi) in orderedNavGroups" :key="gi">
        <div v-if="group.group !== 'server' || serverOptions.length > 0" class="nav-group">
          <div v-if="group.group === 'plugins-custom'" class="nav-group-label">
            <transition name="fade">
              <span v-if="!ui.sidebarCollapsed">{{
                group.items[0]?.pluginName || group.items[0]?.label
              }}</span>
            </transition>
          </div>
          <div v-else-if="group.group === 'plugins-default'" class="nav-group-label">
            <transition name="fade">
              <span v-if="!ui.sidebarCollapsed">{{ i18n.t("common.plugins") }}</span>
            </transition>
          </div>
          <div v-else-if="group.group !== 'main'" class="nav-group-label"></div>

          <div>
            <div v-for="item in group.items" :key="item.name">
              <div
                class="nav-item"
                :class="{ active: isActive(item.path) }"
                @click="navigateTo(item.path)"
                :title="ui.sidebarCollapsed ? item.label : ''"
              >
                <img
                  v-if="item.pluginIcon"
                  :src="item.pluginIcon"
                  class="nav-icon nav-plugin-icon"
                  :alt="item.label"
                  width="20"
                  height="20"
                />
                <component
                  v-else
                  :is="getNavIcon(item.icon)"
                  class="nav-icon"
                  :size="20"
                  :stroke-width="1.8"
                />
                <transition name="fade">
                  <span v-if="!ui.sidebarCollapsed" class="nav-label">
                    {{ item.labelKey ? i18n.t(item.labelKey) : item.label }}
                  </span>
                </transition>
              </div>
              <!-- 子项 -->
              <div v-if="item.children?.length" class="nav-children">
                <div
                  v-for="child in item.children"
                  :key="child.name"
                  class="nav-item nav-child-item"
                  :class="{ active: isActive(child.path) }"
                  @click="navigateTo(child.path)"
                  :title="ui.sidebarCollapsed ? child.label : ''"
                >
                  <img
                    v-if="child.pluginIcon"
                    :src="child.pluginIcon"
                    class="nav-icon nav-plugin-icon"
                    :alt="child.label"
                    width="16"
                    height="16"
                  />
                  <component
                    v-else
                    :is="getNavIcon(child.icon || 'puzzle')"
                    class="nav-icon"
                    :size="16"
                    :stroke-width="1.8"
                  />
                  <transition name="fade">
                    <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ child.label }}</span>
                  </transition>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </nav>

    <!-- 弹出服务器选择由 Listbox 管理（原手动气泡已移除） -->

    <div class="sidebar-footer">
      <div
        class="nav-item"
        :class="{ active: isActive('/about') }"
        @click="navigateTo('/about')"
        :title="ui.sidebarCollapsed ? i18n.t('common.about') : ''"
      >
        <Info class="nav-icon" :size="20" :stroke-width="1.8" />
        <transition name="fade">
          <span v-if="!ui.sidebarCollapsed" class="nav-label">{{ i18n.t("common.about") }}</span>
        </transition>
      </div>
      <div class="nav-item collapse-btn" @click="ui.toggleSidebar()">
        <ChevronLeft
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

/* 在亚克力状态下调整边框透明度 */
[data-acrylic="true"] .sidebar {
  border-right: 1px solid rgba(255, 255, 255, 0.03);
}

[data-theme="dark"][data-acrylic="true"] .sidebar {
  border-right: 1px solid rgba(255, 255, 255, 0.03);
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

.nav-plugin-icon {
  border-radius: 4px;
  object-fit: cover;
  display: block;
}

.nav-children {
  margin-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.08);
  padding-left: 4px;
}

.nav-child-item {
  padding-left: 12px !important;
  font-size: 0.8rem;
  opacity: 0.85;
}

.nav-child-item:hover {
  opacity: 1;
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
