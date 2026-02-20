<script setup lang="ts">
interface Props {
  name: string;
  activeTab?: string;
}

const props = defineProps<Props>();
</script>

<template>
  <div v-show="props.activeTab === props.name" class="sl-tab-panel" role="tabpanel">
    <slot />
  </div>
</template>

<style scoped>
.sl-tab-panel {
  outline: none;
}
</style>
