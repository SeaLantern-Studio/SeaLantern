import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const rawArgs = process.argv.slice(2);
const [mode, ...restArgs] = rawArgs;
const viteBinPath = fileURLToPath(new URL("../node_modules/vite/bin/vite.js", import.meta.url));
const viteArgs = [viteBinPath];

if (mode === "next:auth") {
  viteArgs.push("--open", "/auth?preview=1", ...restArgs);
} else if (mode === "next:home") {
  viteArgs.push("--open", "/?preview=1", ...restArgs);
} else {
  viteArgs.push(...rawArgs);
}

const child = spawn(process.execPath, viteArgs, {
  stdio: "inherit",
  shell: false,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 0);
});
