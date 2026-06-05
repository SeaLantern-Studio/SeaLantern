function splitPathSegments(path: string): string[] {
  return path.split(/[\\/]+/).filter((segment) => segment.length > 0);
}

export function compactPathMiddle(path: string, maxLength = 48): string {
  const normalized = path.trim();
  if (normalized.length <= maxLength) {
    return normalized;
  }

  const segments = splitPathSegments(normalized);
  if (segments.length <= 2) {
    const headLength = Math.max(8, Math.floor((maxLength - 5) / 2));
    const tailLength = Math.max(8, maxLength - 5 - headLength);
    return `${normalized.slice(0, headLength)} ... ${normalized.slice(-tailLength)}`;
  }

  const first = segments[0];
  const last = segments[segments.length - 1];
  const preferred = `${first}${normalized.includes("\\") ? "\\" : "/"}...${normalized.includes("\\") ? "\\" : "/"}${last}`;
  if (preferred.length <= maxLength) {
    return preferred;
  }

  const lastLength = Math.max(12, maxLength - 8);
  return `...${last.slice(-(lastLength - 3))}`;
}
