<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, watch } from "vue";
import { useRoute } from "vue-router";
import { i18n } from "@language";
import NextShellEditToggleButton from "../components/shell/NextShellEditToggleButton.vue";
import { useNextHostRuntime } from "../host/runtime";
import NextHomePage from "../pages/home/NextHomePage.vue";
import NextProtectedPageView from "./NextProtectedPageView.vue";

const route = useRoute();
const nextHostRuntime = useNextHostRuntime();

const instanceHost = computed(() => window.location.host);
const hostSidebarItems = computed(() => nextHostRuntime.sidebarItems.value);
const isPreviewMode = computed(() => route.query.preview === "1");
const isEditMode = shallowRef(false);
const editToggleRef = shallowRef<InstanceType<typeof NextShellEditToggleButton> | null>(null);
const paletteAnchorRect = shallowRef<{
  top: number;
  left: number;
  width: number;
  height: number;
} | null>(null);

function toggleEditMode(): void {
  isEditMode.value = !isEditMode.value;
}

function syncPaletteAnchorRect(): void {
  const element = editToggleRef.value?.getElement();
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
  <NextProtectedPageView :rail-locked="isEditMode">
    <template #header-primary-actions>
      <NextShellEditToggleButton
        ref="editToggleRef"
        :active="isEditMode"
        :start-label="i18n.t('shell.home_edit_start')"
        :done-label="i18n.t('shell.home_edit_done')"
        @toggle="toggleEditMode"
      />
    </template>

    <NextHomePage
      :instance-host="instanceHost"
      :preview-mode="isPreviewMode"
      :sidebar-entry-count="hostSidebarItems.length"
      :edit-mode="isEditMode"
      :palette-anchor-rect="paletteAnchorRect"
    />
  </NextProtectedPageView>
</template>
