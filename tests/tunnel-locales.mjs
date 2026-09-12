import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import test from "node:test";

const directory = new URL("../src/language/", import.meta.url);

/** 复刻 src/language/index.ts 的扁平化顺序：sealantern 子树优先，先命中者优先。 */
function flatten(source) {
  const flat = new Map();
  const walk = (node, prefix) => {
    for (const [key, value] of Object.entries(node)) {
      if (value && typeof value === "object") {
        walk(value, [...prefix, key]);
      } else if (typeof value === "string" && !flat.has([...prefix, key].join("."))) {
        flat.set([...prefix, key].join("."), value);
      }
    }
  };
  if (source.sealantern && typeof source.sealantern === "object") walk(source.sealantern, []);
  walk(source, []);
  return flat;
}

async function loadLocales() {
  const files = (await readdir(directory)).filter((name) => name.endsWith(".json"));
  const entries = await Promise.all(
    files.map(async (file) => {
      const source = JSON.parse(await readFile(new URL(file, directory), "utf8"));
      return [file.replace(/\.json$/, ""), flatten(source)];
    }),
  );
  return new Map(entries);
}

test("联机页文案在每个语言里都能解析到", async () => {
  const locales = await loadLocales();
  const baseline = locales.get("en-US");
  assert.ok(baseline, "缺少 en-US 基线");

  const tunnelKeys = [...baseline.keys()].filter((key) => key.startsWith("tunnel."));
  assert.ok(tunnelKeys.length > 20, "联机页文案数量异常");

  for (const [locale, flat] of locales) {
    const missing = tunnelKeys.filter((key) => !flat.has(key));
    assert.deepEqual(missing, [], `${locale} 缺少联机页文案: ${missing.join(", ")}`);
  }
});

test("联机页新增文案都带占位符或非空内容", async () => {
  const locales = await loadLocales();
  const required = [
    "tunnel.minecraft_address",
    "tunnel.allocating_port",
    "tunnel.copy_address",
    "tunnel.copied",
    "tunnel.join_ready",
    "tunnel.join_hint",
    "tunnel.join_route",
    "tunnel.detecting",
    "tunnel.sent",
    "tunnel.received",
    "tunnel.syncing",
    "tunnel.latency_low",
    "tunnel.latency_medium",
    "tunnel.latency_high",
  ];

  for (const [locale, flat] of locales) {
    for (const key of required) {
      const value = flat.get(key);
      assert.equal(typeof value, "string", `${locale} 缺少 ${key}`);
      assert.ok(value.trim().length > 0, `${locale} 的 ${key} 是空文案`);
    }
  }
});
