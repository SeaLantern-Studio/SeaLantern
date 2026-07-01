<script setup lang="ts">
import type { NextShellPage } from "../contracts/page";
import type { NextShellNavItem } from "../contracts/shell";
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
  railLocked?: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  logout: [];
}>();
</script>

<template>
  <NextShellFrame
    :brand="brand"
    :rail-label="railLabel"
    :page-title="page.title"
    :page-subtitle="page.subtitle || ''"
    :logout-label="logoutLabel"
    :show-logout="showLogout"
    :nav-items="navItems"
    :rail-locked="railLocked"
    @logout="emit('logout')"
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
      <header class="next-workbench-layout__page-header">
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
.next-workbench-layout__page {
  min-width: 0;
  display: grid;
  gap: 18px;
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
}

.next-workbench-layout__page-title {
  display: grid;
  gap: 6px;
}

@media (max-width: 767px) {
  .next-workbench-layout__page-header {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
