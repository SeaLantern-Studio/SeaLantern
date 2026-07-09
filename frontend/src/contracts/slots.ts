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

export const NEXT_HOST_SLOT_SCOPE_BY_ID: Record<NextHostSlotId, NextHostSlotScope> = {
  [NEXT_HOST_SLOT_IDS.sidebarPrimary]: NEXT_HOST_SLOT_SCOPES.shell,
  [NEXT_HOST_SLOT_IDS.headerPrimaryActions]: NEXT_HOST_SLOT_SCOPES.shell,
  [NEXT_HOST_SLOT_IDS.pageHeaderActions]: NEXT_HOST_SLOT_SCOPES.page,
  [NEXT_HOST_SLOT_IDS.pageContentBefore]: NEXT_HOST_SLOT_SCOPES.page,
  [NEXT_HOST_SLOT_IDS.pageContentAfter]: NEXT_HOST_SLOT_SCOPES.page,
  [NEXT_HOST_SLOT_IDS.overlayGlobal]: NEXT_HOST_SLOT_SCOPES.overlay,
};

export function getNextHostSlotScope(slotId: NextHostSlotId): NextHostSlotScope {
  return NEXT_HOST_SLOT_SCOPE_BY_ID[slotId];
}

export function isNextPageScopedHostSlot(slotId: NextHostSlotId): boolean {
  return getNextHostSlotScope(slotId) === NEXT_HOST_SLOT_SCOPES.page;
}
