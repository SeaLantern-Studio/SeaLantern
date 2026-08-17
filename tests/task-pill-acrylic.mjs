import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const source = await readFile(
  new URL("../src/components/layout/TaskPill.vue", import.meta.url),
  "utf8",
);

test("任务详情面板跟随全局高级材质模糊设置", () => {
  assert.match(
    source,
    /\[data-acrylic="on"\] \.pill-panel[\s\S]*?backdrop-filter:\s*blur\(var\(--sl-acrylic-blur\)\)/,
  );
  assert.match(source, /\[data-acrylic="off"\] \.pill-panel[\s\S]*?backdrop-filter:\s*none/);
});
