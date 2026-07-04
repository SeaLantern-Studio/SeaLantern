import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const tauriProjectDir = path.join(repoRoot, 'backend', 'tauri-host');
const tauriCliPath = path.join(repoRoot, 'frontend', 'node_modules', '@tauri-apps', 'cli', 'tauri.js');
const scenarioAliases = new Map([
  ['dev:desktop-full', { baseCommand: 'dev' }],
  ['dev:desktop-min', { baseCommand: 'dev', cargoArgs: ['--no-default-features'] }],
  ['build:desktop-full', { baseCommand: 'build' }],
  ['build:desktop-min', { baseCommand: 'build', cargoArgs: ['--no-default-features'] }],
]);

function splitForwardedArgs(args) {
  const separatorIndex = args.indexOf('--');
  if (separatorIndex === -1) {
    return { tauriArgs: args, cargoArgs: [] };
  }

  return {
    tauriArgs: args.slice(0, separatorIndex),
    cargoArgs: args.slice(separatorIndex + 1),
  };
}

function buildScenarioArgs(command, rest) {
  const scenario = scenarioAliases.get(command);
  if (!scenario) {
    return [tauriCliPath, command, ...rest];
  }

  const { baseCommand, cargoArgs = [] } = scenario;
  if (cargoArgs.length === 0) {
    return [tauriCliPath, baseCommand, ...rest];
  }

  const forwarded = splitForwardedArgs(rest);
  return [
    tauriCliPath,
    baseCommand,
    ...forwarded.tauriArgs,
    '--',
    ...forwarded.cargoArgs,
    ...cargoArgs,
  ];
}

if (args.length === 0) {
  console.error('Usage: pnpm run tauri <command> [args...]');
  console.error('Scenarios: dev:desktop-full, dev:desktop-min, build:desktop-full, build:desktop-min');
  process.exit(1);
}

const [command, ...rest] = args;

const tauriArgs = buildScenarioArgs(command, rest);

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
