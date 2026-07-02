<script setup lang="ts">
import ServerInstancePlayersPage from "@next-src/pages/server-instance/players/ServerInstancePlayersPage.vue";
import NextInstanceWorkspaceView from "@next-src/views/NextInstanceWorkspaceView.vue";
</script>

<template>
  <NextInstanceWorkspaceView section="players">
    <ServerInstancePlayersPage />
  </NextInstanceWorkspaceView>
</template>
