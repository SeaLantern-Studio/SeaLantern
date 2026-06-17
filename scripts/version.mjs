import { readFile, writeFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

const files = {
  packageJson: path.join(rootDir, "frontend", "package.json"),
  cargoToml: path.join(rootDir, "backend", "tauri-host", "Cargo.toml"),
  tauriConf: path.join(rootDir, "backend", "tauri-host", "tauri.conf.json"),
  pkgbuild: path.join(rootDir, "PKGBUILD"),
  srcinfo: path.join(rootDir, ".SRCINFO"),
};

// 校验输入版本号是否符合支持的项目版本格式。
function isValidVersion(version) {
  return isValidSemver(version) || isValidNightlyBuildVersion(version);
}

function isValidSemver(version) {
  return /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version);
}

function isValidNightlyBuildVersion(version) {
  return /^NightlyBuild-[0-9A-Fa-f]{7,40}-\d{8}T\d{6}Z$/.test(version);
}

// 判断指定路径的文件是否存在且可访问。
async function exists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

// 从 Cargo.toml 的 [package] 段中提取 version 值。
function extractCargoPackageVersion(content) {
  const packageSectionMatch = content.match(/\[package\][\s\S]*?(?=\n\[|$)/);
  if (!packageSectionMatch) return null;

  const section = packageSectionMatch[0];
  const versionMatch = section.match(/^version\s*=\s*"([^"]+)"/m);
  return versionMatch?.[1] ?? null;
}

// 将 Cargo.toml 的 [package] 段 version 字段替换为新版本。
function replaceCargoPackageVersion(content, version) {
  const packageSectionMatch = content.match(/\[package\][\s\S]*?(?=\n\[|$)/);
  if (!packageSectionMatch) {
    throw new Error("Cargo.toml 中未找到 [package] 段");
  }

  const section = packageSectionMatch[0];
  if (!/^version\s*=\s*"[^"]+"/m.test(section)) {
    throw new Error("Cargo.toml 的 [package] 段中未找到 version 字段");
  }

  const newSection = section.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);

  return content.replace(section, newSection);
}

// 读取并汇总核心及可选文件中的版本号信息。
async function readVersions() {
  const packageJsonRaw = await readFile(files.packageJson, "utf8");
  const packageJson = JSON.parse(packageJsonRaw);

  const cargoTomlRaw = await readFile(files.cargoToml, "utf8");
  const cargoVersion = extractCargoPackageVersion(cargoTomlRaw);

  const tauriConfRaw = await readFile(files.tauriConf, "utf8");
  const tauriConf = JSON.parse(tauriConfRaw);

  const versions = {
    "frontend/package.json": packageJson.version ?? "(未找到)",
    "backend/tauri-host/Cargo.toml": cargoVersion ?? "(未找到)",
    "backend/tauri-host/tauri.conf.json": tauriConf.version ?? "(未找到)",
  };

  if (await exists(files.pkgbuild)) {
    const pkgbuildRaw = await readFile(files.pkgbuild, "utf8");
    const pkgver = pkgbuildRaw.match(/^pkgver\s*=\s*([^\s#]+)/m)?.[1] ?? "(未找到)";
    versions.PKGBUILD = pkgver;
  }

  if (await exists(files.srcinfo)) {
    const srcinfoRaw = await readFile(files.srcinfo, "utf8");
    const pkgver = srcinfoRaw.match(/^\s*pkgver\s*=\s*(.+)$/m)?.[1]?.trim() ?? "(未找到)";
    versions[".SRCINFO"] = pkgver;
  }

  return versions;
}

// 按统一格式输出版本信息并检查是否一致。
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

// 将新版本写入核心文件，并在存在时同步更新 AUR 相关文件。
async function updateVersion(version) {
  const packageJsonRaw = await readFile(files.packageJson, "utf8");
  const packageJson = JSON.parse(packageJsonRaw);
  packageJson.version = version;
  await writeFile(files.packageJson, `${JSON.stringify(packageJson, null, 2)}\n`, "utf8");

  const cargoTomlRaw = await readFile(files.cargoToml, "utf8");
  const cargoTomlUpdated = replaceCargoPackageVersion(cargoTomlRaw, version);
  await writeFile(files.cargoToml, cargoTomlUpdated, "utf8");

  const tauriConfRaw = await readFile(files.tauriConf, "utf8");
  const tauriConf = JSON.parse(tauriConfRaw);
  tauriConf.version = version;
  await writeFile(files.tauriConf, `${JSON.stringify(tauriConf, null, 2)}\n`, "utf8");

  const optionalUpdated = [];

  if (await exists(files.pkgbuild)) {
    const pkgbuildRaw = await readFile(files.pkgbuild, "utf8");
    const pkgbuildUpdated = pkgbuildRaw.replace(/^pkgver\s*=\s*([^\s#]+)/m, `pkgver=${version}`);
    await writeFile(files.pkgbuild, pkgbuildUpdated, "utf8");
    optionalUpdated.push("PKGBUILD");
  }

  if (await exists(files.srcinfo)) {
    const srcinfoRaw = await readFile(files.srcinfo, "utf8");
    let srcinfoUpdated = srcinfoRaw
      .replace(/^\s*pkgver\s*=\s*.+$/m, `\tpkgver = ${version}`)
      .replace(
        /^\s*source\s*=\s*.*Sea\.Lantern_.*\.deb$/m,
        `\tsource = sealantern-${version}-amd64.deb::https://github.com/SeaLantern-Studio/SeaLantern/releases/download/v${version}/Sea.Lantern_${version}_amd64.deb`,
      );

    await writeFile(files.srcinfo, srcinfoUpdated, "utf8");
    optionalUpdated.push(".SRCINFO");
  }

  console.log(`已更新核心版本文件为 ${version}：`);
  console.log("- frontend/package.json");
  console.log("- backend/tauri-host/Cargo.toml");
  console.log("- backend/tauri-host/tauri.conf.json");
  if (optionalUpdated.length > 0) {
    console.log(`另外已同步更新：${optionalUpdated.join(", ")}`);
  }
}

// 输出脚本命令帮助信息。
function printUsage() {
  console.log("用法：");
  console.log("  pnpm sv");
  console.log("  pnpm cv <version>");
}

// 解析命令参数并分发到查看或修改版本流程。
async function main() {
  const [command, value] = process.argv.slice(2);

  if (!command || command === "show") {
    const versions = await readVersions();
    printVersions(versions);
    return;
  }

  if (command === "change") {
    if (!value) {
      throw new Error("请提供新版本号，例如：pnpm cv 1.2.3");
    }

    if (!isValidVersion(value)) {
      throw new Error(
        `无效版本号：${value}，请使用语义化版本（如 1.2.3）或 NightlyBuild-{SHORT_SHA}-{DateTime}（如 NightlyBuild-abcdef0-20260617T180000Z）`,
      );
    }

    if (!isValidSemver(value)) {
      throw new Error(
        `当前 change 命令只能写入语义化版本到项目清单文件；NightlyBuild 版本请通过构建环境注入显示版本，例如 SEA_LANTERN_BUILD_VERSION=${value}`,
      );
    }

    await updateVersion(value);
    const versions = await readVersions();
    console.log("");
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
