/// MSL 镜像站 API 接口
/// API 文档: https://api.mslmc.cn/v3
/// API 基础地址: https://api.mslmc.cn/v3

import { tauriInvoke } from "@api/tauri";

/// 服务端类型接口
export interface ServerType {
  name: string;
  description?: string;
}

/// 服务端分类接口
export interface ServerClassify {
  pluginsCore: string[];
  pluginsAndModsCore: string[];
  modsCore_Forge: string[];
  modsCore_Fabric: string[];
  vanillaCore: string[];
  bedrockCore: string[];
  proxyCore: string[];
}

/// 下载信息接口
export interface DownloadInfo {
  url: string;
  sha256?: string;
}

/// 获取所有支持的服务端类型
/// @returns 返回服务端类型数组
export async function getServerTypes(): Promise<string[]> {
  try {
    return await tauriInvoke<string[]>("get_msl_server_types");
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`获取服务端类型失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取服务端类型失败: 未知错误", { cause: error });
  }
}

/// 获取特定服务端的简介
/// @param server - 服务端类型（如 "paper", "purpur"）
/// @returns 返回服务端简介
export async function getServerDescription(server: string): Promise<string> {
  try {
    return await tauriInvoke<string>("get_msl_server_description", { server });
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`获取服务端简介失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取服务端简介失败: 未知错误", { cause: error });
  }
}

/// 获取服务端分类
/// @returns 返回服务端分类信息
export async function getServerClassify(): Promise<ServerClassify> {
  try {
    return await tauriInvoke<ServerClassify>("get_msl_server_classify");
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`获取服务端分类失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取服务端分类失败: 未知错误", { cause: error });
  }
}

/// 获取特定服务端支持的 MC 版本列表
/// @param server - 服务端类型（如 "paper", "purpur"）
/// @returns 返回 MC 版本数组
export async function getServerVersions(server: string): Promise<string[]> {
  try {
    return await tauriInvoke<string[]>("get_msl_server_versions", { server });
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`获取版本列表失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取版本列表失败: 未知错误", { cause: error });
  }
}

/// 获取服务端的所有构建信息
/// @param server - 服务端类型（如 "paper", "purpur"）
/// @param version - MC 版本号（如 "1.21"）
/// @returns 返回构建版本数组
export async function getServerBuilds(server: string, version: string): Promise<string[]> {
  try {
    return await tauriInvoke<string[]>("get_msl_server_builds", { server, version });
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`获取构建列表失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取构建列表失败: 未知错误", { cause: error });
  }
}

/// 获取服务端下载地址
/// @param server - 服务端类型（如 "paper", "purpur"）
/// @param version - MC 版本号（如 "1.21"）
/// @param build - 构建版本（如 "latest"，可选）
/// @returns 返回下载信息，包含 URL 和 SHA256（如果有）
export async function getServerDownloadUrl(
  server: string,
  version: string,
  build?: string,
): Promise<DownloadInfo> {
  try {
    return await tauriInvoke<DownloadInfo>("get_msl_server_download_url", {
      server,
      version,
      build: build || "latest",
    });
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`获取下载地址失败: ${error.message}`, { cause: error });
    }
    throw new Error("获取下载地址失败: 未知错误", { cause: error });
  }
}
