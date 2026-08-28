import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tmpDir = path.join(rootDir, "docs", "tmp");
const frontMatterPattern = /^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/;
const expirationKey = "expiration-time";
const templateExpiration = "<time:yyyyMMdd,e.g.:20260901>";

async function discoverMarkdownFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nestedFiles = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        return discoverMarkdownFiles(entryPath);
      }
      return entry.isFile() && entry.name.endsWith(".md") ? [entryPath] : [];
    }),
  );
  return nestedFiles.flat().sort();
}

function relativePath(filePath) {
  return path.relative(rootDir, filePath).replaceAll(path.sep, "/");
}

function expirationText(value) {
  if (typeof value === "number" && Number.isInteger(value)) {
    return String(value);
  }
  return typeof value === "string" ? value.trim() : null;
}

function validateExpiration(value, filePath) {
  const expiration = expirationText(value);
  if (expiration === templateExpiration) {
    return path.basename(filePath) === "example.md"
      ? []
      : [`${expirationKey} 的示例占位值只能出现在 docs/tmp/example.md`];
  }

  if (!expiration || !/^\d{8}$/.test(expiration)) {
    return [`${expirationKey} 必须是 YYYYMMDD 格式`];
  }

  const year = Number(expiration.slice(0, 4));
  const month = Number(expiration.slice(4, 6));
  const day = Number(expiration.slice(6, 8));
  const date = new Date(Date.UTC(year, month - 1, day));
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    return [`${expirationKey} 不是有效日期`];
  }

  const now = new Date();
  const today = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate());
  return date.getTime() < today ? [`${expirationKey} 已过期：${expiration}`] : [];
}

async function validateDocument(filePath) {
  const displayPath = relativePath(filePath);
  const source = await readFile(filePath, "utf8");
  const match = source.match(frontMatterPattern);
  if (!match) {
    return [`${displayPath}: 文件开头必须包含 YAML front matter`];
  }

  let metadata;
  try {
    metadata = parse(match[1]);
  } catch (error) {
    return [`${displayPath}: YAML 解析失败：${error.message}`];
  }

  if (!metadata || typeof metadata !== "object" || Array.isArray(metadata)) {
    return [`${displayPath}: front matter 必须解析为 YAML 对象`];
  }

  const issues = [];
  if (typeof metadata.author !== "string" || metadata.author.trim() === "") {
    issues.push("author 必须是非空字符串");
  }

  if (
    !Array.isArray(metadata.references) ||
    metadata.references.length === 0 ||
    metadata.references.some(
      (reference) => typeof reference !== "string" || reference.trim() === "",
    )
  ) {
    issues.push("references 必须是非空字符串数组");
  }

  if (!Object.hasOwn(metadata, expirationKey)) {
    issues.push(`缺少 ${expirationKey}`);
  } else {
    issues.push(...validateExpiration(metadata[expirationKey], filePath));
  }

  return issues.map((issue) => `${displayPath}: ${issue}`);
}

async function main() {
  const files = await discoverMarkdownFiles(tmpDir);
  const validationResults = await Promise.all(files.map((filePath) => validateDocument(filePath)));
  const issues = validationResults.flat();

  if (issues.length > 0) {
    console.error("docs/tmp 校验失败：");
    for (const issue of issues) {
      console.error(`- ${issue}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log(`docs/tmp 校验通过：${files.length} 个 Markdown 文件。`);
}

main().catch((error) => {
  console.error(`docs/tmp 校验无法执行：${error.message}`);
  process.exitCode = 1;
});
