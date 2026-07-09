import { readFile, writeFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const currentFile = fileURLToPath(import.meta.url);
const currentDir = path.dirname(currentFile);
const rootDir = path.resolve(currentDir, "..");

const files = {
  workspaceCargoToml: path.join(rootDir, "Cargo.toml"),
  packageJson: path.join(rootDir, "frontend", "package.json"),
  tauriConf: path.join(rootDir, "backend", "tauri-host", "tauri.conf.json"),
  pkgbuild: path.join(rootDir, "packaging", "PKGBUILD"),
  srcinfo: path.join(rootDir, "packaging", ".SRCINFO"),
};

function isValidVersion(version) {
  return isValidSemver(version) || isValidNightlyBuildVersion(version);
}

function isValidSemver(version) {
  return /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version);
}

function isValidNightlyBuildVersion(version) {
  return /^NightlyBuild-[0-9A-Fa-f]{7,40}-\d{8}T\d{6}Z$/.test(version);
}

async function exists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

function extractWorkspacePackageVersion(content) {
  const packageSectionMatch = content.match(/\[workspace\.package\][\s\S]*?(?=\n\[|$)/);
  if (!packageSectionMatch) return null;

  const section = packageSectionMatch[0];
  const versionMatch = section.match(/^version\s*=\s*"([^"]+)"/m);
  return versionMatch?.[1] ?? null;
}

function replaceWorkspacePackageVersion(content, version) {
  const packageSectionMatch = content.match(/\[workspace\.package\][\s\S]*?(?=\n\[|$)/);
  if (!packageSectionMatch) {
    throw new Error("Cargo.toml 中未找到 [workspace.package] 段");
  }

  const section = packageSectionMatch[0];
  if (!/^version\s*=\s*"[^"]+"/m.test(section)) {
    throw new Error("Cargo.toml 的 [workspace.package] 段中未找到 version 字段");
  }

  const newSection = section.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);

  return content.replace(section, newSection);
}

async function readVersions() {
  const packageJsonRaw = await readFile(files.packageJson, "utf8");
  const packageJson = JSON.parse(packageJsonRaw);

  const workspaceCargoTomlRaw = await readFile(files.workspaceCargoToml, "utf8");
  const cargoVersion = extractWorkspacePackageVersion(workspaceCargoTomlRaw);

  const tauriConfRaw = await readFile(files.tauriConf, "utf8");
  const tauriConf = JSON.parse(tauriConfRaw);

  const versions = {
    "frontend/package.json": packageJson.version ?? "(未找到)",
    "Cargo.toml [workspace.package]": cargoVersion ?? "(未找到)",
    "backend/tauri-host/tauri.conf.json": tauriConf.version ?? "(未找到)",
  };

  if (await exists(files.pkgbuild)) {
    const pkgbuildRaw = await readFile(files.pkgbuild, "utf8");
    const pkgver = pkgbuildRaw.match(/^pkgver\s*=\s*([^\s#]+)/m)?.[1] ?? "(未找到)";
    versions["packaging/PKGBUILD"] = pkgver;
  }

  if (await exists(files.srcinfo)) {
    const srcinfoRaw = await readFile(files.srcinfo, "utf8");
    const pkgver = srcinfoRaw.match(/^\s*pkgver\s*=\s*(.+)$/m)?.[1]?.trim() ?? "(未找到)";
    versions["packaging/.SRCINFO"] = pkgver;
  }

  return versions;
}

function printVersions(versions) {
  console.log("当前版本信息：\n");
  Object.entries(versions).forEach(([file, version]) => {
    console.log(`${file}: ${version}`);
  });

  const validValues = Object.values(versions).filter((v) => v !== "(未找到)");
  const unique = new Set(validValues);
  console.log("");
  if (unique.size <= 1) {
    console.log("版本状态：所有已检测文件版本一致");
  } else {
    console.log("版本状态：检测到版本不一致，请检查上述文件");
  }
}

async function updateVersion(version) {
  const packageJsonRaw = await readFile(files.packageJson, "utf8");
  const packageJson = JSON.parse(packageJsonRaw);
  packageJson.version = version;
  await writeFile(files.packageJson, `${JSON.stringify(packageJson, null, 2)}\n`, "utf8");

  const workspaceCargoTomlRaw = await readFile(files.workspaceCargoToml, "utf8");
  const workspaceCargoTomlUpdated = replaceWorkspacePackageVersion(workspaceCargoTomlRaw, version);
  await writeFile(files.workspaceCargoToml, workspaceCargoTomlUpdated, "utf8");

  const tauriConfRaw = await readFile(files.tauriConf, "utf8");
  const tauriConf = JSON.parse(tauriConfRaw);
  tauriConf.version = version;
  await writeFile(files.tauriConf, `${JSON.stringify(tauriConf, null, 2)}\n`, "utf8");

  const optionalUpdated = [];

  if (await exists(files.pkgbuild)) {
    const pkgbuildRaw = await readFile(files.pkgbuild, "utf8");
    const pkgbuildUpdated = pkgbuildRaw
      .replace(/^pkgver\s*=\s*([^\s#]+)/m, `pkgver=${version}`)
      .replace(/^pkgrel\s*=\s*([^\s#]+)/m, "pkgrel=1");
    await writeFile(files.pkgbuild, pkgbuildUpdated, "utf8");
    optionalUpdated.push("packaging/PKGBUILD");
  }

  if (await exists(files.srcinfo)) {
    const srcinfoRaw = await readFile(files.srcinfo, "utf8");
    const srcinfoUpdated = srcinfoRaw
      .replace(/^\s*pkgver\s*=\s*.+$/m, `\tpkgver = ${version}`)
      .replace(/^\s*pkgrel\s*=\s*.+$/m, "\tpkgrel = 1")
      .replace(
        /^\s*source\s*=\s*.*Sea\.Lantern_.*\.deb$/m,
        `\tsource = sealantern-${version}-amd64.deb::https://github.com/SeaLantern-Studio/SeaLantern/releases/download/v${version}/Sea.Lantern_${version}_amd64.deb`,
      );

    await writeFile(files.srcinfo, srcinfoUpdated, "utf8");
    optionalUpdated.push("packaging/.SRCINFO");
  }

  console.log(`已更新核心版本文件为 ${version}：`);
  console.log("- frontend/package.json");
  console.log("- Cargo.toml [workspace.package]");
  console.log("- backend/tauri-host/tauri.conf.json");
  if (optionalUpdated.length > 0) {
    console.log(`另外已同步更新：${optionalUpdated.join(", ")}`);
  }
}

async function runShow(args) {
  assertNoExtraArgs(args, "show");
  const versions = await readVersions();
  printVersions(versions);
}

async function runChange(args) {
  const [version, ...rest] = args;
  if (!version) {
    throw new Error("请提供新版本号，例如：pnpm cv 1.2.3");
  }
  if (rest.length > 0) {
    throw new Error(`change 命令只接受一个版本号，多余参数：${rest.join(" ")}`);
  }

  validateChangeVersion(version);

  await updateVersion(version);
  const versions = await readVersions();
  console.log("");
  printVersions(versions);
}

function validateChangeVersion(version) {
  if (!isValidVersion(version)) {
    throw new Error(
      `无效版本号：${version}，请使用语义化版本（如 1.2.3）或 NightlyBuild-{SHORT_SHA}-{DateTime}（如 NightlyBuild-abcdef0-20260617T180000Z）`,
    );
  }

  if (!isValidSemver(version)) {
    throw new Error(
      `当前 change 命令只能写入语义化版本到项目清单文件；NightlyBuild 版本请通过构建环境注入显示版本，例如 SEA_LANTERN_BUILD_VERSION=${version}`,
    );
  }
}

function assertNoExtraArgs(args, commandName) {
  if (args.length > 0) {
    throw new Error(`${commandName} 命令不接受参数：${args.join(" ")}`);
  }
}

const commands = new Map([
  [
    "show",
    {
      usage: "show",
      summary: "显示所有版本文件的当前版本",
      run: runShow,
    },
  ],
  [
    "change",
    {
      usage: "change <version>",
      summary: "同步更新项目版本和 AUR packaging 版本",
      run: runChange,
    },
  ],
]);

const helpCommands = new Set(["help", "-h", "--help"]);

function printUsage() {
  console.log("用法：");
  for (const command of commands.values()) {
    console.log(`  node scripts/version.mjs ${command.usage}`);
  }
  console.log("  node scripts/version.mjs help");
  console.log("");
  console.log("命令：");
  for (const [name, command] of commands) {
    console.log(`  ${name.padEnd(8)} ${command.summary}`);
  }
  console.log("");
  console.log("pnpm 快捷命令：");
  console.log("  pnpm sv");
  console.log("  pnpm cv <version>");
}

async function main() {
  const [commandName = "show", ...args] = process.argv.slice(2);

  if (helpCommands.has(commandName)) {
    assertNoExtraArgs(args, commandName);
    printUsage();
    return;
  }

  const command = commands.get(commandName);
  if (!command) {
    printUsage();
    throw new Error(`未知命令：${commandName}`);
  }

  await command.run(args);
}

main().catch((error) => {
  console.error(`\n错误：${error.message}`);
  process.exit(1);
});
