import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const read = (relative) => readFile(new URL(relative, import.meta.url), "utf8");

test("延迟趋势图的分档色块使用已定义的主题变量", async () => {
  const chart = await read("../src/components/views/tunnel/LatencyChart.vue");
  const variables = await read("../src/styles/variables.css");

  // 变量名拼错时 ECharts 会画出无色图形，分档色块整块消失，所以必须取色后传值。
  assert.match(chart, /const (success|warning|danger) = css\("--sl-(success|warning|error)"\)/);
  assert.doesNotMatch(chart, /css\("--sl-danger"\)/);
  assert.match(chart, /itemStyle: \{ color: success, opacity: [\d.]+ \}/);
  assert.match(chart, /itemStyle: \{ color: warning, opacity: [\d.]+ \}/);
  assert.match(chart, /itemStyle: \{ color: danger, opacity: [\d.]+ \}/);
  assert.match(chart, /FALLBACK_COLORS/);
  // 只注册 Grid/Line/Tooltip 时 markArea 不会渲染。
  assert.match(
    chart,
    /use\(\[CanvasRenderer, GridComponent, LineChart, MarkAreaComponent, TooltipComponent\]\)/,
  );

  for (const token of ["--sl-success:", "--sl-warning:", "--sl-error:"]) {
    assert.ok(variables.includes(token), `主题缺少 ${token}`);
  }
});

test("延迟趋势图按固定心跳采样实时值", async () => {
  const chart = await read("../src/components/views/tunnel/LatencyChart.vue");

  // 状态轮询是 5s，曲线不能再按轮询节奏推进；采样须优先取事件驱动的实时值。
  assert.match(chart, /SAMPLE_INTERVAL_MS = 1000/);
  assert.match(chart, /setInterval\(recordSample, SAMPLE_INTERVAL_MS\)/);
  assert.match(chart, /props\.sample \? props\.sample\(\) : props\.value/);
});

test("建房与加入卡片的操作按钮贴齐卡片底部", async () => {
  const css = await read("../src/styles/views/TunnelView.css");

  assert.match(css, /\.setup-grid :deep\(\.cmz-card-body\)\s*\{[^}]*flex-direction: column/);
  assert.match(
    css,
    /\.setup-grid :deep\(\.cmz-card-body\) \.card-actions\s*\{[^}]*margin-top: auto/,
  );
  assert.doesNotMatch(css, /sl-card-body/);
});
