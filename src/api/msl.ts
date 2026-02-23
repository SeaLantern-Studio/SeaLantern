const API_BASE = "https://api.mslmc.cn/v3";

interface MSLResponse<T> {
  code: number;
  message: string;
  data: T;
}

interface ServerClassify {
  pluginsCore: string[];
  pluginsAndModsCore: string[];
  modsCore_Forge: string[];
  modsCore_Fabric: string[];
  vanillaCore: string[];
  bedrockCore: string[];
  proxyCore: string[];
}

interface VersionListResponse {
  versionList: string[];
}

interface ServerDownload {
  url: string;
  sha256?: string;
}

interface ServerBuild {
  build: string;
  date?: string;
}

const DEVICE_ID_KEY = "msl_device_id";

function getDeviceId(): string {
  let deviceId = localStorage.getItem(DEVICE_ID_KEY);
  if (!deviceId) {
    deviceId = "sl-" + Math.random().toString(36).substring(2, 15) + "-" + Date.now().toString(36);
    localStorage.setItem(DEVICE_ID_KEY, deviceId);
  }
  return deviceId;
}

async function fetchAPI<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      deviceID: getDeviceId(),
      "User-Agent": "SeaLantern/1.0",
      ...options.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const result: MSLResponse<T> = await response.json();

  if (result.code !== 200) {
    throw new Error(result.message || "API request failed");
  }

  return result.data;
}

export const mslApi = {
  async getServerClassify(): Promise<ServerClassify> {
    return fetchAPI<ServerClassify>("/query/server_classify");
  },

  async getServerVersions(server: string): Promise<string[]> {
    const result = await fetchAPI<VersionListResponse>(`/query/available_versions/${server}`);
    return result.versionList || [];
  },

  async getServerBuilds(server: string, version: string): Promise<ServerBuild[]> {
    return fetchAPI<ServerBuild[]>(`/query/server_builds/${server}/${version}`);
  },

  async getServerDownloadUrl(
    server: string,
    version: string,
    build?: string,
  ): Promise<ServerDownload> {
    const buildParam = build ? `?build=${build}` : "";
    return fetchAPI<ServerDownload>(`/download/server/${server}/${version}${buildParam}`);
  },

  async getServerIntro(server: string): Promise<{ name: string; description: string }> {
    return fetchAPI<{ name: string; description: string }>(`/query/server_intro/${server}`);
  },
};

export type { ServerClassify, ServerDownload, ServerBuild };
