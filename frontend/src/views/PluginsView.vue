<script setup lang="ts">
import PluginsWorkspaceShell from "@src/components/plugins/PluginsWorkspaceShell.vue";
import PluginsPage from "@src/pages/plugins/PluginsPage.vue";
import NextProtectedPageView from "@src/views/NextProtectedPageView.vue";
</script>

<template>
  <NextProtectedPageView>
    <PluginsWorkspaceShell>
      <PluginsPage />
    </PluginsWorkspaceShell>
  </NextProtectedPageView>
</template>
