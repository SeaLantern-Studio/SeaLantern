<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useUiStore } from "@stores/uiStore";
import { useServerStore } from "@stores/serverStore";
import { usePluginStore } from "@stores/pluginStore";
import { i18n } from "@language";
import { useElasticLogo } from "@composables/useElasticLogo";
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
  Blocks,
  Store,
  LayoutDashboard,
  BarChart2,
  Sparkles,
  Link2,
  DownloadIcon,
  Archive,
  BookOpen,
  type LucideIcon,
} from "lucide-vue-next";
import logoSvg from "@assets/logo.svg";
import { isMacOSPlatform } from "@utils/platform";

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
  archive: Archive,
  book: BookOpen,
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
const isMacOS = isMacOSPlatform();

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
    name: "backup",
    path: "/backup",
    icon: "archive",
    labelKey: "common.backup",
    label: i18n.t("common.backup"),
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
  {
    name: "help",
    path: "/help",
    icon: "book",
    labelKey: "common.help",
    label: i18n.t("common.help"),
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
  // 顺序：main(4项) → server组 → 插件注册项 → system组(个性化、插件管理、设置)
  const result: NavItem[] = [];

  // 1. main 组：首页、创建服务器、下载、联机
  for (const item of staticNavItems) {
    if (item.group === "main") result.push(item);
  }

  // 2. server 组：控制台、配置、玩家管理
  for (const item of staticNavItems) {
    if (item.group === "server") result.push(item);
  }

  // 3. 插件注册的导航项（在 main/server 和 system 之间）
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

  const pluginRegisteredItems = [...unpositioned, ...defaultItems, ...remainingPluginItems];
  result.push(...pluginRegisteredItems);

  // 4. system 组：个性化、插件管理、设置
  for (const item of staticNavItems) {
    if (item.group === "system") result.push(item);
  }

  // 处理有 after 定位的插件项
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

// 导航指示器位置更新:用 rAF 合并多次触发,避免连续 querySelector
let updateNavIndicatorRafId: number | null = null;
let cachedSidebarNav: HTMLElement | null = null;

function updateNavIndicator() {
  if (updateNavIndicatorRafId !== null) return;
  updateNavIndicatorRafId = requestAnimationFrame(() => {
    updateNavIndicatorRafId = null;
    if (!navIndicator.value) return;

    // 缓存 sidebar-nav 元素,避免每次更新都 querySelector
    if (!cachedSidebarNav) {
      cachedSidebarNav = document.querySelector<HTMLElement>(".sidebar-nav");
    }
    // 使用 scope 内最近的 .nav-item.active,提高查询效率
    const sidebarNav = cachedSidebarNav;
    const activeNavItem = sidebarNav?.querySelector<HTMLElement>(".nav-item.active");
    if (!activeNavItem || !sidebarNav || !navIndicator.value.parentElement) return;

    const navItemRect = activeNavItem.getBoundingClientRect();
    const sidebarNavRect = sidebarNav.getBoundingClientRect();
    const top =
      navItemRect.top - sidebarNavRect.top + sidebarNav.scrollTop + (navItemRect.height - 16) / 2;

    navIndicator.value.style.display = "block";
    // 强制重排,确保过渡动画触发
    void navIndicator.value.offsetHeight;
    navIndicator.value.style.top = `${top}px`;
  });
}

// 监听路由变化,使用 flush: 'post' 确保 DOM 更新后再算位置
watch(
  () => route.path,
  () => {
    updateNavIndicator();
  },
  { flush: "post" },
);

onMounted(async () => {
  await serverStore.refreshList();
  updateNavIndicator();
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

  // 监听侧边栏滚动，更新指示器位置;复用缓存避免重复 query
  cachedSidebarNav = document.querySelector<HTMLElement>(".sidebar-nav");
  if (cachedSidebarNav) {
    cachedSidebarNav.addEventListener("scroll", updateNavIndicator);
  }
});

onUnmounted(() => {
  // 取消未完成的 rAF,避免组件卸载后操作 DOM
  if (updateNavIndicatorRafId !== null) {
    cancelAnimationFrame(updateNavIndicatorRafId);
    updateNavIndicatorRafId = null;
  }
  window.removeEventListener("resize", updateNavIndicator);

  // 移除侧边栏滚动监听
  const sidebarNav = cachedSidebarNav || document.querySelector(".sidebar-nav");
  if (sidebarNav) {
    sidebarNav.removeEventListener("scroll", updateNavIndicator);
  }
  cachedSidebarNav = null;
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
  // 按 group 聚合，连续相同 group 合并
  // 分组顺序：main → server → plugins-default → plugins-custom → system
  const groups: NavGroup[] = [];
  let currentGroup: NavGroup | null = null;

  for (const item of navItems.value) {
    const effectiveGroup = item.group;
    if (item.isPlugin && !item.after) {
      // 插件自定义项单独成组
      groups.push({ group: "plugins-custom", items: [item] });
      currentGroup = null;
      continue;
    }
    if (!currentGroup || currentGroup.group !== effectiveGroup) {
      currentGroup = { group: effectiveGroup, items: [] };
      groups.push(currentGroup);
    }
    currentGroup.items.push(item);
  }

  return groups;
});

// 彩蛋
function getAppName() {
  const now = new Date();
  if (now.getMonth() == 3 && now.getDate() == 1) {
    return i18n.t("common.easter_name");
  }
  return i18n.t("common.app_name");
}

// 海景灯图标彩蛋：长按 3 秒可拖动，松手后弹力绳弹飞
const logoIconRef = ref<HTMLElement | null>(null);
const elastic = useElasticLogo();

function onLogoMouseDown(e: MouseEvent) {
  if (!logoIconRef.value) return;
  elastic.startHold(e, logoIconRef.value);
}

function onLogoTouchStart(e: TouchEvent) {
  if (!logoIconRef.value) return;
  elastic.startHold(e, logoIconRef.value);
}

// 全局监听拖动（避免鼠标移出元素就丢失）
function onWindowMouseMove(e: MouseEvent) {
  if (elastic.isArmed.value && !elastic.isDragging.value) {
    // 激活后按下左键才开始拖动
    if (e.buttons === 1) elastic.startDrag(e);
  } else if (elastic.isDragging.value) {
    elastic.moveDrag(e);
  }
}

function onWindowTouchMove(e: TouchEvent) {
  if (elastic.isArmed.value && !elastic.isDragging.value) {
    elastic.startDrag(e);
  } else if (elastic.isDragging.value) {
    elastic.moveDrag(e);
  }
}

function onWindowMouseUp() {
  if (elastic.isDragging.value) {
    elastic.releaseDrag();
  } else if (elastic.isArmed.value && !elastic.isAnimating.value) {
    // 已激活但没拖动，单击则取消激活
    elastic.isArmed.value = false;
  } else {
    elastic.cancelHold();
  }
}

function onLogoMouseLeave() {
  // 没激活时鼠标离开就取消长按计时
  if (!elastic.isArmed.value && !elastic.isAnimating.value) {
    elastic.cancelHold();
  }
}

/** 点击 logo：激活态下不触发导航，避免拖动后被误判为点击 */
function onLogoClick() {
  if (elastic.isArmed.value || elastic.isAnimating.value || elastic.isDragging.value) {
    return;
  }
  navigateTo("/");
}

onMounted(() => {
  window.addEventListener("mousemove", onWindowMouseMove);
  window.addEventListener("mouseup", onWindowMouseUp);
  window.addEventListener("touchmove", onWindowTouchMove, { passive: false });
  window.addEventListener("touchend", onWindowMouseUp);
});

onUnmounted(() => {
  window.removeEventListener("mousemove", onWindowMouseMove);
  window.removeEventListener("mouseup", onWindowMouseUp);
  window.removeEventListener("touchmove", onWindowTouchMove);
  window.removeEventListener("touchend", onWindowMouseUp);
});

// 图标已按需导入，模板中直接使用组件标签替代映射表
</script>

<template>
  <aside class="sidebar" :class="{ 'macos-overlay': isMacOS }">
    <div
      class="sidebar-logo"
      :class="{ 'logo-armed': elastic.isArmed.value, 'logo-dragging': elastic.isDragging.value }"
      @click="onLogoClick"
    >
      <div
        ref="logoIconRef"
        class="logo-icon"
        :style="
          elastic.isDragging.value || elastic.isAnimating.value
            ? {
                position: 'fixed',
                left: elastic.position.value.x - 14 + 'px',
                top: elastic.position.value.y - 14 + 'px',
                zIndex: 9999,
                pointerEvents: 'none',
              }
            : {}
        "
        @mousedown="onLogoMouseDown"
        @touchstart="onLogoTouchStart"
        @mouseleave="onLogoMouseLeave"
      >
        <img
          :src="logoSvg"
          width="28"
          height="28"
          :alt="i18n.t('common.app_name')"
          draggable="false"
        />
        <!-- 激活提示光晕 -->
        <div v-if="elastic.isArmed.value" class="logo-armed-glow"></div>
      </div>
      <!-- 拖动/弹飞时保留原占位，避免布局塌陷 -->
      <div
        v-if="elastic.isDragging.value || elastic.isAnimating.value"
        class="logo-placeholder"
        aria-hidden="true"
      ></div>
      <span class="logo-text">{{ getAppName() }}</span>
    </div>

    <!-- 弹力绳 SVG 覆盖层 -->
    <svg v-if="elastic.isDragging.value || elastic.isAnimating.value" class="elastic-rope-layer">
      <path
        :d="elastic.elasticPath()"
        :stroke="elastic.elasticColor()"
        :stroke-width="elastic.elasticWidth()"
        fill="none"
        stroke-linecap="round"
      />
      <!-- 锚点圆环 -->
      <circle
        :cx="elastic.anchor.value.x"
        :cy="elastic.anchor.value.y"
        r="4"
        fill="var(--sl-primary, #0ea5e9)"
        opacity="0.6"
      />
    </svg>
    <nav class="sidebar-nav">
      <div class="nav-active-indicator" ref="navIndicator"></div>
      <cmz-select
        v-if="serverOptions.length > 0"
        v-model="currentServerRef"
        :options="serverOptions"
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
            <span>{{ group.items[0]?.pluginName || group.items[0]?.label }}</span>
          </div>
          <div v-else-if="group.group === 'plugins-default'" class="nav-group-label">
            <span>{{ i18n.t("common.plugins") }}</span>
          </div>
          <div v-else-if="group.group !== 'main'" class="nav-separator"></div>

          <div>
            <div v-for="item in group.items" :key="item.name">
              <div
                class="nav-item"
                :class="{ active: isActive(item.path) }"
                @click="navigateTo(item.path)"
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
                <span class="nav-label">
                  {{ item.labelKey ? i18n.t(item.labelKey) : item.label }}
                </span>
              </div>
              <!-- 子项 -->
              <div v-if="item.children?.length" class="nav-children">
                <div
                  v-for="child in item.children"
                  :key="child.name"
                  class="nav-item nav-child-item"
                  :class="{ active: isActive(child.path) }"
                  @click="navigateTo(child.path)"
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
                  <span class="nav-label">{{ child.label }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- 关于按钮 -->
      <div class="nav-group lower-side">
        <div class="nav-item" :class="{ active: isActive('/about') }" @click="navigateTo('/about')">
          <Info class="nav-icon" :size="20" :stroke-width="1.8" />
          <span class="nav-label">{{ i18n.t("common.about") }}</span>
        </div>
      </div>
    </nav>
  </aside>
</template>

<style src="@styles/components/layout/AppSidebar.css" scoped></style>
