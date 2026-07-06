<script setup lang="ts">
import { computed } from "vue";
import { resolveNextHomeCardRenderer } from "@src/pages/home/cardRendererRegistry";
import type {
  NextHomeCardRendererProps,
  NextHomeCardRendererRegistry,
  NextHomeResolvedCardRenderer,
} from "@src/pages/home/cardRendererContract";

defineOptions({
  inheritAttrs: false,
});

const props = defineProps<
  NextHomeCardRendererProps & {
    registry: NextHomeCardRendererRegistry;
  }
>();

const resolvedRenderer = computed<NextHomeResolvedCardRenderer>(() =>
  resolveNextHomeCardRenderer(props.instance.kind, props.registry),
);
</script>

<template>
  <component
    v-if="resolvedRenderer.status === 'registered'"
    :is="resolvedRenderer.component"
    :instance="instance"
    :meta="meta"
    :context="context"
  />
  <component
    v-else
    :is="resolvedRenderer.component"
    :instance="instance"
    :meta="meta"
    :context="context"
    :resolution="resolvedRenderer"
  />
</template>
