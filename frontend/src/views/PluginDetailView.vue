<script setup lang="ts">
import PluginsWorkspaceShell from "@src/components/plugins/PluginsWorkspaceShell.vue";
import PluginDetailPage from "@src/pages/plugins/PluginDetailPage.vue";
import NextProtectedPageView from "@src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <PluginsWorkspaceShell>
      <PluginDetailPage />
    </PluginsWorkspaceShell>
  </NextProtectedPageView>
</template>
