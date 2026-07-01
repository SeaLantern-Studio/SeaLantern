<script setup lang="ts">
import { i18n } from "@language";
import NextHomeAttentionBand from "../NextHomeAttentionBand.vue";
import type { NextHomeCardRendererProps } from "@next-src/pages/home/cardRendererContract";

defineProps<NextHomeCardRendererProps>();
</script>

<template>
  <NextHomeAttentionBand
    :alerts="context.alertItems"
    :title="i18n.t('home.recent_alerts')"
    :empty-title="i18n.t('shell.home_attention_empty_title')"
    :empty-description="i18n.t('shell.home_attention_empty_description')"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
  />
</template>
