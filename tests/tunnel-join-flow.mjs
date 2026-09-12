import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const source = await readFile(new URL("../src/views/TunnelView.vue", import.meta.url), "utf8");

test("点击加入后立即进入连接页,而不是等命令返回", async () => {
  // 后端 join 立即返回 starting，本地乐观阶段只是让切页再早一拍。
  assert.match(
    source,
    /pendingPhase\.value = "starting";\s*\n\s*try \{\s*\n\s*const snapshot = await tunnelApi\.join/,
  );
  assert.match(
    source,
    /const currentPhase = computed\(\(\) => pendingPhase\.value \?\? status\.value\?\.phase/,
  );
  assert.match(source, /const isIdle = computed\(\(\) => currentPhase\.value === "idle"\)/);
});

test("建链阶段允许取消连接", async () => {
  // 停止按钮不能挂在 running(=== active) 上,否则建链期间按钮是灰的、点了没反应。
  assert.match(
    source,
    /const hasSession = computed\(\(\) => isStarting\.value \|\| status\.value\?\.mode != null\)/,
  );
  assert.match(source, /const canStopTunnel = computed\(\s*\(\) =>\s*hasSession\.value/);
  assert.match(source, /const isCancellable = computed\(\(\) => isStarting\.value\)/);
  // 取消与就绪竞争时以取消为准,否则会把已停止的隧道重新显示成已连接。
  assert.match(source, /if \(stopRequested\) \{\s*\n\s*void refreshStatus/);
  // 停止就是一次原生 stop，前端不得再加等待/重试拖慢它。
  assert.doesNotMatch(source, /stopWithRetry|await new Promise/);
});
