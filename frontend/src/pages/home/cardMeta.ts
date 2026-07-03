import { i18n } from "@language";
import type { NextHomeCardLayoutMeta } from "./layoutContract";

export function resolveNextHomeCardTitle(meta: Pick<NextHomeCardLayoutMeta, "id" | "title" | "titleKey">): string {
  if (meta.titleKey) {
    return i18n.t(meta.titleKey);
  }

  if (meta.title?.trim()) {
    return meta.title.trim();
  }

  return meta.id;
}
