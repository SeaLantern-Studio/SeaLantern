import { i18n } from "@language";
import taxonomyData from "@shared/server-core-taxonomy.json";

interface ServerCoreTaxonomyAlias {
  value: string;
  label?: string;
}

interface ServerCoreTaxonomyEntry {
  key: string;
  label: string;
  i18nKey?: string;
  supportsPluginExtensions?: boolean;
  aliases?: ServerCoreTaxonomyAlias[];
}

interface ServerCoreTaxonomyDocument {
  entries: ServerCoreTaxonomyEntry[];
}

interface ServerCoreDisplayInfo {
  label: string;
  i18nKey?: string;
}

const taxonomy = taxonomyData as ServerCoreTaxonomyDocument;
const canonicalKeyByValue = new Map<string, string>();
const directDisplayInfoByValue = new Map<string, ServerCoreDisplayInfo>();
const displayInfoByCanonicalKey = new Map<string, ServerCoreDisplayInfo>();
const pluginExtensionSupportByCanonicalKey = new Map<string, boolean>();

for (const entry of taxonomy.entries) {
  const canonicalKey = entry.key.trim().toLowerCase();
  if (!canonicalKey) {
    continue;
  }

  const canonicalDisplayInfo = {
    label: entry.label,
    i18nKey: entry.i18nKey,
  } satisfies ServerCoreDisplayInfo;

  canonicalKeyByValue.set(canonicalKey, canonicalKey);
  displayInfoByCanonicalKey.set(canonicalKey, canonicalDisplayInfo);
  pluginExtensionSupportByCanonicalKey.set(canonicalKey, !!entry.supportsPluginExtensions);

  for (const alias of entry.aliases ?? []) {
    const normalizedAlias = alias.value.trim().toLowerCase();
    if (!normalizedAlias) {
      continue;
    }

    canonicalKeyByValue.set(normalizedAlias, canonicalKey);
    directDisplayInfoByValue.set(normalizedAlias, {
      label: alias.label ?? entry.label,
      i18nKey: alias.label ? undefined : entry.i18nKey,
    });
  }
}

function resolveDisplayLabel(info: ServerCoreDisplayInfo): string {
  return info.i18nKey ? i18n.t(info.i18nKey) : info.label;
}

export function normalizeServerCoreTypeKey(value: string): string {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return normalized;
  }

  return canonicalKeyByValue.get(normalized) ?? normalized;
}

export function serverCoreTypeSupportsPluginExtensions(value: string): boolean {
  const canonicalKey = normalizeServerCoreTypeKey(value);
  return pluginExtensionSupportByCanonicalKey.get(canonicalKey) ?? false;
}

export function formatServerCoreTypeLabel(value: string): string {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return value;
  }

  const directDisplayInfo = directDisplayInfoByValue.get(normalized);
  if (directDisplayInfo) {
    return resolveDisplayLabel(directDisplayInfo);
  }

  const canonical = normalizeServerCoreTypeKey(value);
  const canonicalDisplayInfo = displayInfoByCanonicalKey.get(canonical);

  return canonicalDisplayInfo ? resolveDisplayLabel(canonicalDisplayInfo) : value;
}
