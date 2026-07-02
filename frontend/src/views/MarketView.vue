<script setup lang="ts">
import PluginsWorkspaceShell from "@src/components/plugins/PluginsWorkspaceShell.vue";
import PluginMarketPage from "@src/pages/plugins/PluginMarketPage.vue";
import NextProtectedPageView from "@src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <PluginsWorkspaceShell>
      <PluginMarketPage />
    </PluginsWorkspaceShell>
  </NextProtectedPageView>
</template>
