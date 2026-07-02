<script setup lang="ts">
import PluginsWorkspaceShell from "@src/components/plugins/PluginsWorkspaceShell.vue";
import PluginCategoryPage from "@src/pages/plugins/PluginCategoryPage.vue";
import NextProtectedPageView from "@src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <PluginsWorkspaceShell>
      <PluginCategoryPage />
    </PluginsWorkspaceShell>
  </NextProtectedPageView>
</template>
