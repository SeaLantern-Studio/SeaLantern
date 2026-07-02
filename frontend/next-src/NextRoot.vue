<script setup lang="ts">
import { RouterView } from "vue-router";
import { isBrowserEnv } from "@api/tauri";
import AppShellGlobalHost from "@components/startup/AppShellGlobalHost.vue";
import { useClientSettingsSync } from "@composables/useClientSettingsSync";
import { provideNextHostRuntime } from "./host/runtime";

const isDesktopShell = !isBrowserEnv();

useClientSettingsSync();

provideNextHostRuntime();
</script>

<template>
  <AppShellGlobalHost
    :bootstrap-enabled="isDesktopShell"
    :enable-desktop-shell-features="isDesktopShell"
  >
    <RouterView />
  </AppShellGlobalHost>
</template>
