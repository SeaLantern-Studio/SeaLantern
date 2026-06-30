import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const rawArgs = process.argv.slice(2);
const [mode, ...restArgs] = rawArgs;
const viteBinPath = fileURLToPath(new URL("../node_modules/vite/bin/vite.js", import.meta.url));
const viteArgs = [viteBinPath];

const nextOpenPathByMode = {
  next: "/?preview=1",
  "next:auth": "/auth?preview=1",
  "next:home": "/?preview=1",
  "next:servers": "/servers?preview=1",
  "next:plugins": "/plugins?preview=1",
  "next:settings": "/settings?preview=1",
};

if (mode && Object.hasOwn(nextOpenPathByMode, mode)) {
  viteArgs.push("--open", nextOpenPathByMode[mode], ...restArgs);
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
