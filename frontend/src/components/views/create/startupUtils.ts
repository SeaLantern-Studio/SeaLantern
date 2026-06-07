import type { StartupCandidate, StartupMode } from "@components/views/create/startupTypes";

const STARTER_MAIN_CLASS_PREFIX = "net.neoforged.serverstarterjar";

export function isStarterMainClass(mainClass: string | null): boolean {
  return !!mainClass?.startsWith(STARTER_MAIN_CLASS_PREFIX);
}

export function normalizePathForCompare(path: string): string {
  return path.replace(/\\/g, "/").replace(/\/+$/, "");
}

export function isStrictChildPath(candidatePath: string, parentPath: string): boolean {
  const candidateNorm = normalizePathForCompare(candidatePath.trim());
  const parentNorm = normalizePathForCompare(parentPath.trim());
  if (!candidateNorm || !parentNorm) {
    return false;
  }

  const windowsLike = /^[a-zA-Z]:/.test(candidateNorm) || /^[a-zA-Z]:/.test(parentNorm);
  const candidate = windowsLike ? candidateNorm.toLowerCase() : candidateNorm;
  const parent = windowsLike ? parentNorm.toLowerCase() : parentNorm;

  return candidate !== parent && candidate.startsWith(`${parent}/`);
}

export function getPathSeparator(path: string): string {
  return path.includes("\\") ? "\\" : "/";
}

export function joinPath(base: string, entryName: string): string {
  const trimmedBase = base.replace(/[\\/]+$/, "");
  const sep = getPathSeparator(base);
  return `${trimmedBase}${sep}${entryName}`;
}

export function getPathName(path: string): string {
  const segments = path.split(/[\\/]/).filter(Boolean);
  return segments.length > 0 ? segments[segments.length - 1] : path;
}

export function getParentPath(path: string): string {
  const trimmed = path.trim();
  if (!trimmed) {
    return "";
  }

  const normalized = trimmed.replace(/[\\/]+$/, "");
  const lastSeparator = Math.max(normalized.lastIndexOf("\\"), normalized.lastIndexOf("/"));
  if (lastSeparator < 0) {
    return "";
  }

  return normalized.slice(0, lastSeparator);
}

export function detectVersionCandidatesFromText(input: string, knownVersions: string[]): string[] {
  const trimmed = input.trim();
  if (!trimmed) {
    return [];
  }

  const lowered = trimmed.toLowerCase();
  const matches = knownVersions.filter((version) => lowered.includes(version.toLowerCase()));
  return [...new Set(matches)].sort((left, right) => right.length - left.length);
}

export function containsIoRedirection(command: string): boolean {
  const stripped = command.replace(/"[^"]*"|'[^']*'/g, "");
  return /(>>?|<<|[0-9]?>|[0-9]?<|\|)/.test(stripped);
}

export function sortStartupCandidates(candidates: StartupCandidate[]): StartupCandidate[] {
  return [...candidates].toSorted((a, b) => {
    if (a.recommended !== b.recommended) {
      return a.recommended - b.recommended;
    }
    return a.label.localeCompare(b.label);
  });
}

export function mapStartupModeForApi(
  mode: Exclude<StartupMode, "custom">,
): "jar" | "bat" | "sh" | "ps1" {
  switch (mode) {
    case "bat":
      return "bat";
    case "sh":
      return "sh";
    case "ps1":
      return "ps1";
    default:
      return "jar";
  }
}

export function mapStartupModeForModpack(
  mode: StartupMode,
): "starter" | "jar" | "bat" | "sh" | "ps1" | "custom" {
  if (mode === "starter") {
    return "starter";
  }
  if (mode === "custom") {
    return "custom";
  }
  return mapStartupModeForApi(mode);
}

export function resolveExecutablePathForTarget(
  executablePath: string,
  sourceDir: string,
  targetDir: string,
): string {
  if (normalizePathForCompare(sourceDir) === normalizePathForCompare(targetDir)) {
    return executablePath;
  }

  const sourceNorm = normalizePathForCompare(sourceDir);
  const executableNorm = normalizePathForCompare(executablePath);
  if (!executableNorm.startsWith(`${sourceNorm}/`)) {
    return executablePath;
  }

  const relative = executableNorm.slice(sourceNorm.length + 1);
  const targetSeparator = getPathSeparator(targetDir);
  return joinPath(targetDir, relative.split("/").join(targetSeparator));
}
