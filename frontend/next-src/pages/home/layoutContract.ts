export const HOME_GRID_COLUMNS = 12;
export const HOME_GRID_MAX_ROWS = 48;
export const HOME_GRID_GAP = 16;
export const HOME_GRID_ROW_HEIGHT = 92;

export type NextHomeCardSection = "summary" | "operations" | "workspace" | "attention";

export type NextHomeBuiltinCardKind =
  | "summary-band"
  | "operations-band"
  | "system-overview"
  | "workspace-band"
  | "attention-band";

export type NextHomePluginCardKind = `plugin:${string}`;

export type NextHomeCardKind = NextHomeBuiltinCardKind | NextHomePluginCardKind;

export interface NextHomeCardInstance {
  instanceId: string;
  kind: NextHomeCardKind;
  x: number;
  y: number;
  width: number;
  height: number;
  colStart: number;
  rowStart: number;
  colSpan: number;
  rowSpan: number;
  zIndex: number;
}

export interface NextHomeCardLayoutMeta {
  id: NextHomeCardKind;
  titleKey: string;
  section: NextHomeCardSection;
  minCols: 1 | 2 | 3 | 4 | 6 | 8 | 12;
  defaultCols: 1 | 2 | 3 | 4 | 6 | 8 | 12;
  maxCols?: 1 | 2 | 3 | 4 | 6 | 8 | 12;
  minRows: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
  defaultRows: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
  maxRows?: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
  resizable: boolean;
  movable: boolean;
  maxInstances?: number;
  provider?: "builtin" | "plugin";
  pluginRendererKey?: string | null;
}

export type NextHomeCardRegistry = Record<NextHomeCardKind, NextHomeCardLayoutMeta>;

export const NEXT_HOME_CARD_LAYOUTS: Record<NextHomeBuiltinCardKind, NextHomeCardLayoutMeta> = {
  "summary-band": {
    id: "summary-band",
    titleKey: "shell.home_card_summary_title",
    section: "summary",
    minCols: 12,
    defaultCols: 12,
    maxCols: 12,
    minRows: 2,
    defaultRows: 2,
    maxRows: 3,
    resizable: false,
    movable: true,
    maxInstances: 5,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "operations-band": {
    id: "operations-band",
    titleKey: "shell.home_card_operations_title",
    section: "operations",
    minCols: 6,
    defaultCols: 8,
    maxCols: 12,
    minRows: 3,
    defaultRows: 3,
    maxRows: 5,
    resizable: true,
    movable: true,
    maxInstances: 5,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "system-overview": {
    id: "system-overview",
    titleKey: "shell.home_card_system_overview_title",
    section: "operations",
    minCols: 4,
    defaultCols: 4,
    maxCols: 6,
    minRows: 4,
    defaultRows: 6,
    maxRows: 8,
    resizable: true,
    movable: true,
    maxInstances: 5,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "workspace-band": {
    id: "workspace-band",
    titleKey: "shell.home_card_workspace_title",
    section: "workspace",
    minCols: 6,
    defaultCols: 8,
    maxCols: 12,
    minRows: 4,
    defaultRows: 6,
    maxRows: 8,
    resizable: true,
    movable: true,
    maxInstances: 5,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "attention-band": {
    id: "attention-band",
    titleKey: "shell.home_card_attention_title",
    section: "attention",
    minCols: 4,
    defaultCols: 4,
    maxCols: 6,
    minRows: 4,
    defaultRows: 4,
    maxRows: 8,
    resizable: true,
    movable: true,
    maxInstances: 5,
    provider: "builtin",
    pluginRendererKey: null,
  },
};

export const NEXT_HOME_DEFAULT_LAYOUT: NextHomeCardInstance[] = [
  { instanceId: "summary-band-1", kind: "summary-band", x: 0, y: 0, width: 12, height: 2, colStart: 1, rowStart: 1, colSpan: 12, rowSpan: 2, zIndex: 1 },
  { instanceId: "operations-band-1", kind: "operations-band", x: 0, y: 2, width: 8, height: 3, colStart: 1, rowStart: 3, colSpan: 8, rowSpan: 3, zIndex: 2 },
  { instanceId: "system-overview-1", kind: "system-overview", x: 8, y: 2, width: 4, height: 6, colStart: 9, rowStart: 3, colSpan: 4, rowSpan: 6, zIndex: 3 },
  { instanceId: "workspace-band-1", kind: "workspace-band", x: 0, y: 5, width: 8, height: 6, colStart: 1, rowStart: 6, colSpan: 8, rowSpan: 6, zIndex: 4 },
  { instanceId: "attention-band-1", kind: "attention-band", x: 8, y: 8, width: 4, height: 4, colStart: 9, rowStart: 9, colSpan: 4, rowSpan: 4, zIndex: 5 },
];

export function isBuiltinNextHomeCardKind(kind: NextHomeCardKind): kind is NextHomeBuiltinCardKind {
  return kind in NEXT_HOME_CARD_LAYOUTS;
}
