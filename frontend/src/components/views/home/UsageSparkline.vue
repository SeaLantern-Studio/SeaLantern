<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  values: number[];
  tone: "primary" | "success" | "warning";
}>();

const normalizedValues = computed(() => {
  const values = props.values.length === 0 ? [0] : props.values;

  const normalized = values.map((value) => {
    if (!Number.isFinite(value)) return 0;
    return Math.min(100, Math.max(0, value));
  });

  if (normalized.length === 1) {
    return [normalized[0], normalized[0]];
  }

  return normalized;
});

const linePoints = computed(() => {
  const values = normalizedValues.value;
  const lastIndex = Math.max(values.length - 1, 1);

  return values
    .map((value, index) => {
      const x = (index / lastIndex) * 100;
      const y = 100 - value;
      return `${x},${y}`;
    })
    .join(" ");
});

const areaPoints = computed(() => {
  const points = linePoints.value;
  return `0,100 ${points} 100,100`;
});
</script>

<template>
  <svg
    class="usage-sparkline"
    :class="`usage-sparkline--${tone}`"
    viewBox="0 0 100 100"
    preserveAspectRatio="none"
    aria-hidden="true"
  >
    <polygon class="usage-sparkline-area" :points="areaPoints" />
    <polyline class="usage-sparkline-line" :points="linePoints" />
  </svg>
</template>

<style scoped>
.usage-sparkline {
  width: 100%;
  height: 100%;
}

.usage-sparkline-area {
  opacity: 0.18;
}

.usage-sparkline-line {
  fill: none;
  stroke-width: 5;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.usage-sparkline--primary .usage-sparkline-area,
.usage-sparkline--primary .usage-sparkline-line {
  color: var(--sl-primary);
  fill: var(--sl-primary);
  stroke: var(--sl-primary);
}

.usage-sparkline--success .usage-sparkline-area,
.usage-sparkline--success .usage-sparkline-line {
  color: var(--sl-success);
  fill: var(--sl-success);
  stroke: var(--sl-success);
}

.usage-sparkline--warning .usage-sparkline-area,
.usage-sparkline--warning .usage-sparkline-line {
  color: var(--sl-warning);
  fill: var(--sl-warning);
  stroke: var(--sl-warning);
}
</style>
