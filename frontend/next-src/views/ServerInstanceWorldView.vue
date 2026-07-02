<script setup lang="ts">
import ServerInstancePlaceholderPage from "@next-src/components/server-instance/ServerInstancePlaceholderPage.vue";
import NextInstanceWorkspaceView from "@next-src/views/NextInstanceWorkspaceView.vue";
</script>

<template>
  <NextInstanceWorkspaceView section="world">
    <ServerInstancePlaceholderPage />
  </NextInstanceWorkspaceView>
</template>
