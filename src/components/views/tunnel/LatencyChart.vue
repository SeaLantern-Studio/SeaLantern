<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import type { ECharts } from "echarts/core";

/** 保留的采样点数量；状态轮询间隔 5s，约覆盖最近 5 分钟。 */
const SAMPLE_LIMIT = 60;

const props = defineProps<{ value: number | null; label: string }>();
const element = ref<HTMLDivElement | null>(null);
let chart: ECharts | null = null;
let resizeObserver: ResizeObserver | null = null;
/** 动态 import 期间组件可能已卸载，用它避免在已卸载的实例上创建图表。 */
let disposed = false;
const samples: Array<[number, number]> = [];

function css(name: string) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

function render() {
  if (!chart) return;
  const max = Math.max(250, ...samples.map(([, value]) => value), 0) + 20;
  chart.setOption({
    animation: !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    grid: { top: 12, right: 12, bottom: 18, left: 44 },
    tooltip: { trigger: "axis", valueFormatter: (value: number | string) => `${value} ms` },
    xAxis: {
      type: "time",
      boundaryGap: false,
      axisLabel: { show: false },
      axisTick: { show: false },
      axisLine: { lineStyle: { color: css("--sl-border") } },
      splitLine: { show: false },
    },
    yAxis: {
      type: "value",
      min: 0,
      max,
      axisLabel: { color: css("--sl-text-tertiary"), fontSize: 10, formatter: "{value} ms" },
      axisTick: { show: false },
      axisLine: { show: false },
      splitLine: { lineStyle: { color: css("--sl-border"), type: "dashed" } },
    },
    series: [
      {
        type: "line",
        data: samples,
        smooth: 0.3,
        showSymbol: false,
        lineStyle: { color: css("--sl-primary"), width: 2 },
        areaStyle: { color: css("--sl-primary"), opacity: 0.08 },
      },
    ],
  });
}

async function setup() {
  if (!element.value || disposed) return;
  const [{ init, use }, { LineChart }, { GridComponent, TooltipComponent }, { CanvasRenderer }] =
    await Promise.all([
      import("echarts/core"),
      import("echarts/charts"),
      import("echarts/components"),
      import("echarts/renderers"),
    ]);
  if (disposed || !element.value) return;
  use([CanvasRenderer, GridComponent, LineChart, TooltipComponent]);
  chart = init(element.value);
  render();
  resizeObserver = new ResizeObserver(() => chart?.resize());
  resizeObserver.observe(element.value);
}

/** 只在延迟真正变化时采样，避免用重复值把趋势线拉平。 */
watch(
  () => props.value,
  (value) => {
    if (value == null) return;
    samples.push([Date.now(), value]);
    if (samples.length > SAMPLE_LIMIT) samples.shift();
    render();
  },
  { immediate: true },
);

onMounted(() => {
  void setup();
});

onUnmounted(() => {
  disposed = true;
  resizeObserver?.disconnect();
  resizeObserver = null;
  chart?.dispose();
  chart = null;
});
</script>

<template>
  <section class="tunnel-latency-chart" :aria-label="props.label">
    <div class="latency-heading">
      <span>{{ props.label }}</span>
      <strong v-if="props.value == null">--</strong>
      <strong v-else>{{ props.value }} ms</strong>
    </div>
    <div ref="element" class="latency-canvas"></div>
  </section>
</template>

<style scoped>
.tunnel-latency-chart {
  display: flex;
  flex-direction: column;
  gap: var(--sl-space-sm);
}
.latency-heading {
  display: flex;
  justify-content: space-between;
  color: var(--sl-text-secondary);
  font-size: 0.85rem;
}
.latency-heading strong {
  color: var(--sl-text-primary);
}
.latency-canvas {
  width: 100%;
  height: 260px;
  min-height: 220px;
}
</style>
