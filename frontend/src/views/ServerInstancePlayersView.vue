<script setup lang="ts">
import ServerInstancePlayersPage from "@src/pages/server-instance/players/ServerInstancePlayersPage.vue";
import NextInstanceWorkspaceView from "@src/views/NextInstanceWorkspaceView.vue";
</script>

<template>
  <NextInstanceWorkspaceView section="players">
    <ServerInstancePlayersPage />
  </NextInstanceWorkspaceView>
</template>
