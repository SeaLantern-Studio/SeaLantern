const DELETE_CONFIRMATION_ITEMS = [
  "potato",
  "carrot",
  "bedrock",
  "sand",
  "stone",
  "torch",
  "melon",
  "stick",
] as const;

export function hasHashLikeServerName(name: string): boolean {
  const normalized = name.trim();
  if (!normalized) {
    return false;
  }

  if (/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/i.test(normalized)) {
    return true;
  }

  const compactHex = normalized.replace(/[^0-9a-f]/gi, "");
  return compactHex.length >= 12 && compactHex.length >= Math.ceil(normalized.length * 0.6);
}

export function shouldUseDeleteConfirmationItem(name: string): boolean {
  const normalized = name.trim();
  return normalized.length > 8 || hasHashLikeServerName(normalized);
}

export function pickDeleteConfirmationItem(): string {
  const index = Math.floor(Math.random() * DELETE_CONFIRMATION_ITEMS.length);
  return DELETE_CONFIRMATION_ITEMS[index] ?? DELETE_CONFIRMATION_ITEMS[0];
}
