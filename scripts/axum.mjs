//! sealantern-server 构建/运行脚本。
//!
//! 用法：
//!   npm run axum dev     # 开发模式：先启动 vite dev server，再以 cargo-watch 运行 server
//!                        #（监听 Rust 源码变化自动重编译重启，近似 tauri dev）
//!   npm run axum build   # 生产构建：先 pnpm build 生成前端产物，再 cargo release 构建
//!
//! 无参数或参数不合法时打印用法并退出非零码。

import { spawn, spawnSync } from "node:child_process";

const isWindows = process.platform === "win32";

const USAGE = `用法:
  npm run axum dev     # 开发模式：先启动 vite dev server，再前台运行 sealantern-server
  npm run axum build   # 生产构建：pnpm build 生成前端产物，再 cargo release 构建`;

/** 同步执行前台子进程（继承 stdio），失败时以子进程退出码结束脚本。 */
function run(command, args, options = {}) {
  const fullCommand = `${command} ${args.join(" ")}`;
  console.log(`\n> ${fullCommand}`);
  // Windows 上 pnpm/npm 是 .cmd shim，必须经 shell 启动（shell: true 走 cmd / sh）。
  // 命令与参数均为脚本内常量（无用户输入），字符串拼接安全；
  // 不传 args 数组可避免 Node 的 DEP0190 告警（shell 模式下参数拼接安全提示）。
  const result = spawnSync(fullCommand, { stdio: "inherit", shell: true, ...options });

  if (result.error) {
    console.error(`[axum] 无法执行 ${command}: ${result.error.message}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    console.error(`[axum] ${command} 以退出码 ${result.status} 结束`);
    process.exit(result.status);
  }
}

/** 后台启动子进程（继承 stdio），返回进程句柄；不阻塞当前脚本。 */
function spawnBackground(command, options = {}) {
  console.log(`\n> ${command}`);
  return spawn(command, { stdio: "inherit", shell: true, ...options });
}

/** 探测 vite dev server 是否已就绪（HTTP 可响应）。 */
async function viteReady(host, port, timeoutMs) {
  const url = `http://${host}:${port}/`;
  const deadline = Date.now() + timeoutMs;
  // 轮询必须串行等待，无法用 Promise.all 并行，禁用 no-await-in-loop 告警。
  // eslint-disable-next-line no-await-in-loop
  while (Date.now() < deadline) {
    try {
      // eslint-disable-next-line no-await-in-loop
      const response = await fetch(url);
      if (response.ok) return true;
    } catch {
      // 未就绪，继续轮询。
    }
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => setTimeout(resolve, 300));
  }
  return false;
}

/** 结束后台进程；Windows 下用 taskkill 连进程树一起杀（pnpm 会派生 node 子进程）。 */
function killProcess(proc) {
  if (!proc || proc.killed) return;
  if (isWindows) {
    // taskkill 是系统 exe，无需 shell；参数数组 + 无 shell 不触发 DEP0190。
    spawnSync("taskkill", ["/pid", String(proc.pid), "/t", "/f"], {
      stdio: "ignore",
    });
  } else {
    proc.kill("SIGTERM");
  }
}

/** 探测命令是否可用（无输出，仅返回是否找到）。 */
function commandAvailable(command) {
  // 命令含子命令（如 "cargo watch"）需经 shell；字符串命令无 args 数组，不触发 DEP0190。
  const result = spawnSync(`${command} --version`, {
    stdio: "ignore",
    shell: true,
  });
  return result.status === 0;
}

/**
 * 确保 cargo-watch 已安装。
 *
 * cargo watch 是实现 Rust 代码热重载的标准工具；未安装时自动执行
 * `cargo install cargo-watch`（需编译，首次可能耗时数分钟）。
 */
function ensureCargoWatch() {
  if (commandAvailable("cargo watch")) {
    console.log("[axum] cargo-watch 已安装");
    return;
  }
  console.log("[axum] 未检测到 cargo-watch，正在安装（首次编译可能需要数分钟）...");
  run("cargo", ["install", "cargo-watch"]);
  if (!commandAvailable("cargo watch")) {
    console.error("[axum] cargo-watch 安装后仍不可用，请手动执行 cargo install cargo-watch");
    process.exit(1);
  }
}

/**
 * 解析命令行参数，执行对应的构建/运行流程。
 *
 * @param {string[]} args 命令行参数（不含 node 与脚本路径）
 */
async function main(args) {
  const mode = args[0];
  const viteHost = process.env.VITE_DEV_HOST || "127.0.0.1";
  const vitePort = Number(process.env.VITE_PORT || 5173);
  const viteUrl = `http://${viteHost}:${vitePort}/`;

  switch (mode) {
    case "dev": {
      let vite = null;
      let watcher = null;
      if (await viteReady(viteHost, vitePort, 800)) {
        console.log(`[axum] 检测到已有 vite dev server（${viteUrl}），直接复用`);
      } else {
        vite = spawnBackground("pnpm dev");
        const ready = await viteReady(viteHost, vitePort, 15000);
        if (!ready) {
          console.error(`[axum] vite dev server 未在 ${viteUrl} 就绪`);
          killProcess(vite);
          process.exit(1);
        }
      }

      ensureCargoWatch();
      // cargo watch 监听 Rust 源码变化，自动重编译并重启 server（近似 tauri dev 体验）。
      // cargo-watch 是真实可执行文件，直接参数数组 spawn、不经 shell：
      // 避免 cmd 对 -x 内部双引号的解析问题，同时不触发 DEP0190 告警。
      const watchCommand = isWindows ? "cargo-watch.exe" : "cargo-watch";
      watcher = spawn(watchCommand, ["-x", "run -p sealantern-server"], {
        stdio: "inherit",
        env: { ...process.env, VITE_AUTO_START: "false" },
      });
      watcher.on("exit", () => {
        console.log("[axum] cargo watch 已退出");
      });

      // 等待进程结束：cargo watch 常驻，随 Ctrl+C 终止。
      await new Promise((resolve) => {
        const stop = () => resolve();
        process.on("SIGINT", stop);
        process.on("SIGTERM", stop);
        watcher.on("exit", stop);
      });

      if (vite) {
        killProcess(vite);
      }
      break;
    }

    case "build":
      run("pnpm", ["build"]);
      run("cargo", ["build", "-p", "sealantern-server", "--release"]);
      break;

    default:
      console.error(`[axum] 未知参数: ${mode ?? "(无)"}`);
      console.error(USAGE);
      process.exit(2);
  }
}

main(process.argv.slice(2)).catch((error) => {
  console.error(`[axum] 脚本执行失败: ${error}`);
  process.exit(1);
});
