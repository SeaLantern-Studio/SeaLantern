<script setup lang="ts">
import NextHomeSummaryBand from "../NextHomeSummaryBand.vue";
import type { NextHomeCardRendererProps } from "@src/pages/home/cardRendererContract";

defineProps<NextHomeCardRendererProps>();
</script>

<template>
  <NextHomeSummaryBand
    :metrics="context.summaryMetrics"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
  />
</template>
