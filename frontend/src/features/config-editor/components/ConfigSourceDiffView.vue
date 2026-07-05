<script setup lang="ts">
import { computed, ref } from "vue";
import { usePropertiesDiffViewer } from "@src/features/config-editor/usePropertiesDiffViewer";

interface Props {
  original: string;
  modified: string;
}

const props = defineProps<Props>();

const editorRoot = ref<HTMLElement | null>(null);
usePropertiesDiffViewer({
  editorRoot,
  original: computed(() => props.original),
  modified: computed(() => props.modified),
});
</script>

<template>
  <div ref="editorRoot" class="source-diff-cm-root"></div>
</template>

<style scoped>
.source-diff-cm-root {
  min-height: 0;
}
</style>
