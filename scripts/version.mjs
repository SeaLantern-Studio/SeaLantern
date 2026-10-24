import { readFile, writeFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const workspaceCargo = path.join(rootDir, "Cargo.toml");
const EXCLUDED_VERSION_MEMBERS = new Set(["crates/vendor/java-manager", "crates/vendor/sysproxy"]);

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

/** 从 Cargo.toml 的 [package] 段中提取 version 信息。
 *
 * 返回 `{ value, inherited }`：
 * - 字面量 `version = "1.2.3"` → `{ value: "1.2.3", inherited: false }`
 * - 继承 `version.workspace = true` → `{ value: null, inherited: true }`
 *   （真实版本在根 Cargo.toml 的 `[workspace.package]` 段，由调用方填入）
 * - 两者皆无 → `null`
 */
function extractCargoPackageVersion(content) {
  const section = content.match(/\[package\][^\[]*/)?.[0];
  if (!section) return null;

  // 先判继承：`version.workspace = true` 的写法不含字面量版本
  if (/^\s*version\.workspace\s*=\s*true/m.test(section)) {
    return { value: null, inherited: true };
  }

  const literal = section.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  return literal === undefined ? null : { value: literal, inherited: false };
}

/** 从根 Cargo.toml 的 [workspace.package] 段中提取 version 值。 */
function extractWorkspacePackageVersion(content) {
  const section = content.match(/\[workspace\.package\][^\[]*/)?.[0];
  if (!section) return null;

  return section.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? null;
}

/** 将根 Cargo.toml 的 [workspace.package] 段 version 字段替换为新版本。
 *
 * 只改 `version` 一行，段内其它字段（如 `edition`）保持原样。
 */
function replaceWorkspacePackageVersion(content, version) {
  const section = content.match(/\[workspace\.package\][^\[]*/)?.[0];
  if (!section) throw new Error("Cargo.toml 中未找到 [workspace.package] 段");

  if (!/^version\s*=\s*"[^"]+"/m.test(section)) {
    throw new Error("Cargo.toml 的 [workspace.package] 段中未找到 version 字段");
  }

  const newSection = section.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  return content.replace(section, () => newSection);
}

/** 将 Cargo.toml 的 [package] 段 version 字段替换为新版本。
 *
 * 继承版本（`version.workspace = true`）不在此处写入——真实值位于根
 * `[workspace.package]`，由 `replaceWorkspacePackageVersion` 负责。
 */
function replaceCargoPackageVersion(content, version) {
  const section = content.match(/\[package\][^\[]*/)?.[0];
  if (!section) throw new Error("Cargo.toml 中未找到 [package] 段");

  if (/^\s*version\.workspace\s*=\s*true/m.test(section)) {
    throw new Error(
      "Cargo.toml 的 [package] 段使用 workspace 继承版本，应在根 Cargo.toml 的 [workspace.package] 中修改",
    );
  }

  if (!/^version\s*=\s*"[^"]+"/m.test(section)) {
    throw new Error("Cargo.toml 的 [package] 段中未找到 version 字段");
  }

  const newSection = section.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  return content.replace(section, () => newSection);
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

/** 判断 workspace member 是否由外部 vendoring 管理，不参与项目版本同步。 */
function isExcludedVersionMember(member) {
  return EXCLUDED_VERSION_MEMBERS.has(member.replaceAll("\\", "/"));
}

/** 发现所有需要管理的文件路径。 */
async function discoverFiles() {
  const workspaceRaw = await readFile(workspaceCargo, "utf8");
  const members = parseWorkspaceMembers(workspaceRaw);

  // 先收集所有候选路径，再批量检查存在性
  const candidates = [];

  // 根 Cargo.toml 的 [workspace.package]：Rust 侧唯一版本真源
  candidates.push({
    label: "Cargo.toml [workspace.package]",
    path: workspaceCargo,
    type: "cargo-workspace",
  });

  // workspace member 的 Cargo.toml
  for (const member of members) {
    if (isExcludedVersionMember(member)) {
      continue;
    }

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

/** 从已读入的内存内容中解析单个文件的版本信息。
 *
 * 返回 `{ display, comparable }`：
 * - `display`：展示用字符串（继承成员带来源标注）
 * - `comparable`：参与一致性比较的版本值；无法解析时为 `null`
 *
 * 用结构化结果而非字符串后缀判断，避免「(未找到)（继承 workspace）」
 * 这类拼接串绕过缺失检查、把占位符当成真实版本参与比较。
 *
 * `inheritedVersion` 为根 `[workspace.package]` 的当前值：成员声明
 * `version.workspace = true` 时用它回填，并标注来源。
 */
function parseVersionFromRaw(file, raw, inheritedVersion) {
  if (file.type === "cargo") {
    const extracted = extractCargoPackageVersion(raw);
    if (!extracted) return { display: "(未找到)", comparable: null };
    if (extracted.inherited) {
      // 根版本读不到时，继承成员同样视为缺失——不能把占位符当版本比对。
      if (!inheritedVersion) {
        return { display: "(继承 workspace，但根版本不可读)", comparable: null };
      }
      return { display: `${inheritedVersion}（继承 workspace）`, comparable: inheritedVersion };
    }
    return { display: extracted.value, comparable: extracted.value };
  }
  if (file.type === "cargo-workspace") {
    const value = extractWorkspacePackageVersion(raw);
    return value === null
      ? { display: "(未找到)", comparable: null }
      : { display: value, comparable: value };
  }
  if (file.type === "json") {
    try {
      const value = JSON.parse(raw).version ?? null;
      return value === null
        ? { display: "(未找到)", comparable: null }
        : { display: value, comparable: value };
    } catch {
      return { display: "(解析失败)", comparable: null };
    }
  }
  return { display: "(未知类型)", comparable: null };
}

/** 读取所有文件的版本信息。
 *
 * 先把全部文件内容读进内存（单次快照），再从快照解析——避免「读 workspace
 * 与读成员之间」被其它写入插入，出现新旧混杂的比较结果。
 */
async function readVersions(files) {
  const raws = await Promise.all(files.map((f) => readFile(f.path, "utf8")));

  const workspaceIndex = files.findIndex((f) => f.type === "cargo-workspace");
  const inheritedVersion =
    workspaceIndex === -1 ? null : extractWorkspacePackageVersion(raws[workspaceIndex]);

  const entries = files.map((file, i) => [
    file.label,
    parseVersionFromRaw(file, raws[i], inheritedVersion),
  ]);
  return Object.fromEntries(entries);
}

/** 按统一格式输出版本信息并检查是否一致。
 *
 * 一致性只看 `comparable`：无法解析的条目（缺失 / 解析失败 / 根版本不可读）
 * 不参与比较，避免占位符被当成真实版本值。
 */
function printVersions(versions) {
  console.log("当前版本信息：\n");
  Object.entries(versions).forEach(([file, info]) => {
    console.log(`  ${file}: ${info.display}`);
  });

  const comparableValues = Object.values(versions)
    .map((info) => info.comparable)
    .filter((value) => value !== null);
  const unique = new Set(comparableValues);

  console.log("");
  const missing = Object.values(versions).filter((info) => info.comparable === null).length;
  if (unique.size === 1) {
    console.log("✅ 版本状态：所有已检测文件版本一致");
  } else if (unique.size === 0) {
    console.log("⚠️ 版本状态：没有可比较的版本值，请检查上述文件");
  } else {
    console.log("❌ 版本状态：检测到版本不一致，请检查上述文件");
  }
  if (missing > 0) {
    console.log(`   （${missing} 个条目无法解析，已排除在比较之外）`);
  }
}

// ---------------------------------------------------------------------------
// 写入
// ---------------------------------------------------------------------------

/** 生成单个文件的新内容。
 *
 * 返回 `null` 表示该文件无需写入（继承版本成员由根 `[workspace.package]`
 * 统一承载，不单独落盘）。
 */
function buildUpdatedContent(file, raw, version) {
  if (file.type === "cargo") {
    // 继承版本：跳过写入，但不是错误——真实值在 [workspace.package]。
    const section = raw.match(/\[package\][\s\S]*?(?=\n\[|$)/)?.[0] ?? "";
    if (/^\s*version\.workspace\s*=\s*true/m.test(section)) {
      return null;
    }
    return replaceCargoPackageVersion(raw, version);
  }
  if (file.type === "cargo-workspace") {
    return replaceWorkspacePackageVersion(raw, version);
  }
  if (file.type === "json") {
    const parsed = JSON.parse(raw);
    parsed.version = version;
    return `${JSON.stringify(parsed, null, 2)}\n`;
  }
  throw new Error(`未知文件类型：${file.type}`);
}

/** 将新版本写入所有核心文件。
 *
 * 两阶段：先全部生成并校验，再统一写入——避免中途某个文件失败留下
 * 「前面已改、后面没改」的半更新状态。
 */
async function updateVersion(files, version) {
  // 阶段一：并行读取并生成新内容（此阶段抛错则磁盘无任何改动）
  const contents = await Promise.all(files.map((f) => readFile(f.path, "utf8")));
  const planned = files.map((file, i) => ({
    file,
    content: buildUpdatedContent(file, contents[i], version),
  }));

  // 阶段二：写入需要变更的文件；继承版本成员内容为 null，跳过
  const written = [];
  for (const { file, content } of planned) {
    if (content === null) {
      continue;
    }
    await writeFile(file.path, content, "utf8");
    written.push(file.label);
  }

  console.log(`已更新 ${written.length} 个文件为 ${version}：`);
  for (const label of written) {
    console.log(`  - ${label}`);
  }

  const skipped = planned.filter((p) => p.content === null).map((p) => p.file.label);
  if (skipped.length > 0) {
    console.log(`\n跳过 ${skipped.length} 个继承版本文件（版本随 [workspace.package] 变化）：`);
    for (const label of skipped) {
      console.log(`  - ${label}`);
    }
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
