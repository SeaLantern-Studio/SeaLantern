<script setup lang="ts">
import { computed } from "vue";
import { i18n } from "@language";
import NextHomeWorkspaceBand from "../NextHomeWorkspaceBand.vue";
import type { NextHomeCardRendererProps } from "@next-src/pages/home/cardRendererContract";

const props = defineProps<NextHomeCardRendererProps>();

const serverCountLabel = computed(() => {
  if (props.context.totalServerCount > 0) {
    return i18n.t("shell.home_workspace_count_label", { count: props.context.totalServerCount });
  }
  return i18n.t("shell.home_workspace_empty_title");
});
</script>

<template>
  <NextHomeWorkspaceBand
    :featured-server="context.featuredServer"
    :secondary-servers="context.secondaryServers"
    :server-count-label="serverCountLabel"
    :empty-title="i18n.t('shell.home_workspace_empty_title')"
    :empty-description="i18n.t('shell.home_workspace_empty_description')"
    :preview-dataset="context.usingPreviewFallback"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
    @toggle="context.toggleServer"
  />
</template>
