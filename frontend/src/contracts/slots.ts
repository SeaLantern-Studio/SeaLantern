export const NEXT_HOST_SLOT_SCOPES = {
  shell: "shell",
  page: "page",
  overlay: "overlay",
} as const;

export type NextHostSlotScope = (typeof NEXT_HOST_SLOT_SCOPES)[keyof typeof NEXT_HOST_SLOT_SCOPES];

export const NEXT_HOST_SLOT_IDS = {
  sidebarPrimary: "shell.sidebar.primary",
  headerPrimaryActions: "shell.header.primary_actions",
  pageHeaderActions: "page.header.actions",
  pageContentBefore: "page.content.before",
  pageContentAfter: "page.content.after",
  overlayGlobal: "overlay.global",
} as const;

export type NextHostSlotId = (typeof NEXT_HOST_SLOT_IDS)[keyof typeof NEXT_HOST_SLOT_IDS];
