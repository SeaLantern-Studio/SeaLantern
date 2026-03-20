import { readFile, writeFile, access, unlink } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { exec } from "node:child_process";
import { promisify } from "node:util";

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

const files = {
  licenseJson: path.join(rootDir, "licenses.json"),
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

async function generateLicenseJson() {
  const { stdout } = await execAsync(
    "npx license-checker-rseidelsohn --start . --json --production",
    { cwd: rootDir },
  );
  await writeFile(files.licenseJson, stdout, "utf8");
  return JSON.parse(stdout);
}

async function readLicenseFile(licensePath) {
  try {
    //排除md和spdx这些七七八八的内容
    if (!licensePath.includes(".md") && !licensePath.includes(".spdx")) {
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

function formatDependency(name, version, info, id) {
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

async function getLicenseText(info) {
  if (info.licenseFile) {
    const licenseText = await readLicenseFile(info.licenseFile);
    if (licenseText) {
      return licenseText.replace(/^\uFEFF/, "").trim();
    }
  }
  return null;
}

async function generateNotice() {
  const packageJson = JSON.parse(await readFile(files.packageJson, "utf8"));

  const licenses = await generateLicenseJson();

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
  noticeLines.push("Third-party dependencies:");
  noticeLines.push("");

  const sortedPackages = Object.entries(licenses).sort(([a], [b]) => a.localeCompare(b));
  let id = 1;

  for (const [pkgKey, info] of sortedPackages) {
    if (pkgKey.startsWith(packageJson.name)) continue;

    const [name, version] = pkgKey.split("@");

    noticeLines.push(formatDependency(name, version, info, id));

    const licenseText = await getLicenseText(info);
    if (licenseText) {
      noticeLines.push(licenseText);
      noticeLines.push("");
    }
    id += 1;
  }

  await writeFile(files.noticeFile, noticeLines.join("\n"), "utf8");

  if (await exists(files.licenseJson)) {
    await unlink(files.licenseJson);
  }

  console.log("已生成NOTICE：");
  console.log(files.noticeFile);
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
