import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const source = await readFile(
  new URL("../src/components/console/ConsoleOutput.vue", import.meta.url),
  "utf8",
);

test("控制台显式指定高对比度主题色选区,避免回退到深色默认选中框", () => {
  const templateMatch = source.match(/<template>([\s\S]*?)<\/template>/);
  assert.ok(templateMatch, "未找到 <template> 块");
  const consoleTagMatch = templateMatch[1].match(/<cmz-console\b([\s\S]*?)(?:\/>|>)/);
  assert.ok(consoleTagMatch, "未找到 <cmz-console> 组件标签");
  assert.match(
    consoleTagMatch[1],
    /\bselection-color="color-mix\(in srgb, var\(--sl-primary\) 26%, transparent\)"/,
  );
});
