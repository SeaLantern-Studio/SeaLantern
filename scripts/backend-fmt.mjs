/**
 * lint-staged 专用后端格式化入口。
 *
 * lint-staged 会把暂存的 .rs 文件列表作为参数传入，但 `cargo fmt` 不接受
 * 文件路径参数（它按 crate/workspace 粒度格式化），直接透传会报错。
 * 本脚本忽略传入的文件列表，在 workspace 根统一执行 `cargo fmt`，
 * 保证与手跑 `cargo fmt` 的行为一致。
 */

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

// lint-staged 传入的文件列表仅作诊断提示，不参与 cargo fmt 参数。
const stagedFiles = process.argv.slice(2);
if (stagedFiles.length > 0) {
  console.log(`[backend-fmt] 暂存 .rs 文件 ${stagedFiles.length} 个，统一格式化 workspace…`);
}

// 参数固定为空数组（cargo fmt 不接受文件参数），无需 shell 拼接，
// 避免 DEP0190 弃用警告与注入风险。
const result = spawnSync("cargo", ["fmt"], {
  cwd: rootDir,
  stdio: "inherit",
});

if (result.error) {
  console.error(`[backend-fmt] 无法执行 cargo：${result.error.message}`);
  process.exit(1);
}

if (result.status !== 0) {
  console.error(`[backend-fmt] cargo fmt 退出码 ${result.status}`);
  process.exit(result.status ?? 1);
}

console.log("[backend-fmt] cargo fmt 完成");
