/// Paper 核心 API 接口
/// 提供 Paper 核心版本列表、构建版本和构建信息的获取功能

const PAPER_API_BASE = "https://api.papermc.io/v2";
const API_TIMEOUT = 10000;

interface PaperProjectResponse {
  project_id: string;
  project_name: string;
  version_groups: string[];
  versions: string[];
}

interface PaperVersionResponse {
  project_id: string;
  version: string;
  builds: number[];
}

interface PaperBuildChange {
  commit: string;
  summary: string;
  message: string;
}

interface PaperBuildDownload {
  name: string;
  sha256: string;
}

export interface PaperBuildInfo {
  project_id: string;
  project_name: string;
  version: string;
  build: number;
  time: string;
  channel: string;
  promoted: boolean;
  changes: PaperBuildChange[];
  downloads: {
    application: PaperBuildDownload;
  };
}

/// 获取 Paper 核心的所有版本列表
/// @returns 返回版本数组，从最新到最旧排序
export async function getPaperVersions(): Promise<string[]> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);

  try {
    const response = await fetch(`${PAPER_API_BASE}/projects/paper`, {
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error(`API 请求失败: ${response.status} ${response.statusText}`);
    }

    const data: PaperProjectResponse = await response.json();
    return data.versions.filter((v) => !v.includes("-rc")).toReversed();
  } catch (error) {
    if (error instanceof Error) {
      if (error.name === "AbortError") {
        throw new Error("请求超时，请检查网络连接", { cause: error });
      }
      throw new Error(`获取版本列表失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取版本列表失败: 未知错误", { cause: error });
  } finally {
    clearTimeout(timeoutId);
  }
}

/// 获取 Paper 核心指定版本的所有构建版本
/// @param version - Minecraft 版本号（如 "1.21.1"）
/// @returns 返回构建版本数组，从最新到最旧排序
export async function getPaperBuilds(version: string): Promise<number[]> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);

  try {
    const response = await fetch(`${PAPER_API_BASE}/projects/paper/versions/${version}`, {
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error(`API 请求失败: ${response.status} ${response.statusText}`);
    }

    const data: PaperVersionResponse = await response.json();
    return data.builds.toReversed();
  } catch (error) {
    if (error instanceof Error) {
      if (error.name === "AbortError") {
        throw new Error("请求超时，请检查网络连接", { cause: error });
      }
      throw new Error(`获取构建版本失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取构建版本失败: 未知错误", { cause: error });
  } finally {
    clearTimeout(timeoutId);
  }
}

/// 获取 Paper 核心指定构建的详细信息
/// @param version - Minecraft 版本号（如 "1.21.1"）
/// @param build - 构建版本号（如 120）
/// @returns 返回构建详细信息，包含时间戳和 SHA256 校验和
export async function getPaperBuildInfo(version: string, build: number): Promise<PaperBuildInfo> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);

  try {
    const response = await fetch(
      `${PAPER_API_BASE}/projects/paper/versions/${version}/builds/${build}`,
      {
        signal: controller.signal,
      },
    );

    if (!response.ok) {
      throw new Error(`API 请求失败: ${response.status} ${response.statusText}`);
    }

    const data: PaperBuildInfo = await response.json();
    return data;
  } catch (error) {
    if (error instanceof Error) {
      if (error.name === "AbortError") {
        throw new Error("请求超时，请检查网络连接", { cause: error });
      }
      throw new Error(`获取构建信息失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取构建信息失败: 未知错误", { cause: error });
  } finally {
    clearTimeout(timeoutId);
  }
}
