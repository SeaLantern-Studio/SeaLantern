<script setup lang="ts">
import ServerInstanceConfigPage from "@src/pages/server-instance/config/ServerInstanceConfigPage.vue";
import NextInstanceWorkspaceView from "@src/views/NextInstanceWorkspaceView.vue";
</script>

<template>
  <NextInstanceWorkspaceView section="config">
    <ServerInstanceConfigPage />
  </NextInstanceWorkspaceView>
</template>
