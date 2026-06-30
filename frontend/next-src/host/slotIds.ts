export const NEXT_HOST_SLOT_IDS = {
  shellSidebarPrimary: "shell.sidebar.primary",
  shellHeaderPrimaryActions: "shell.header.primary_actions",
  pageHeaderActions: "page.header.actions",
  pageContentBefore: "page.content.before",
  pageContentAfter: "page.content.after",
  overlayGlobal: "overlay.global",
} as const;

export type NextHostSlotId = (typeof NEXT_HOST_SLOT_IDS)[keyof typeof NEXT_HOST_SLOT_IDS];
