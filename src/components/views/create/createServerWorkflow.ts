import { copyFile, exists, mkdir, readDir } from "@tauri-apps/plugin-fs";
import type { ParsedServerCoreInfo } from "@api/server";
import type { StartupCandidate } from "@components/views/create/startupTypes";
import { isStarterMainClass, joinPath, sortStartupCandidates } from "@components/views/create/startupUtils";
import { i18n } from "@language";

type ParseCore = (sourcePath: string) => Promise<ParsedServerCoreInfo>;

export function appendCustomCandidate(candidates: StartupCandidate[]): StartupCandidate[] {
  const custom: StartupCandidate = {
    id: "custom-command",
    mode: "custom",
    label: i18n.t("create.startup_candidate_custom"),
    detail: i18n.t("create.startup_candidate_custom_desc"),
    path: "",
    recommended: 9,
  };
  return [...sortStartupCandidates(candidates), custom];
}

export async function buildArchiveStartupCandidates(
  path: string,
  parseServerCoreType: ParseCore,
): Promise<StartupCandidate[]> {
  // 压缩包场景无法直接枚举脚本文件，先基于核心识别生成可选启动项。
  const result = await parseServerCoreType(path);
  const starter = isStarterMainClass(result.mainClass);
  const title = starter
    ? i18n.t("create.startup_candidate_starter")
    : i18n.t("create.startup_candidate_server_jar");

  return [
    {
      id: `archive-${starter ? "starter" : "jar"}`,
      mode: starter ? "starter" : "jar",
      label: title,
      detail: [result.coreType, result.mainClass].filter(Boolean).join(" · "),
      path: result.jarPath ?? "server.jar",
      recommended: starter ? 1 : 3,
    },
  ];
}

export async function buildFolderStartupCandidates(
  folderPath: string,
  parseServerCoreType: ParseCore,
  isWindows: boolean,
): Promise<StartupCandidate[]> {
  const entries = await readDir(folderPath);
  const fileEntries = entries.filter((entry) => entry.isFile);
  const candidates: StartupCandidate[] = [];

  // 先并发解析所有 JAR，避免后续循环中串行等待导致卡顿。
  const jarEntries = fileEntries.filter((entry) => entry.name.toLowerCase().endsWith(".jar"));
  const parsedJarMap = new Map<string, ParsedServerCoreInfo>();

  await Promise.all(
    jarEntries.map(async (entry) => {
      const fullPath = joinPath(folderPath, entry.name);
      try {
        parsedJarMap.set(fullPath, await parseServerCoreType(fullPath));
      } catch {
        parsedJarMap.set(fullPath, {
          coreType: i18n.t("create.source_core_unknown"),
          mainClass: null,
          jarPath: fullPath,
        });
      }
    }),
  );

  for (const entry of fileEntries) {
    const fileNameLower = entry.name.toLowerCase();
    const fullPath = joinPath(folderPath, entry.name);

    if (fileNameLower.endsWith(".jar")) {
      const parsed = parsedJarMap.get(fullPath);
      const starter = isStarterMainClass(parsed?.mainClass ?? null);
      const isServerJar = fileNameLower === "server.jar";

      candidates.push({
        id: `jar-${entry.name}`,
        mode: starter ? "starter" : "jar",
        label: starter
          ? i18n.t("create.startup_candidate_starter")
          : isServerJar
            ? i18n.t("create.startup_candidate_server_jar")
            : entry.name,
        detail: [parsed?.coreType, parsed?.mainClass].filter(Boolean).join(" · "),
        path: fullPath,
        recommended: starter ? 1 : isServerJar ? 3 : 4,
      });
      continue;
    }

    if (fileNameLower.endsWith(".bat")) {
      candidates.push({
        id: `bat-${entry.name}`,
        mode: "bat",
        label: entry.name,
        detail: i18n.t("create.startup_mode_script"),
        path: fullPath,
        recommended: 2,
      });
      continue;
    }

    if (fileNameLower.endsWith(".sh")) {
      candidates.push({
        id: `sh-${entry.name}`,
        mode: "sh",
        label: entry.name,
        detail: i18n.t("create.startup_mode_script"),
        path: fullPath,
        recommended: 2,
      });
      continue;
    }

    if (isWindows && fileNameLower.endsWith(".ps1")) {
      candidates.push({
        id: `ps1-${entry.name}`,
        mode: "ps1",
        label: entry.name,
        detail: i18n.t("create.startup_mode_script"),
        path: fullPath,
        recommended: 2,
      });
    }
  }

  return sortStartupCandidates(candidates);
}

export async function collectCopyConflicts(sourceDir: string, targetDir: string): Promise<string[]> {
  // 递归扫描所有子目录，提前给用户展示“将被覆盖”的清单。
  const conflicts: string[] = [];
  await collectCopyConflictsRecursive(sourceDir, targetDir, "", conflicts);
  return conflicts;
}

async function collectCopyConflictsRecursive(
  sourceDir: string,
  targetDir: string,
  relativePrefix: string,
  conflicts: string[],
) {
  const entries = await readDir(sourceDir);

  for (const entry of entries) {
    const sourceEntryPath = joinPath(sourceDir, entry.name);
    const targetEntryPath = joinPath(targetDir, entry.name);
    const relative = relativePrefix ? `${relativePrefix}/${entry.name}` : entry.name;

    if (await exists(targetEntryPath)) {
      conflicts.push(relative);
    }

    if (entry.isDirectory) {
      await collectCopyConflictsRecursive(sourceEntryPath, targetEntryPath, relative, conflicts);
    }
  }
}

export async function copyDirectoryRecursive(sourceDir: string, targetDir: string) {
  await mkdir(targetDir, { recursive: true });
  const entries = await readDir(sourceDir);

  for (const entry of entries) {
    const sourceEntryPath = joinPath(sourceDir, entry.name);
    const targetEntryPath = joinPath(targetDir, entry.name);

    if (entry.isDirectory) {
      await copyDirectoryRecursive(sourceEntryPath, targetEntryPath);
      continue;
    }

    if (entry.isFile) {
      await copyFile(sourceEntryPath, targetEntryPath);
    }
  }
}
