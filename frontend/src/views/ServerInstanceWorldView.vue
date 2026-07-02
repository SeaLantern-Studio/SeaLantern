<script setup lang="ts">
import ServerInstancePlaceholderPage from "@src/components/server-instance/ServerInstancePlaceholderPage.vue";
import NextInstanceWorkspaceView from "@src/views/NextInstanceWorkspaceView.vue";
</script>

<template>
  <NextInstanceWorkspaceView section="world">
    <ServerInstancePlaceholderPage />
  </NextInstanceWorkspaceView>
</template>
