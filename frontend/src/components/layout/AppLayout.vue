<script setup lang="ts">
import { computed } from "vue";
import AppSidebar from "@components/layout/AppSidebar.vue";
import AppHeader from "@components/layout/AppHeader.vue";
import { useClientSettingsSync } from "@composables/useClientSettingsSync";
import { useShellBackground } from "@composables/useShellBackground";
import { useUiStore } from "@stores/uiStore";

const ui = useUiStore();
const { backgroundStyle, hasTransparentWindowBackdrop } = useShellBackground();

useClientSettingsSync();

const layoutClasses = computed(() => ({
  "has-native-window-effect": hasTransparentWindowBackdrop.value,
}));

const mainClasses = computed(() => ({
  "has-native-window-effect": hasTransparentWindowBackdrop.value,
  "sidebar-collapsed": ui.sidebarCollapsed,
}));
</script>

<template>
  <div class="app-layout" :class="layoutClasses">
    <div class="app-background" :style="backgroundStyle"></div>
    <AppSidebar />
    <div class="app-main" :class="mainClasses">
      <AppHeader />
      <main class="app-content">
        <router-view v-slot="{ Component }">
          <transition name="page-fade" mode="out-in">
            <keep-alive :max="5">
              <component :is="Component" />
            </keep-alive>
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<style src="@styles/components/layout/AppLayout.css" scoped></style>
