<script setup lang="ts">
import { computed, useSlots } from "vue";
import type { NextShellPage } from "../contracts/page";
import type { NextShellNavItem, NextShellRailPinControl } from "../contracts/shell";
import { NEXT_HOST_SLOT_IDS } from "../host/slotIds";
import NextHostSlot from "../components/host/NextHostSlot.vue";
import NextShellFrame from "../components/shell/NextShellFrame.vue";

interface Props {
  brand: string;
  railLabel: string;
  page: NextShellPage;
  logoutLabel: string;
  showLogout?: boolean;
  navItems: NextShellNavItem[];
  railPinControl?: NextShellRailPinControl | null;
  railLocked?: boolean;
  railExpanded?: boolean;
  pageTransitionClass?: string | null;
  shellHeaderMode?: "title" | "hidden";
}

const props = withDefaults(defineProps<Props>(), {
  showLogout: true,
  railPinControl: null,
  railLocked: false,
  railExpanded: false,
  pageTransitionClass: null,
  shellHeaderMode: "title",
});

const slots = useSlots();

const hasPageHeader = computed(() => Boolean(slots.pageHeader));
const hasPageHeaderActions = computed(() => Boolean(slots["page-header-actions"]));
const showPageHeader = computed(() => hasPageHeader.value || hasPageHeaderActions.value);
const layoutClasses = computed(() => ({
  "next-workbench-layout": true,
  "next-workbench-layout--shell-header-hidden": props.shellHeaderMode === "hidden",
}));

const emit = defineEmits<{
  logout: [];
  pageTransitionSettled: [];
  railFocusWithinChange: [value: boolean];
  railPointerInsideChange: [value: boolean];
  toggleRailPin: [];
}>();
</script>

<template>
  <NextShellFrame
    :class="layoutClasses"
    :brand="brand"
    :rail-label="railLabel"
    :page-title="page.title"
    :page-subtitle="page.subtitle || ''"
    :logout-label="logoutLabel"
    :show-logout="showLogout"
    :nav-items="navItems"
    :rail-pin-control="railPinControl"
    :rail-locked="railLocked"
    :rail-expanded="railExpanded"
    :page-transition-class="pageTransitionClass"
    @logout="emit('logout')"
    @page-transition-settled="emit('pageTransitionSettled')"
    @rail-focus-within-change="emit('railFocusWithinChange', $event)"
    @rail-pointer-inside-change="emit('railPointerInsideChange', $event)"
    @toggle-rail-pin="emit('toggleRailPin')"
  >
    <template #sidebar-primary>
      <NextHostSlot :slot-id="NEXT_HOST_SLOT_IDS.sidebarPrimary" scope="shell">
        <slot name="sidebar-primary" />
      </NextHostSlot>
    </template>

    <template #header-primary-actions>
      <NextHostSlot :slot-id="NEXT_HOST_SLOT_IDS.headerPrimaryActions" scope="shell">
        <slot name="header-primary-actions" />
      </NextHostSlot>
    </template>

    <section class="next-workbench-layout__page">
      <header v-if="showPageHeader" class="next-workbench-layout__page-header">
        <div class="next-workbench-layout__page-title">
          <slot name="page-header" />
        </div>

        <NextHostSlot :slot-id="NEXT_HOST_SLOT_IDS.pageHeaderActions" scope="page">
          <slot name="page-header-actions" />
        </NextHostSlot>
      </header>

      <NextHostSlot :slot-id="NEXT_HOST_SLOT_IDS.pageContentBefore" scope="page">
        <slot name="page-content-before" />
      </NextHostSlot>

      <div class="next-workbench-layout__page-body">
        <slot />
      </div>

      <NextHostSlot :slot-id="NEXT_HOST_SLOT_IDS.pageContentAfter" scope="page">
        <slot name="page-content-after" />
      </NextHostSlot>
    </section>

    <template #overlay>
      <NextHostSlot :slot-id="NEXT_HOST_SLOT_IDS.overlayGlobal" scope="overlay">
        <slot name="overlay" />
      </NextHostSlot>
    </template>
  </NextShellFrame>
</template>

<style scoped>
.next-workbench-layout {
  min-width: 0;
}

.next-workbench-layout__page {
  min-width: 0;
  display: grid;
  gap: 20px;
}

.next-workbench-layout__page-header {
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.next-workbench-layout__page-title,
.next-workbench-layout__page-body {
  min-width: 0;
  display: grid;
  gap: 20px;
}

.next-workbench-layout__page-title {
  display: grid;
  gap: 6px;
}

.next-workbench-layout :deep(.next-desktop-titlebar) {
  position: sticky;
  top: 0;
  z-index: 5;
  background: color-mix(in srgb, var(--sl-bg) 76%, transparent);
  backdrop-filter: blur(18px);
}

.next-workbench-layout :deep(.next-desktop-titlebar__copy),
.next-workbench-layout :deep(.next-shell-frame__header-eyebrow) {
  display: none;
}

.next-workbench-layout--shell-header-hidden :deep(.next-shell-frame__header) {
  display: none;
}

@media (max-width: 767px) {
  .next-workbench-layout__page-header {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
