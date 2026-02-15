<script setup lang="ts">
interface Props {
  size?: "sm" | "md" | "lg";
}

const props = withDefaults(defineProps<Props>(), {
  size: "md",
});

const sizeMap = {
  sm: "16px",
  md: "20px",
  lg: "24px",
};
</script>

<template>
  <div class="sl-spinner" :style="{ width: sizeMap[size], height: sizeMap[size] }" />
</template>

<style scoped>
.sl-spinner {
  border: 2px solid var(--sl-border);
  border-top-color: var(--sl-primary);
  border-radius: 50%;
  animation: sl-spin 0.8s linear infinite;
}
</style>
