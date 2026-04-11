import { readFile, writeFile, access, unlink } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { exec } from "node:child_process";
import { promisify } from "node:util";
import { glob } from "glob";
import { homedir } from "node:os";

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");
const cargoDir = path.join(rootDir, "src-tauri");

const noticeFiles = {
  frontendLicenseJson: path.join(rootDir, "frontend-licenses.json"),
  backendLicenseJson: path.join(rootDir, "backend-licenses.json"),
  noticeFile: path.join(rootDir, "NOTICE"),
  packageJson: path.join(rootDir, "package.json"),
};

async function exists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function ensureCargoLicense() {
  try {
    // 检查 cargo-license 是否已安装
    await execAsync("cargo license --version", { cwd: cargoDir });
    return true;
  } catch {
    console.log("未找到 cargo-license，正在自动安装...");
    try {
      // 自动安装 cargo-license
      await execAsync("cargo install cargo-license", {
        cwd: cargoDir,
        stdio: "inherit", // 显示安装进度
      });
      console.log("cargo-license 安装完成！");
      return true;
    } catch (installError) {
      console.error("自动安装 cargo-license 失败，请手动执行：cargo install cargo-license");
      return false;
    }
  }
}

async function generateFrontedLicenseJson() {
  const { stdout } = await execAsync(
    "npx license-checker-rseidelsohn --start . --json --production",
    { cwd: rootDir },
  );
  await writeFile(noticeFiles.frontendLicenseJson, stdout, "utf8");
  return JSON.parse(stdout);
}

async function generateBackendLicenseJson() {
  const hasLicense = await ensureCargoLicense();
  if (!hasLicense) {
    return [];
  }
  const { stdout } = await execAsync("cargo license --json --avoid-dev-deps", { cwd: cargoDir });
  await writeFile(noticeFiles.backendLicenseJson, stdout, "utf8");
  return JSON.parse(stdout);
}

async function readLicenseFile(licensePath) {
  try {
    //排除README和spdx这些七七八八的内容
    if (!licensePath.includes("README") && !licensePath.includes(".spdx")) {
      const fullPath = path.join(rootDir, licensePath);
      if (await exists(fullPath)) {
        return await readFile(fullPath, "utf8");
      } else {
        if (await exists(licensePath)) {
          return await readFile(licensePath, "utf8");
        }
      }
    }
  } catch (e) {
    console.log(e);
    return null;
  }
  return null;
}

async function readBackendLicenseFile(crate) {
  if (!crate.license_file) return null;

  if (await exists(crate.license_file)) {
    return await readFile(crate.license_file, "utf8");
  }

  const cargoHome = process.env.CARGO_HOME || path.join(homedir(), ".cargo");
  const possiblePath = path.join(
    cargoHome,
    "registry/src",
    "index.crates.io-*",
    crate.name,
    crate.version,
    "LICENSE",
  );

  const matchedFiles = await glob(possiblePath);
  if (matchedFiles.length > 0) {
    return await readFile(matchedFiles[0], "utf8");
  }
  return null;
}

function formatFrontendDependency(name, version, info, id) {
  const lines = [];

  lines.push(`${id}. ${name}@${version}`);
  lines.push("");

  if (info.publisher) {
    lines.push(`Copyright (c) ${info.publisher}`);
  }

  if (info.repository) {
    lines.push(`Source: ${info.repository}`);
  }

  lines.push(`License: ${info.licenses}`);
  lines.push("");

  return lines.join("\n");
}

function formatBackendDependency(crate, id) {
  const lines = [];

  lines.push(`${id}. ${crate.name}@${crate.version}`);
  lines.push("");

  // 作者信息
  if (crate.authors) {
    let authors;
    if (typeof crate.authors === "string") {
      authors = [crate.authors];
    } else {
      authors = crate.authors;
    }
    if (authors.length > 0) {
      lines.push(`Authors: ${authors.join(", ")}`);
    }
  }

  // 仓库地址
  if (crate.repository) {
    lines.push(`Repository: ${crate.repository}`);
  }

  // 许可证
  lines.push(`License: ${crate.license || "Unknown"}`);
  lines.push("");

  return lines.join("\n");
}

async function getFrontendLicenseText(info) {
  if (info.licenseFile) {
    const licenseText = await readLicenseFile(info.licenseFile);
    if (licenseText) {
      return licenseText.replace(/^\uFEFF/, "").trim();
    }
  }
  return null;
}

async function generateNotice() {
  const packageJson = JSON.parse(await readFile(noticeFiles.packageJson, "utf8"));

  const frontendLicenses = await generateFrontedLicenseJson();
  const backendLicenses = await generateBackendLicenseJson();

  const noticeLines = [];

  noticeLines.push("Copyright (C) 2026 Sea Lantern Community");
  noticeLines.push("");
  noticeLines.push("This program is free software: you can redistribute it and/or modify");
  noticeLines.push("it under the terms of the GNU General Public License as published by");
  noticeLines.push("the Free Software Foundation, either version 3 of the License, or");
  noticeLines.push("(at your option) any later version.");
  noticeLines.push("");
  noticeLines.push("This program is distributed in the hope that it will be useful,");
  noticeLines.push("but WITHOUT ANY WARRANTY; without even the implied warranty of");
  noticeLines.push("MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the");
  noticeLines.push("GNU General Public License for more details.");
  noticeLines.push("");
  noticeLines.push("You should have received a copy of the GNU General Public License");
  noticeLines.push("along with this program. If not, see <https://www.gnu.org/licenses/>.");
  noticeLines.push("");
  noticeLines.push("---");
  noticeLines.push("");
  noticeLines.push("Third-party frontend dependencies:");
  noticeLines.push("");

  const sortedPackages = Object.entries(frontendLicenses).toSorted(([a], [b]) =>
    a.localeCompare(b),
  );
  let id = 1;

  const frontendSections = await Promise.all(
    sortedPackages.map(async ([pkgKey, info]) => {
      if (pkgKey.startsWith(packageJson.name)) {
        return null;
      }

      const atIndex = pkgKey.lastIndexOf("@");
      const name = pkgKey.slice(0, atIndex);
      const version = pkgKey.slice(atIndex + 1);
      const licenseText = await getFrontendLicenseText(info);

      return {
        name,
        version,
        info,
        licenseText,
      };
    }),
  );

  for (const section of frontendSections) {
    if (!section) {
      continue;
    }

    noticeLines.push(formatFrontendDependency(section.name, section.version, section.info, id));

    if (section.licenseText) {
      noticeLines.push(section.licenseText);
      noticeLines.push("");
    }
    id += 1;
  }

  noticeLines.push("");
  noticeLines.push("---");
  noticeLines.push("");
  noticeLines.push("Third-party backend dependencies:");
  noticeLines.push("");

  const sortedBackend = [...backendLicenses]
    .filter((crate) => crate.name !== "sea-lantern")
    .toSorted((a, b) => `${a.name}@${a.version}`.localeCompare(`${b.name}@${b.version}`));

  id = 1;
  const backendSections = await Promise.all(
    sortedBackend.map(async (crate) => ({
      crate,
      licenseText: await readBackendLicenseFile(crate),
    })),
  );

  for (const section of backendSections) {
    noticeLines.push(formatBackendDependency(section.crate, id));

    if (section.licenseText) {
      noticeLines.push(section.licenseText);
      noticeLines.push("");
    }
    id += 1;
  }

  await writeFile(noticeFiles.noticeFile, noticeLines.join("\n"), "utf8");

  if (await exists(noticeFiles.frontendLicenseJson)) {
    await unlink(noticeFiles.frontendLicenseJson);
  }

  if (await exists(noticeFiles.backendLicenseJson)) {
    await unlink(noticeFiles.backendLicenseJson);
  }

  console.log("已生成NOTICE：");
  console.log(noticeFiles.noticeFile);
}

function printUsage() {
  console.log("用法：");
  console.log("  pnpm notice");
}

async function main() {
  const [command] = process.argv.slice(2);

  if (command === "--help" || command === "-h") {
    printUsage();
    return;
  }

  if (command) {
    printUsage();
    throw new Error(`未知命令：${command}`);
  }

  await generateNotice();
}

main().catch((error) => {
  console.error(`\n错误：${error.message}`);
  process.exit(1);
});
