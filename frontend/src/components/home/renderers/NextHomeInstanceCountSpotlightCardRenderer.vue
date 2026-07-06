<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import NextHomeMetricSpotlightCard from "../NextHomeMetricSpotlightCard.vue";
import type { NextHomeCardRendererProps } from "@src/pages/home/cardRendererContract";

const props = defineProps<NextHomeCardRendererProps>();

const metric = computed(() => props.context.instanceCountMetric);
</script>

<template>
  <NextHomeMetricSpotlightCard
    v-if="metric"
    :title="i18n.t('shell.home_card_instance_count_title')"
    :metric="metric"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
  />
</template>
