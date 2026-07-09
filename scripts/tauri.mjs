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
const helpCommands = new Set(['help', '-h', '--help']);

function printUsage() {
  console.log('Usage:');
  console.log('  node scripts/tauri.mjs <scenario> [args...]');
  console.log('  pnpm tauri:dev');
  console.log('  pnpm tauri:build');
  console.log('');
  console.log('Scenarios:');
  for (const name of scenarioAliases.keys()) {
    console.log(`  ${name}`);
  }
}

function splitForwardedArgs(inputArgs) {
  const separatorIndex = inputArgs.indexOf('--');
  if (separatorIndex === -1) {
    return { tauriArgs: inputArgs, cargoArgs: [] };
  }

  return {
    tauriArgs: inputArgs.slice(0, separatorIndex),
    cargoArgs: inputArgs.slice(separatorIndex + 1),
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
  printUsage();
  process.exit(0);
}

const [command, ...rest] = args;

if (helpCommands.has(command)) {
  printUsage();
  process.exit(0);
}

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
