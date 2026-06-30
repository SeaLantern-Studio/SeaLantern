<script setup lang="ts">
import { computed } from "vue";
import { NextHomePluginCardRenderer } from "../../pages/home/builtinCardRenderers";
import type { NextHomeCardRendererProps, NextHomeCardRendererRegistry } from "../../pages/home/cardRendererContract";

const props = defineProps<NextHomeCardRendererProps & {
  registry: NextHomeCardRendererRegistry;
}>();

const resolvedRenderer = computed(() => props.registry[props.instance.kind] ?? NextHomePluginCardRenderer);
</script>

<template>
  <component :is="resolvedRenderer" :instance="instance" :meta="meta" :context="context" />
</template>

