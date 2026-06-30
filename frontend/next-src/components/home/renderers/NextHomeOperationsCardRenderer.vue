<script setup lang="ts">
import { i18n } from "@language";
import NextHomeOperationsBand from "../NextHomeOperationsBand.vue";
import type { NextHomeCardRendererProps } from "../../../pages/home/cardRendererContract";

defineProps<NextHomeCardRendererProps>();
</script>

<template>
  <NextHomeOperationsBand
    :refresh-label="i18n.t('shell.action_refresh')"
    :create-label="i18n.t('common.create_server')"
    :import-label="i18n.t('common.add_existing_server')"
    :last-updated-label="context.lastUpdatedLabel"
    :is-refreshing="context.isRefreshing"
    :card-id="instance.instanceId"
    :section="meta.section"
    :editable="meta.movable"
    @create="context.goToCreateServer"
    @import="context.goToImportServer"
    @refresh="context.refreshNow"
  />
</template>

