<script setup lang="ts">
import { computed, ref, nextTick, watch, onMounted, onUnmounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "@stores/uiStore";
import { useServerStore } from "@stores/serverStore";
import { usePluginStore } from "@stores/pluginStore";
import { useSettingsStore } from "@stores/settingsStore";
import { i18n } from "@language";
import SLSelect from "@components/common/SLSelect.vue";
import {
  Home,
  Plus,
  Terminal,
  Settings,
  Users,
  Sliders,
  PaintRoller,
  Info,
  Server,
  ChevronLeft,
  Blocks,
  Store,
  LayoutDashboard,
  BarChart2,
  Sparkles,
  Link2,
  DownloadIcon,
  type LucideIcon,
} from "@lucide/vue";
import logoSvg from "@assets/logo.svg";
import { isMacOSPlatform } from "@utils/platform";
import { getAppDisplayName } from "@utils/theme";

const iconMap: Record<string, LucideIcon> = {
  home: Home,
  plus: Plus,
  terminal: Terminal,
  settings: Settings,
  users: Users,
  sliders: Sliders,
  paint: PaintRoller,
  info: Info,
  server: Server,
  blocks: Blocks,
  store: Store,
  "layout-dashboard": LayoutDashboard,
  chart: BarChart2,
  sparkles: Sparkles,
  link2: Link2,
  download: DownloadIcon,
};

function getNavIcon(name: string): LucideIcon {
  return iconMap[name] ?? Info;
}

const router = useRouter();
const route = useRoute();
const ui = useUiStore();
const serverStore = useServerStore();
const pluginStore = usePluginStore();
const settingsStore = useSettingsStore();
const navIndicator = ref<HTMLElement | null>(null);
const sidebarNavRef = ref<HTMLElement | null>(null);
const sidebarTransitioning = ref(false);
const isMacOS = isMacOSPlatform();
let indicatorSyncTimeout: ReturnType<typeof setTimeout> | null = null;
let indicatorFrameId: number | null = null;
let indicatorResizeObserver: ResizeObserver | null = null;

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
    name: "download",
    path: "/download",
    icon: "download",
    labelKey: "common.download",
    label: i18n.t("common.download"),
    group: "main",
  },
  {
    name: "tunnel",
    path: "/tunnel",
    icon: "link2",
    labelKey: "common.tunnel",
    label: i18n.t("common.tunnel"),
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
    icon: "blocks",
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
    icon: item.icon || "blocks",
    labelKey: "",
    label: item.label,
    group: "plugins",
    isPlugin: true,
    pluginId: item.plugin_id,
    pluginIcon: pluginStore.icons[item.plugin_id] || undefined,
  }));
});

function sidebarItemToNavItem(item: import("@type/plugin").SidebarItem): NavItem {
  const path =
    item.mode === "category" ? `/plugin-category/${item.pluginId}` : `/plugin/${item.pluginId}`;
  const pluginManifest = pluginStore.plugins.find((p) => p.manifest.id === item.pluginId)?.manifest;
  return {
    name: `sidebar-${item.pluginId}`,
    path,
    icon: item.icon || "blocks",
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
  const result: NavItem[] = [];
  const settingsIndex = staticNavItems.findIndex((item) => item.name === "settings");
  const developerItem: NavItem = {
    name: "developer",
    path: "/developer",
    icon: "sparkles",
    labelKey: "common.developer_tools",
    label: i18n.t("common.developer_tools"),
    group: "system",
  };
  const baseStaticItems =
    settingsStore.settings.developer_mode && settingsIndex >= 0
      ? [
          ...staticNavItems.slice(0, settingsIndex),
          developerItem,
          ...staticNavItems.slice(settingsIndex),
        ]
      : staticNavItems;

  // 收集插件边栏项目
  const positioned = pluginStore.sidebarItems
    .filter((i) => !i.isDefault && i.after)
    .map(sidebarItemToNavItem);

  const unpositioned = pluginStore.sidebarItems
    .filter((i) => !i.isDefault && !i.after)
    .map(sidebarItemToNavItem);

  const defaultItems = pluginStore.sidebarItems
    .filter((i) => i.isDefault)
    .map(sidebarItemToNavItem);

  const handledPluginIds = new Set(pluginStore.sidebarItems.map((i) => i.pluginId));
  const remainingPluginItems = pluginNavItems.value.filter(
    (i) => !i.pluginId || !handledPluginIds.has(i.pluginId),
  );

  // 放在 plugins 和 settings 之间的插件项
  const pluginItemsBetweenPluginsAndSettings = [
    ...unpositioned,
    ...defaultItems,
    ...remainingPluginItems,
  ];

  // 遍历静态导航项，在 plugins 和 settings 之间插入插件边栏项目
  for (const staticItem of baseStaticItems) {
    result.push(staticItem);

    // 在 plugins 项之后插入插件边栏项目
    if (staticItem.name === "plugins") {
      result.push(...pluginItemsBetweenPluginsAndSettings);
    }
  }

  // 处理有 after 定位的插件项（插入到指定位置）
  for (const item of positioned) {
    const targetIdx = result.findIndex((r) => r.name === item.after);
    if (targetIdx !== -1) {
      result.splice(targetIdx + 1, 0, item);
    } else {
      result.push(item);
    }
  }

  return result;
});

function navigateTo(path: string) {
  router.push(path);
}

function cancelIndicatorFrame() {
  if (indicatorFrameId !== null) {
    cancelAnimationFrame(indicatorFrameId);
    indicatorFrameId = null;
  }
}

function scheduleNavIndicatorUpdate() {
  cancelIndicatorFrame();
  indicatorFrameId = requestAnimationFrame(() => {
    indicatorFrameId = null;
    void updateNavIndicator();
  });
}

function updateNavIndicator() {
  return nextTick(() => {
    const indicator = navIndicator.value;
    const sidebarNav = sidebarNavRef.value;
    const activeNavItem = sidebarNav?.querySelector<HTMLElement>(".nav-item.active");

    if (!indicator || !sidebarNav || !activeNavItem || !indicator.parentElement) {
      if (indicator) {
        indicator.style.display = "none";
      }
      return;
    }

    const navItemRect = activeNavItem.getBoundingClientRect();
    const sidebarNavRect = sidebarNav.getBoundingClientRect();
    const top =
      navItemRect.top - sidebarNavRect.top + sidebarNav.scrollTop + (navItemRect.height - 16) / 2;

    indicator.style.display = "block";
    indicator.style.top = `${top}px`;
  });
}

function startIndicatorSyncDuringSidebarTransition() {
  if (indicatorSyncTimeout) {
    clearTimeout(indicatorSyncTimeout);
    indicatorSyncTimeout = null;
  }

  sidebarTransitioning.value = true;
  scheduleNavIndicatorUpdate();

  indicatorSyncTimeout = setTimeout(() => {
    sidebarTransitioning.value = false;
    scheduleNavIndicatorUpdate();
  }, 360);
}

// 监听侧边栏折叠状态变化，更新指示器位置
watch(
  () => ui.sidebarCollapsed,
  () => {
    scheduleNavIndicatorUpdate();
    startIndicatorSyncDuringSidebarTransition();
  },
);

// 监听路由变化，更新指示器位置
watch(
  () => route.path,
  () => {
    scheduleNavIndicatorUpdate();
  },
);

onMounted(async () => {
  await serverStore.refreshList();
  scheduleNavIndicatorUpdate();
});

function handleServerChange(value: string) {
  serverStore.setCurrentServer(value);
  if (
    route.path.startsWith("/console") ||
    route.path.startsWith("/config") ||
    route.path.startsWith("/players")
  ) {
    const currentPath = route.path.split("/")[1];
    router.push(`/${currentPath}/${value}`);
  }
}

const serverOptions = computed(() => {
  return serverStore.servers.map((s) => ({
    label: s.name,
    value: s.id,
  }));
});

const currentServerRef = computed({
  get: () => serverStore.currentServerId ?? undefined,
  set: (v) => {
    if (v) handleServerChange(v);
  },
});

watch(
  () => serverOptions.value.length,
  () => {
    scheduleNavIndicatorUpdate();
  },
);

onMounted(() => {
  window.addEventListener("resize", scheduleNavIndicatorUpdate);

  const sidebarNav = sidebarNavRef.value;
  if (sidebarNav) {
    sidebarNav.addEventListener("scroll", scheduleNavIndicatorUpdate, { passive: true });
    if (typeof ResizeObserver !== "undefined") {
      indicatorResizeObserver = new ResizeObserver(() => {
        scheduleNavIndicatorUpdate();
      });
      indicatorResizeObserver.observe(sidebarNav);
    }
  }
});

onUnmounted(() => {
  if (indicatorSyncTimeout) {
    clearTimeout(indicatorSyncTimeout);
    indicatorSyncTimeout = null;
  }
  cancelIndicatorFrame();
  indicatorResizeObserver?.disconnect();
  indicatorResizeObserver = null;

  window.removeEventListener("resize", scheduleNavIndicatorUpdate);

  const sidebarNav = sidebarNavRef.value;
  if (sidebarNav) {
    sidebarNav.removeEventListener("scroll", scheduleNavIndicatorUpdate);
  }
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

// 彩蛋
function getAppName() {
  if (settingsStore.settings) {
    return getAppDisplayName(settingsStore.settings);
  }

  return i18n.t("common.app_name");
}

// 图标已按需导入，模板中直接使用组件标签替代映射表
</script>

<template>
  <aside
    class="sidebar glass-strong"
    :class="{
      collapsed: ui.sidebarCollapsed,
      'macos-overlay': isMacOS,
      'sidebar-transitioning': sidebarTransitioning,
    }"
  >
    <div class="sidebar-logo" @click="navigateTo('/')">
      <div class="logo-icon">
        <img :src="logoSvg" width="28" height="28" :alt="i18n.t('common.app_name')" />
      </div>
      <transition name="fade">
        <span v-if="!ui.sidebarCollapsed" class="logo-text">{{ getAppName() }}</span>
      </transition>
    </div>
    <nav ref="sidebarNavRef" class="sidebar-nav">
      <div class="nav-active-indicator" ref="navIndicator"></div>
      <SLSelect
        v-if="serverOptions.length > 0"
        v-model="currentServerRef"
        :options="serverOptions"
        :collapsed="ui.sidebarCollapsed"
        :icon="Server"
        :placeholder="i18n.t('common.select_server')"
        variant="server"
        dropdown-align="right"
        class="server-selector"
      />

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
                    :is="getNavIcon(child.icon || 'blocks')"
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

      <!-- 关于按钮 -->
      <div class="nav-group lower-side">
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
      </div>
    </nav>

    <div class="sidebar-footer">
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

<style src="@styles/components/layout/AppSidebar.css" scoped></style>
