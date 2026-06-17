import type { ServerInstance } from "@type/server";
import { formatServerCoreTypeLabel } from "@utils/serverCoreLabel";

const LEGACY_CORE_TYPE_ALIASES: Record<string, string> = {
  banner: "fabric",
  "arclight-fabric": "fabric",
  pufferfish_purpur: "pufferfish",
  "vanilla-snapshot": "vanilla",
  nukkitx: "nukkit",
  bedrock: "bds",
  leaf: "leaves",
  spongevanilla: "sponge",
  spongeforge: "forge",
};

const PLUGIN_SUPPORTED_CORE_TYPES = new Set([
  "allay",
  "arclight",
  "arclight-forge",
  "arclight-neoforge",
  "bdsx",
  "bukkit",
  "bungeecord",
  "catserver",
  "endstone",
  "flamecord",
  "folia",
  "glowstone",
  "leaves",
  "levilamina",
  "lightfall",
  "minestom",
  "mohist",
  "nukkit",
  "paper",
  "pocketmine",
  "powernukkitx",
  "pufferfish",
  "purpur",
  "sponge",
  "spigot",
  "travertine",
  "tuinity",
  "velocity",
  "waterfall",
]);

function normalizeServerCoreTypeKey(coreType: string): string {
  const normalized = coreType.trim().toLowerCase();
  return LEGACY_CORE_TYPE_ALIASES[normalized] ?? normalized;
}

export function serverSupportsPluginExtensions(server: ServerInstance): boolean {
  return PLUGIN_SUPPORTED_CORE_TYPES.has(normalizeServerCoreTypeKey(server.core_type));
}

export function getPluginUnsupportedReason(server: ServerInstance): string {
  return `当前服务端类型 ${formatServerCoreTypeLabel(server.core_type)} 不支持插件式扩展`;
}
