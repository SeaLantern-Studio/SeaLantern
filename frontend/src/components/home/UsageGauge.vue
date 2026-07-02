<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  value: number;
  tone: "primary" | "success" | "warning";
}>();

const radius = 22;
const circumference = 2 * Math.PI * radius;

const normalizedValue = computed(() => {
  if (!Number.isFinite(props.value)) return 0;
  return Math.min(100, Math.max(0, Math.round(props.value)));
});

const dashOffset = computed(() => circumference - (normalizedValue.value / 100) * circumference);
</script>

<template>
  <div class="usage-gauge" :class="`usage-gauge--${tone}`">
    <svg class="usage-gauge-svg" viewBox="0 0 56 56" aria-hidden="true">
      <circle class="usage-gauge-track" cx="28" cy="28" :r="radius" />
      <circle
        class="usage-gauge-fill"
        cx="28"
        cy="28"
        :r="radius"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="dashOffset"
      />
    </svg>
    <span class="usage-gauge-value">{{ normalizedValue }}%</span>
  </div>
</template>

<style scoped>
.usage-gauge {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.usage-gauge-svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
  overflow: visible;
}

.usage-gauge-track,
.usage-gauge-fill {
  fill: none;
  stroke-width: 6;
}

.usage-gauge-track {
  stroke: var(--sl-border);
}

.usage-gauge-fill {
  stroke-linecap: round;
  transition: stroke-dashoffset 0.3s ease;
}

.usage-gauge--primary .usage-gauge-fill {
  stroke: var(--sl-primary);
}

.usage-gauge--success .usage-gauge-fill {
  stroke: var(--sl-success);
}

.usage-gauge--warning .usage-gauge-fill {
  stroke: var(--sl-warning);
}

.usage-gauge-value {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8125rem;
  font-weight: 600;
  font-family: var(--sl-font-mono);
  color: var(--sl-text-primary);
}
</style>
