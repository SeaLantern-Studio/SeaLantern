<script setup lang="ts">
import NextHomeSystemOverviewPanel from "../NextHomeSystemOverviewPanel.vue";
import { resolveNextHomeCardTitle } from "@src/pages/home/cardMeta";
import type { NextHomeCardRendererProps } from "@src/pages/home/cardRendererContract";

defineProps<NextHomeCardRendererProps>();
</script>

<template>
  <NextHomeSystemOverviewPanel
    :title="resolveNextHomeCardTitle(meta)"
    :metrics="context.systemMetrics"
    :view-mode="context.statsViewMode"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
    @toggle-view="context.toggleStatsViewMode"
  />
</template>
