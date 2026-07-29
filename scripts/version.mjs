import { readFile, writeFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const workspaceCargo = path.join(rootDir, "Cargo.toml");

// ---------------------------------------------------------------------------
// 工具函数
// ---------------------------------------------------------------------------

/** 校验输入版本号是否符合语义化版本格式。 */
function isValidVersion(version) {
  return /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version);
}

/** 判断指定路径的文件是否存在且可访问。 */
async function exists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

/** 从 Cargo.toml 的 [package] 段中提取 version 值。 */
function extractCargoPackageVersion(content) {
  const section = content.match(/\[package\][\s\S]*?(?=\n\[|$)/)?.[0];
  if (!section) return null;

  return section.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? null;
}

/** 将 Cargo.toml 的 [package] 段 version 字段替换为新版本。 */
function replaceCargoPackageVersion(content, version) {
  const section = content.match(/\[package\][\s\S]*?(?=\n\[|$)/)?.[0];
  if (!section) throw new Error("Cargo.toml 中未找到 [package] 段");

  if (!/^version\s*=\s*"[^"]+"/m.test(section)) {
    throw new Error("Cargo.toml 的 [package] 段中未找到 version 字段");
  }

  const newSection = section.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  return content.replace(section, newSection);
}

// ---------------------------------------------------------------------------
// 文件发现
// ---------------------------------------------------------------------------

/** 从 workspace Cargo.toml 中解析 [workspace] members 列表。 */
function parseWorkspaceMembers(content) {
  const wsSection = content.match(/\[workspace\][\s\S]*?(?=\n\[|$)/)?.[0];
  if (!wsSection) return [];

  const membersLine = wsSection.match(/^members\s*=\s*\[([^\]]+)\]/m)?.[1];
  if (!membersLine) return [];

  return membersLine
    .split(",")
    .map((m) => m.trim().replace(/["\s]/g, ""))
    .filter(Boolean);
}

/** 发现所有需要管理的文件路径。 */
async function discoverFiles() {
  const workspaceRaw = await readFile(workspaceCargo, "utf8");
  const members = parseWorkspaceMembers(workspaceRaw);

  // 先收集所有候选路径，再批量检查存在性
  const candidates = [];

  // workspace member 的 Cargo.toml
  for (const member of members) {
    candidates.push({
      label: `${member}/Cargo.toml`,
      path: path.join(rootDir, member, "Cargo.toml"),
      type: "cargo",
    });
  }

  // package.json
  candidates.push({
    label: "package.json",
    path: path.join(rootDir, "package.json"),
    type: "json",
  });

  // tauri.conf.json
  candidates.push({
    label: "src-tauri/tauri.conf.json",
    path: path.join(rootDir, "src-tauri", "tauri.conf.json"),
    type: "json",
  });

  // 批量检查存在性
  const results = await Promise.all(candidates.map((c) => exists(c.path)));

  return candidates.filter((_, i) => results[i]);
}

// ---------------------------------------------------------------------------
// 读取与输出
// ---------------------------------------------------------------------------

/** 提取单个文件的版本号。 */
async function readVersion(file) {
  const raw = await readFile(file.path, "utf8");

  if (file.type === "cargo") {
    return extractCargoPackageVersion(raw) ?? "(未找到)";
  }
  if (file.type === "json") {
    try {
      return JSON.parse(raw).version ?? "(未找到)";
    } catch {
      return "(解析失败)";
    }
  }
  return "(未知类型)";
}

/** 读取所有文件的版本号。 */
async function readVersions(files) {
  const entries = await Promise.all(
    files.map(async (file) => [file.label, await readVersion(file)]),
  );
  return Object.fromEntries(entries);
}

/** 按统一格式输出版本信息并检查是否一致。 */
function printVersions(versions) {
  console.log("当前版本信息：\n");
  Object.entries(versions).forEach(([file, version]) => {
    console.log(`  ${file}: ${version}`);
  });

  const validValues = Object.values(versions).filter((v) => v !== "(未找到)" && v !== "(解析失败)");
  const unique = new Set(validValues);
  console.log("");
  if (unique.size <= 1) {
    console.log("✅ 版本状态：所有已检测文件版本一致");
  } else {
    console.log("❌ 版本状态：检测到版本不一致，请检查上述文件");
  }
}

// ---------------------------------------------------------------------------
// 写入
// ---------------------------------------------------------------------------

/** 生成单个文件的新内容。 */
function buildUpdatedContent(file, raw, version) {
  if (file.type === "cargo") {
    return replaceCargoPackageVersion(raw, version);
  }
  if (file.type === "json") {
    const parsed = JSON.parse(raw);
    parsed.version = version;
    return `${JSON.stringify(parsed, null, 2)}\n`;
  }
  throw new Error(`未知文件类型：${file.type}`);
}

/** 将新版本写入所有核心文件。 */
async function updateVersion(files, version) {
  // 先并行读取所有文件
  const contents = await Promise.all(files.map((f) => readFile(f.path, "utf8")));

  // 并行生成新内容并写入
  const writes = files.map(async (file, i) => {
    const newContent = buildUpdatedContent(file, contents[i], version);
    await writeFile(file.path, newContent, "utf8");
    return file.label;
  });

  const updated = await Promise.all(writes);

  console.log(`已更新 ${updated.length} 个文件为 ${version}：`);
  for (const label of updated) {
    console.log(`  - ${label}`);
  }
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function printUsage() {
  console.log("用法：");
  console.log("  pnpm sv                     查看当前所有版本号");
  console.log("  pnpm cv <version>           统一修改所有版本号");
}

async function main() {
  const [command, value] = process.argv.slice(2);

  const files = await discoverFiles();

  if (!command || command === "show") {
    const versions = await readVersions(files);
    printVersions(versions);
    return;
  }

  if (command === "change") {
    if (!value) {
      throw new Error("请提供新版本号，例如：pnpm cv 1.2.3");
    }
    if (!isValidVersion(value)) {
      throw new Error(`无效版本号：${value}，请使用语义化版本，例如 1.2.3`);
    }

    await updateVersion(files, value);
    console.log("");
    const versions = await readVersions(files);
    printVersions(versions);
    return;
  }

  printUsage();
  throw new Error(`未知命令：${command}`);
}

main().catch((error) => {
  console.error(`\n错误：${error.message}`);
  process.exit(1);
});
