export type ResourceSource = "modrinth" | "curseforge";

export interface ResourceSearchResult {
  id: string;
  name: string;
  summary: string;
  source: ResourceSource;
  sourceUrl?: string;
  iconUrl?: string;
  author?: string;
  downloads?: number;
  latestVersion?: string;
}

// 一页搜索结果:结果列表 + 是否还有更多(任一来源返回满页即视为还有更多)
export interface ResourceSearchPage {
  results: ResourceSearchResult[];
  hasMore: boolean;
}

const MODRINTH_SEARCH_URL = "https://api.modrinth.com/v2/search";
const CURSEFORGE_SEARCH_URL = "https://api.curseforge.com/v1/mods/search";
const CURSEFORGE_API_KEY = import.meta.env.VITE_CURSEFORGE_API_KEY || "";
const CURSEFORGE_GAME_ID = 432; // Minecraft

function formatSearchError(reason: unknown): string {
  if (reason instanceof Error) return reason.message;
  if (typeof reason === "object" && reason !== null && "message" in reason) {
    const message = (reason as { message?: unknown }).message;
    if (typeof message === "string") return message;
  }
  return String(reason);
}

function normalizeModrinthHit(hit: any): ResourceSearchResult {
  return {
    id: hit.project_id || hit.id || "",
    name: hit.title || hit.name || "",
    summary: hit.summary || hit.description || "",
    source: "modrinth",
    sourceUrl:
      hit.website_url ||
      hit.project_url ||
      hit.url ||
      `https://modrinth.com/${hit.project_type || "project"}/${hit.slug || hit.title || hit.project_id}`,
    iconUrl: hit.icon_url || hit.iconUrl,
    author:
      hit.author ||
      hit.authors?.map((author: any) => author.username || author.name).join(", ") ||
      undefined,
    downloads: hit.downloads || hit.stats?.downloads_total,
    latestVersion: hit.versions?.[0] || hit.latest_version || undefined,
  };
}

function normalizeCurseForgeHit(hit: any): ResourceSearchResult {
  const icon = hit.logo || hit.logo_url || hit.logoUrl;
  return {
    id: hit.id?.toString() || "",
    name: hit.name || "",
    summary: hit.summary || hit.slug || "",
    source: "curseforge",
    sourceUrl: hit.links?.websiteUrl || hit.websiteUrl || hit.link || undefined,
    iconUrl: icon?.thumbnailUrl || icon?.url || icon?.url512 || undefined,
    author: hit.authors?.map((author: any) => author.name).join(", ") || undefined,
    downloads: hit.downloadCount || hit.downloads || undefined,
    latestVersion: hit.latestFiles?.[0]?.displayName || hit.latestFiles?.[0]?.fileName || undefined,
  };
}

async function fetchModrinth(
  query: string,
  limit: number,
  offset: number,
): Promise<ResourceSearchResult[]> {
  const url = new URL(MODRINTH_SEARCH_URL);
  if (query) {
    url.searchParams.set("query", query);
  }
  url.searchParams.set("limit", limit.toString());
  url.searchParams.set("offset", offset.toString());
  // 统一按下载量排序,无搜索词时即为热门内容
  url.searchParams.set("index", "downloads");

  const response = await fetch(url.toString(), {
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Modrinth API error ${response.status}`);
  }

  const data = await response.json();
  return Array.isArray(data.hits) ? data.hits.map(normalizeModrinthHit) : [];
}

async function fetchCurseForge(
  query: string,
  limit: number,
  offset: number,
): Promise<ResourceSearchResult[]> {
  if (!CURSEFORGE_API_KEY) {
    return [];
  }

  const url = new URL(CURSEFORGE_SEARCH_URL);
  url.searchParams.set("gameId", CURSEFORGE_GAME_ID.toString());
  url.searchParams.set("pageSize", limit.toString());
  // searchFilter 才是 CurseForge 的关键词搜索参数,直接传入查询词(原先误用了不存在的 search 参数,导致查询被忽略)
  if (query) {
    url.searchParams.set("searchFilter", query);
  }
  url.searchParams.set("index", offset.toString());
  // sortField 6 = TotalDownloads,统一按下载量排序
  url.searchParams.set("sortField", "6");
  url.searchParams.set("sortOrder", "desc");

  const response = await fetch(url.toString(), {
    headers: {
      Accept: "application/json",
      "x-api-key": CURSEFORGE_API_KEY,
    },
  });

  if (!response.ok) {
    throw new Error(`CurseForge API error ${response.status}`);
  }

  const data = await response.json();
  return Array.isArray(data.data) ? data.data.map(normalizeCurseForgeHit) : [];
}

export async function searchResources(
  query = "",
  limit = 20,
  offset = 0,
): Promise<ResourceSearchPage> {
  const trimmedQuery = query.trim();

  const requests = [
    fetchModrinth(trimmedQuery, limit, offset),
    fetchCurseForge(trimmedQuery, limit, offset),
  ];

  const settled = await Promise.allSettled(requests);
  const results: ResourceSearchResult[] = [];
  let modrinthCount = 0;
  let curseforgeCount = 0;

  if (settled[0].status === "fulfilled") {
    modrinthCount = settled[0].value.length;
    results.push(...settled[0].value);
  }

  if (settled[1].status === "fulfilled") {
    curseforgeCount = settled[1].value.length;
    results.push(...settled[1].value);
  }

  if (results.length === 0) {
    if (settled[0].status === "rejected" && settled[1].status === "rejected") {
      throw new Error(
        `Both sources failed: Modrinth (${formatSearchError(settled[0].reason)}), CurseForge (${formatSearchError(settled[1].reason)})`,
      );
    }
  }

  const sorted = results.toSorted((a, b) => {
    const aHeat = a.downloads ?? 0;
    const bHeat = b.downloads ?? 0;
    return bHeat - aHeat;
  });

  // 任一来源返回满页,说明该来源可能还有下一页可翻
  const hasMore = modrinthCount >= limit || curseforgeCount >= limit;

  return { results: sorted, hasMore };
}

export const curseforgeApiKey = CURSEFORGE_API_KEY;
