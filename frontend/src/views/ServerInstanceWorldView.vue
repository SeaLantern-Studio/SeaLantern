<script setup lang="ts">
import { i18n } from "@language";
import NextInstanceWorkspaceView from "@src/views/NextInstanceWorkspaceView.vue";
</script>

<template>
  <NextInstanceWorkspaceView section="world">
    <section class="server-instance-world-view">
      <p class="server-instance-world-view__notice">
        {{ i18n.t("servers.next.instance.world.developing") }}
      </p>
    </section>
  </NextInstanceWorkspaceView>
</template>

<style scoped>
.server-instance-world-view {
  display: grid;
}

.server-instance-world-view__notice {
  margin: 0;
  color: var(--sl-text-secondary);
  line-height: 1.6;
}
</style>
