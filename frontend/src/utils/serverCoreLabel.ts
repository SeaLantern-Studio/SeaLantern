import { i18n } from "@language";

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

const CANONICAL_CORE_TYPE_LABELS: Record<string, string> = {
  allay: "AllayMC",
  airplane: "Airplane",
  "arclight-forge": "Arclight-Forge",
  "arclight-neoforge": "Arclight-NeoForge",
  bds: "BDS",
  bdsx: "BDSX",
  bukkit: "Bukkit",
  bungeecord: "BungeeCord",
  catserver: "CatServer",
  cuberite: "Cuberite",
  endstone: "Endstone",
  fabric: "Fabric",
  flamecord: "FlameCord",
  folia: "Folia",
  forge: "Forge",
  glowstone: "Glowstone",
  leaves: "Leaves",
  levilamina: "LeviLamina",
  liteloaderbds: "LiteLoaderBDS",
  lightfall: "Lightfall",
  minestom: "Minestom",
  neoforge: "NeoForge",
  nukkit: "Nukkit",
  paper: "Paper",
  pocketmine: "PocketMine-MP",
  powernukkitx: "PowerNukkitX",
  purpur: "Purpur",
  pufferfish: "Pufferfish",
  pumpkin: i18n.t("create.core_type_pumpkin"),
  quilt: "Quilt",
  sponge: "Sponge",
  spigot: "Spigot",
  travertine: "Travertine",
  tuinity: "Tuinity",
  vanilla: "Vanilla",
  velocity: "Velocity",
  waterfall: "Waterfall",
  youer: "Youer",
};

export function formatServerCoreTypeLabel(value: string): string {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return value;
  }

  const canonical = LEGACY_CORE_TYPE_ALIASES[normalized] ?? normalized;

  return CANONICAL_CORE_TYPE_LABELS[canonical] ?? value;
}
