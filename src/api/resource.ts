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

const MODRINTH_SEARCH_URL = "https://api.modrinth.com/v2/search";
const CURSEFORGE_SEARCH_URL = "https://api.curseforge.com/v1/mods/search";
const CURSEFORGE_API_KEY = import.meta.env.VITE_CURSEFORGE_API_KEY || "";
const CURSEFORGE_GAME_ID = 432; // Minecraft

function normalizeModrinthHit(hit: any): ResourceSearchResult {
  return {
    id: hit.project_id || hit.id || "",
    name: hit.title || hit.name || "",
    summary: hit.summary || hit.description || "",
    source: "modrinth",
    sourceUrl: hit.website_url || hit.project_url || hit.url || `https://modrinth.com/${hit.project_type || "project"}/${hit.slug || hit.title || hit.project_id}`,
    iconUrl: hit.icon_url || hit.iconUrl,
    author: hit.author || hit.authors?.map((author: any) => author.username || author.name).join(", ") || undefined,
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

async function fetchModrinth(query: string, limit: number): Promise<ResourceSearchResult[]> {
  const url = new URL(MODRINTH_SEARCH_URL);
  url.searchParams.set("query", query);
  url.searchParams.set("limit", limit.toString());

  const response = await fetch(url.toString(), {
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`Modrinth API error ${response.status}`);
  }

  const data = await response.json();
  return Array.isArray(data.hits)
    ? data.hits.map(normalizeModrinthHit)
    : [];
}

async function fetchCurseForge(query: string, limit: number): Promise<ResourceSearchResult[]> {
  if (!CURSEFORGE_API_KEY) {
    return [];
  }

  const url = new URL(CURSEFORGE_SEARCH_URL);
  url.searchParams.set("gameId", CURSEFORGE_GAME_ID.toString());
  url.searchParams.set("pageSize", limit.toString());
  url.searchParams.set("searchFilter", "mod");
  url.searchParams.set("search", query);

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
  return Array.isArray(data.data)
    ? data.data.map(normalizeCurseForgeHit)
    : [];
}

export async function searchResources(query: string, limit = 20): Promise<ResourceSearchResult[]> {
  const trimmedQuery = query.trim();
  if (!trimmedQuery) {
    return [];
  }

  const requests = [
    fetchModrinth(trimmedQuery, limit),
    fetchCurseForge(trimmedQuery, limit),
  ];

  const settled = await Promise.allSettled(requests);
  const results: ResourceSearchResult[] = [];

  if (settled[0].status === "fulfilled") {
    results.push(...settled[0].value);
  }

  if (settled[1].status === "fulfilled") {
    results.push(...settled[1].value);
  }

  if (results.length === 0) {
    if (settled[0].status === "rejected" && settled[1].status === "rejected") {
      throw new Error(
        `Both sources failed: Modrinth (${settled[0].reason}), CurseForge (${settled[1].reason})`,
      );
    }
  }

  return results.sort((a, b) => {
    const aHeat = a.downloads ?? 0;
    const bHeat = b.downloads ?? 0;
    return bHeat - aHeat;
  });
}

export const curseforgeApiKey = CURSEFORGE_API_KEY;
