<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";
import { Check, House, Pencil, Puzzle, Server, Settings2 } from "@lucide/vue";
import { useRoute, useRouter } from "vue-router";
import { i18n } from "@language";
import { AUTH_ROUTE_NAME } from "@router/authRoute";
import { useAuthStore } from "@stores/authStore";
import NextSidebarHostItems from "../components/host/NextSidebarHostItems.vue";
import type { NextShellNavItem } from "../components/shell/NextShellFrame.vue";
import { useNextHostRuntime } from "../host/runtime";
import NextWorkbenchLayout from "../layout/NextWorkbenchLayout.vue";
import NextHomePage from "../pages/home/NextHomePage.vue";

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();
const nextHostRuntime = useNextHostRuntime();

const instanceHost = computed(() => window.location.host);
const hostSidebarItems = computed(() => nextHostRuntime.sidebarItems.value);
const isPreviewMode = computed(() => route.query.preview === "1");
const isEditMode = shallowRef(false);
const editToggleRef = shallowRef<HTMLButtonElement | null>(null);
const paletteAnchorRect = shallowRef<{ top: number; left: number; width: number; height: number } | null>(null);
const pageSubtitle = computed(() =>
  isPreviewMode.value ? "" : "",
);
const navItems = computed<NextShellNavItem[]>(() => [
  {
    id: "home",
    label: i18n.t("shell.nav_home"),
    icon: House,
    active: true,
  },
  {
    id: "servers",
    label: i18n.t("shell.nav_servers"),
    icon: Server,
    disabled: true,
  },
  {
    id: "plugins",
    label: i18n.t("shell.nav_plugins"),
    icon: Puzzle,
    disabled: true,
  },
  {
    id: "settings",
    label: i18n.t("shell.nav_settings"),
    icon: Settings2,
    disabled: true,
  },
]);

async function handleLogout(): Promise<void> {
  authStore.logout();
  await router.replace({ name: AUTH_ROUTE_NAME });
}

function toggleEditMode(): void {
  isEditMode.value = !isEditMode.value;
}

function syncPaletteAnchorRect(): void {
  const element = editToggleRef.value;
  if (!element) {
    paletteAnchorRect.value = null;
    return;
  }
  const rect = element.getBoundingClientRect();
  paletteAnchorRect.value = {
    top: rect.top,
    left: rect.left,
    width: rect.width,
    height: rect.height,
  };
}

onMounted(() => {
  syncPaletteAnchorRect();
  window.addEventListener("resize", syncPaletteAnchorRect, { passive: true });
  window.addEventListener("scroll", syncPaletteAnchorRect, { passive: true, capture: true });
});

onUnmounted(() => {
  window.removeEventListener("resize", syncPaletteAnchorRect);
  window.removeEventListener("scroll", syncPaletteAnchorRect, true);
});

watch(isEditMode, () => {
  requestAnimationFrame(syncPaletteAnchorRect);
});
</script>

<template>
  <NextWorkbenchLayout
    brand="SeaLantern"
    :rail-label="i18n.t('shell.rail_label')"
    :page-title="i18n.t('shell.page_title')"
    :page-subtitle="pageSubtitle"
    :logout-label="i18n.t('shell.logout')"
    :nav-items="navItems"
    :rail-locked="isEditMode"
    @logout="handleLogout"
  >
    <template #sidebar-primary>
      <NextSidebarHostItems :items="hostSidebarItems" />
    </template>

    <template #header-primary-actions>
      <button ref="editToggleRef" class="next-workbench-view__edit-toggle" type="button" :title="isEditMode ? i18n.t('shell.home_edit_done') : i18n.t('shell.home_edit_start')" :aria-pressed="isEditMode" @click="toggleEditMode">
        <Check v-if="isEditMode" :size="16" />
        <Pencil v-else :size="16" />
      </button>
    </template>

    <NextHomePage
      :instance-host="instanceHost"
      :preview-mode="isPreviewMode"
      :sidebar-entry-count="hostSidebarItems.length"
      :edit-mode="isEditMode"
      :palette-anchor-rect="paletteAnchorRect"
    />
  </NextWorkbenchLayout>
</template>

<style scoped>
.next-workbench-view__edit-toggle {
  width: 42px;
  height: 42px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-surface) 88%, transparent);
  color: var(--sl-text-primary);
  cursor: pointer;
}

.next-workbench-view__edit-toggle[aria-pressed="true"] {
  color: var(--sl-primary);
  border-color: color-mix(in srgb, var(--sl-primary) 36%, white);
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
}
</style>
