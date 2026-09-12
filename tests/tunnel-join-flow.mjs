import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const source = await readFile(new URL("../src/views/TunnelView.vue", import.meta.url), "utf8");

test("点击加入后立即进入连接页,而不是等命令返回", async () => {
  // 后端 join 立即返回 starting，本地乐观阶段只是让切页再早一拍。
  assert.match(
    source,
    /pendingPhase\.value = "starting";\s*\n\s*commandsInFlight\.value \+= 1;\s*\n\s*try \{\s*\n\s*const snapshot = await tunnelApi\.join/,
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

test("停止请求不能因为已有 pending 动作而被吞掉", async () => {
  // 回归：stopTunnel 曾经用 beginAction("stop")，建房进行中 pendingAction 是 "host"，
  // 于是点击「取消连接」直接 return、请求根本没发出去。
  assert.doesNotMatch(source, /if \(!beginAction\("stop"\)\) return/);
  assert.match(
    source,
    /async function stopTunnel\(\) \{\s*\n[^}]*?if \(!canStopTunnel\.value\) return;/,
  );
  // 后端确认空闲后立刻退出过渡态，不必等 host/join 命令收尾。
  assert.match(
    source,
    /applyStatus\(await tunnelApi\.stop\(\)\);\s*\n\s*\/\/[^\n]*\n\s*clearPendingPhase\(\)/,
  );
  // 取消没生效、隧道终究起来时，停止按钮要恢复可用。
  assert.match(
    source,
    /watch\(running, \(value\) => \{\s*\n\s*if \(value\) stopRequested = false;/,
  );
});

test("取消后旧命令收尾期间禁用建房与加入", async () => {
  // 回归：取消只让界面回到表单，host 命令仍在 bind，后端依旧 Busy——
  // 此时按钮必须禁用，而不是让用户点了才收到「隧道正在处理中」。
  assert.match(source, /const commandsInFlight = ref\(0\)/);
  assert.match(source, /const hasPendingCommand = computed\(\(\) => commandsInFlight\.value > 0\)/);
  assert.match(
    source,
    /const canStartHost = computed\(\s*\(\) => isIdle\.value && !isBusy\.value && !hasPendingCommand\.value,?\s*\)/,
  );
  assert.match(
    source,
    /const canStartJoin = computed\(\s*\(\) => isIdle\.value && !isBusy\.value && !hasPendingCommand\.value,?\s*\)/,
  );
  // host / join / stop 三条命令都要在收尾期间计入在飞行数。
  assert.equal(source.match(/commandsInFlight\.value \+= 1;/g)?.length, 3);
  assert.equal(source.match(/commandsInFlight\.value -= 1;/g)?.length, 3);
});
