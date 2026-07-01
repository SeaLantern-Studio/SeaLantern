<script setup lang="ts">
import { i18n } from "@language";
import NextHomeSystemOverviewPanel from "../NextHomeSystemOverviewPanel.vue";
import type { NextHomeCardRendererProps } from "@next-src/pages/home/cardRendererContract";

defineProps<NextHomeCardRendererProps>();
</script>

<template>
  <NextHomeSystemOverviewPanel
    :title="i18n.t(meta.titleKey)"
    :metrics="context.systemMetrics"
    :view-mode="context.statsViewMode"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
    @toggle-view="context.toggleStatsViewMode"
  />
</template>
