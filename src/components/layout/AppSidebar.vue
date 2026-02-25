<script setup lang="ts">
import { computed, ref, nextTick, watch, onMounted, onUnmounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "@stores/uiStore";
import { useServerStore } from "@stores/serverStore";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import SLServerSelector from "@components/common/SLServerSelector.vue";
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
  Puzzle,
  Store,
  LayoutDashboard,
  BarChart2,
  Sparkles,
  DownloadCloudIcon,
  type LucideIcon,
} from "lucide-vue-next";
import logoSvg from "@assets/logo.svg";

const iconMap: Record<string, LucideIcon> = {
  home: Home,
  plus: Plus,
  terminal: Terminal,
  settings: Settings,
  users: Users,
  sliders: Sliders,
  paint: Palette,
  info: Info,
  server: Server,
  puzzle: Puzzle,
  store: Store,
  "layout-dashboard": LayoutDashboard,
  chart: BarChart2,
  sparkles: Sparkles,
  download: DownloadCloudIcon,
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
  {
    name: "download-file",
    path: "/download-file",
    icon: "download",
    labelKey: "common.download-file",
    label: i18n.t("common.download-file"),
    group: "tools",
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
  const result: NavItem[] = [];

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
  for (const staticItem of staticNavItems) {
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

function updateNavIndicator() {
  nextTick(() => {
    if (!navIndicator.value) return;

    const activeNavItem = document.querySelector(".nav-item.active");
    const sidebarNav = document.querySelector(".sidebar-nav");

    if (activeNavItem && sidebarNav && navIndicator.value.parentElement) {
      // 获取滚动容器和激活项的位置
      const navItemRect = activeNavItem.getBoundingClientRect();
      const sidebarNavRect = sidebarNav.getBoundingClientRect();

      // 计算相对于滚动容器的位置（考虑滚动偏移）
      const top =
        navItemRect.top - sidebarNavRect.top + sidebarNav.scrollTop + (navItemRect.height - 16) / 2;

      // 确保导航指示器可见
      navIndicator.value.style.display = "block";

      // 强制触发重排，确保动画能够正确执行
      void navIndicator.value.offsetHeight;

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
    setTimeout(() => {
      updateNavIndicator();
    }, 350);
  },
);

// 监听路由变化，更新指示器位置
watch(
  () => route.path,
  () => {
    nextTick(() => {
      updateNavIndicator();
    });
  },
);

onMounted(async () => {
  await serverStore.refreshList();
  nextTick(() => {
    updateNavIndicator();
  });
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
    updateNavIndicator();
  },
);

// 监听窗口尺寸变化，更新选项位置
onMounted(() => {
  window.addEventListener("resize", updateNavIndicator);

  // 监听侧边栏滚动，更新指示器位置
  const sidebarNav = document.querySelector(".sidebar-nav");
  if (sidebarNav) {
    sidebarNav.addEventListener("scroll", updateNavIndicator);
  }
});

onUnmounted(() => {
  window.removeEventListener("resize", updateNavIndicator);

  // 移除侧边栏滚动监听
  const sidebarNav = document.querySelector(".sidebar-nav");
  if (sidebarNav) {
    sidebarNav.removeEventListener("scroll", updateNavIndicator);
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

// 图标已按需导入，模板中直接使用组件标签替代映射表
</script>

<template>
  <aside class="sidebar glass-strong" :class="{ collapsed: ui.sidebarCollapsed }">
    <div class="sidebar-logo" @click="navigateTo('/')">
      <div class="logo-icon">
        <img
          :src="logoSvg"
          width="28"
          height="28"
          :alt="i18n.t('common.app_name')"
        />
      </div>
      <transition name="fade">
        <span v-if="!ui.sidebarCollapsed" class="logo-text">{{ i18n.t("common.app_name") }}</span>
      </transition>
    </div>
    <nav class="sidebar-nav">
      <div class="nav-active-indicator" ref="navIndicator"></div>
      <SLServerSelector
        v-if="serverOptions.length > 0"
        v-model="currentServerRef"
        :options="serverOptions"
        :collapsed="ui.sidebarCollapsed"
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

      <!-- 关于按钮 -->
      <div class="nav-group">
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
