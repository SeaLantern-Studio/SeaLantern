<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import type { ECharts } from "echarts/core";

/** 采样间隔；sculk 的路径事件也按 1s 节流，两者节奏一致。 */
const SAMPLE_INTERVAL_MS = 1000;

const SAMPLE_LIMIT = 60;

/** 延迟分档阈值（毫秒），与趋势图背景色块一致。 */
const LOW_LATENCY_MS = 80;
const MEDIUM_LATENCY_MS = 180;

const props = defineProps<{
  value: number | null;
  label: string;
  /** 分档色块的文案；由调用方完成国际化。 */
  thresholds: { low: string; medium: string; high: string };
  /** 采样回调：状态轮询间隔远大于采样间隔，实时值回源到隧道事件。 */
  sample?: () => number | null;
}>();
const element = ref<HTMLDivElement | null>(null);
let chart: ECharts | null = null;
let resizeObserver: ResizeObserver | null = null;
let themeObserver: MutationObserver | null = null;
let sampleTimer: ReturnType<typeof setInterval> | null = null;
/** 动态 import 期间组件可能已卸载，用它避免在已卸载的实例上创建图表。 */
let disposed = false;
const samples: Array<[number, number]> = [];

/** 主题变量缺失时 ECharts 会画出「无色」图形，分档色块会整块消失。 */
const FALLBACK_COLORS: Record<string, string> = {
  "--sl-primary": "#0ea5e9",
  "--sl-border": "#e2e8f0",
  "--sl-text-tertiary": "#94a3b8",
  "--sl-text-primary": "#0f172a",
  "--sl-surface": "#ffffff",
  "--sl-success": "#22c55e",
  "--sl-warning": "#f59e0b",
  "--sl-error": "#ef4444",
};

function css(name: string) {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || FALLBACK_COLORS[name] || "";
}

function render() {
  if (!chart) return;
  const values = samples.map(([, value]) => value);
  const peak = Math.max(250, ...values, 0);
  const floor = values.length > 0 ? Math.min(...values) : 0;
  // 抖动较小时留余量，否则 1ms 的波动会占满整个纵轴。
  const max = peak + Math.max(5, Math.ceil((peak - floor) * 0.25));
  const primary = css("--sl-primary");
  const border = css("--sl-border");
  const muted = css("--sl-text-tertiary");
  const success = css("--sl-success");
  const warning = css("--sl-warning");
  const danger = css("--sl-error");
  const { low: lowLabel, medium: mediumLabel, high: highLabel } = props.thresholds;

  chart.setOption({
    animation: !window.matchMedia("(prefers-reduced-motion: reduce)").matches,
    animationDuration: 220,
    animationDurationUpdate: 260,
    animationEasingUpdate: "cubicOut",
    grid: { top: 12, right: 12, bottom: 18, left: 44 },
    tooltip: {
      trigger: "axis",
      backgroundColor: css("--sl-surface"),
      borderColor: border,
      borderWidth: 1,
      textStyle: { color: css("--sl-text-primary"), fontSize: 12 },
      valueFormatter: (value: number | string) => `${value} ms`,
    },
    xAxis: {
      type: "time",
      boundaryGap: false,
      axisLabel: { show: false },
      axisTick: { show: false },
      axisLine: { lineStyle: { color: border } },
      splitLine: { show: false },
    },
    yAxis: {
      type: "value",
      min: 0,
      max,
      axisLabel: { color: muted, fontSize: 10, formatter: "{value} ms" },
      axisTick: { show: false },
      axisLine: { show: false },
      splitLine: { lineStyle: { color: border, type: "dashed" } },
    },
    series: [
      {
        type: "line",
        data: samples,
        smooth: 0.35,
        showSymbol: false,
        lineStyle: { color: primary, width: 2 },
        itemStyle: { color: primary },
        areaStyle: { color: primary, opacity: 0.08 },
        markArea: {
          silent: true,
          label: { position: "insideRight", color: muted, fontSize: 10 },
          data: [
            [
              {
                name: lowLabel,
                yAxis: 0,
                itemStyle: { color: success, opacity: 0.12 },
              },
              { yAxis: LOW_LATENCY_MS },
            ],
            [
              {
                name: mediumLabel,
                yAxis: LOW_LATENCY_MS,
                itemStyle: { color: warning, opacity: 0.12 },
              },
              { yAxis: MEDIUM_LATENCY_MS },
            ],
            [
              {
                name: highLabel,
                yAxis: MEDIUM_LATENCY_MS,
                itemStyle: { color: danger, opacity: 0.1 },
              },
              { yAxis: max },
            ],
          ],
        },
      },
    ],
  });
}

async function setup() {
  if (!element.value || disposed) return;
  const [
    { init, use },
    { LineChart },
    { GridComponent, MarkAreaComponent, TooltipComponent },
    { CanvasRenderer },
  ] = await Promise.all([
    import("echarts/core"),
    import("echarts/charts"),
    import("echarts/components"),
    import("echarts/renderers"),
  ]);
  if (disposed || !element.value) return;
  use([CanvasRenderer, GridComponent, LineChart, MarkAreaComponent, TooltipComponent]);
  chart = init(element.value);
  render();
  resizeObserver = new ResizeObserver(() => chart?.resize());
  resizeObserver.observe(element.value);
}

/** 采样回调：实时值由隧道事件驱动，而非 5s 一次的状态快照。 */
function recordSample() {
  const next = props.sample ? props.sample() : props.value;
  if (next == null || !Number.isFinite(next)) return;
  const now = Date.now();
  const last = samples[samples.length - 1];
  // 同一毫秒内的重复采样会让相邻点在 x 轴上重叠。
  if (last && last[0] === now) {
    last[1] = next;
  } else {
    samples.push([now, next]);
    if (samples.length > SAMPLE_LIMIT) samples.shift();
  }
  render();
}

onMounted(() => {
  void setup();
  recordSample();
  sampleTimer = setInterval(recordSample, SAMPLE_INTERVAL_MS);
  // 主题切换会改配色变量，画布需要重绘。
  themeObserver = new MutationObserver(() => render());
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["class", "data-theme", "style"],
  });
});

onUnmounted(() => {
  disposed = true;
  if (sampleTimer) {
    clearInterval(sampleTimer);
    sampleTimer = null;
  }
  resizeObserver?.disconnect();
  resizeObserver = null;
  themeObserver?.disconnect();
  themeObserver = null;
  chart?.dispose();
  chart = null;
});

// props 变化只影响标题;曲线由采样心跳推进。
watch(
  () => props.value,
  () => render(),
);
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
