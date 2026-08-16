// 临时脚本：对比各语言文件与 zh-CN 基准的 key 覆盖情况
const fs = require("fs");
const path = require("path");

const dir = path.join(__dirname, "..", "src", "language");
const files = fs.readdirSync(dir).filter((f) => f.endsWith(".json"));

// 递归收集所有叶子 key 路径
function collectKeys(obj, prefix = "") {
  const keys = [];
  for (const [k, v] of Object.entries(obj)) {
    const p = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === "object" && !Array.isArray(v)) {
      keys.push(...collectKeys(v, p));
    } else {
      keys.push(p);
    }
  }
  return keys;
}

const base = JSON.parse(fs.readFileSync(path.join(dir, "zh-CN.json"), "utf8"));
const baseKeys = new Set(collectKeys(base));

for (const file of files) {
  if (file === "zh-CN.json") continue;
  const data = JSON.parse(fs.readFileSync(path.join(dir, file), "utf8"));
  const keys = new Set(collectKeys(data));
  const missing = [...baseKeys].filter((k) => !keys.has(k));
  // 检查值是否仍是中文（未翻译），只统计叶子为非 languageName 的
  const chineseLeft = missing.length ? missing.map((k) => k) : [];
  if (missing.length) {
    console.log(`\n=== ${file} 缺失 ${missing.length} 个 key ===`);
    console.log(missing.join("\n"));
  } else {
    console.log(`${file}: OK`);
  }
}
