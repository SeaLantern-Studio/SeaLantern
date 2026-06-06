import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const tauriProjectDir = path.join(repoRoot, 'backend', 'tauri-host');
const tauriCliPath = path.join(repoRoot, 'frontend', 'node_modules', '@tauri-apps', 'cli', 'tauri.js');

if (args.length === 0) {
  console.error('Usage: pnpm run tauri <command> [args...]');
  process.exit(1);
}

const [command, ...rest] = args;
const tauriArgs = [tauriCliPath, command, ...rest];

const result = spawnSync(process.execPath, tauriArgs, {
  cwd: tauriProjectDir,
  stdio: 'inherit',
});

if (result.error) {
  console.error(`Failed to launch tauri CLI: ${result.error.message}`);
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 1);
