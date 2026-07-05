import type { Component } from "vue";

export const HOME_GRID_COLUMNS = 12;
export const HOME_GRID_MAX_ROWS = 48;
export const HOME_GRID_GAP = 16;
export const HOME_GRID_ROW_HEIGHT = 92;

export type NextHomeCardSection = "summary" | "operations" | "workspace" | "attention";
export type NextHomeHostCardSection = Extract<NextHomeCardSection, "operations" | "attention">;

export type NextHomeBuiltinCardKind =
  | "summary-band"
  | "operations-band"
  | "system-overview"
  | "cpu-usage-spotlight"
  | "memory-usage-spotlight"
  | "instance-count-spotlight"
  | "workspace-band"
  | "attention-band";

export type NextHomePluginCardKind = `plugin:${string}`;

export type NextHomeCardKind = NextHomeBuiltinCardKind | NextHomePluginCardKind;

export interface NextHomeHostCardDefinition {
  kind: NextHomePluginCardKind;
  title: string;
  section: NextHomeHostCardSection;
  component: Component;
  defaultCols?: 4 | 6;
  defaultRows?: 3 | 4 | 5 | 6;
}

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
  titleKey?: string | null;
  title?: string | null;
  section: NextHomeCardSection;
  minCols: 1 | 2 | 3 | 4 | 6 | 8 | 12;
  defaultCols: 1 | 2 | 3 | 4 | 6 | 8 | 12;
  maxCols?: 1 | 2 | 3 | 4 | 6 | 8 | 12;
  minRows: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
  defaultRows: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
  maxRows?: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;
  resizable: boolean;
  movable: boolean;
  removable: boolean;
  autoHeight?: boolean;
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
    removable: false,
    maxInstances: 1,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "operations-band": {
    id: "operations-band",
    titleKey: "shell.home_card_operations_title",
    section: "operations",
    minCols: 12,
    defaultCols: 12,
    maxCols: 12,
    minRows: 3,
    defaultRows: 3,
    maxRows: 3,
    resizable: false,
    movable: true,
    removable: false,
    maxInstances: 1,
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
    defaultRows: 4,
    maxRows: 8,
    resizable: true,
    movable: true,
    removable: true,
    autoHeight: true,
    maxInstances: 1,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "cpu-usage-spotlight": {
    id: "cpu-usage-spotlight",
    titleKey: "shell.home_card_cpu_usage_title",
    section: "operations",
    minCols: 4,
    defaultCols: 4,
    maxCols: 6,
    minRows: 3,
    defaultRows: 3,
    maxRows: 6,
    resizable: true,
    movable: true,
    removable: true,
    autoHeight: true,
    maxInstances: 1,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "memory-usage-spotlight": {
    id: "memory-usage-spotlight",
    titleKey: "shell.home_card_memory_usage_title",
    section: "operations",
    minCols: 4,
    defaultCols: 4,
    maxCols: 6,
    minRows: 3,
    defaultRows: 3,
    maxRows: 6,
    resizable: true,
    movable: true,
    removable: true,
    autoHeight: true,
    maxInstances: 1,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "instance-count-spotlight": {
    id: "instance-count-spotlight",
    titleKey: "shell.home_card_instance_count_title",
    section: "workspace",
    minCols: 4,
    defaultCols: 4,
    maxCols: 6,
    minRows: 3,
    defaultRows: 3,
    maxRows: 5,
    resizable: true,
    movable: true,
    removable: true,
    autoHeight: true,
    maxInstances: 1,
    provider: "builtin",
    pluginRendererKey: null,
  },
  "workspace-band": {
    id: "workspace-band",
    titleKey: "shell.home_card_workspace_title",
    section: "workspace",
    minCols: 8,
    defaultCols: 8,
    maxCols: 12,
    minRows: 5,
    defaultRows: 7,
    maxRows: 8,
    resizable: true,
    movable: true,
    removable: false,
    autoHeight: true,
    maxInstances: 1,
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
    defaultRows: 3,
    maxRows: 8,
    resizable: true,
    movable: true,
    removable: true,
    autoHeight: true,
    maxInstances: 1,
    provider: "builtin",
    pluginRendererKey: null,
  },
};

export const NEXT_HOME_DEFAULT_LAYOUT: NextHomeCardInstance[] = [
  {
    instanceId: "summary-band-1",
    kind: "summary-band",
    x: 0,
    y: 0,
    width: 12,
    height: 2,
    colStart: 1,
    rowStart: 1,
    colSpan: 12,
    rowSpan: 2,
    zIndex: 1,
  },
  {
    instanceId: "operations-band-1",
    kind: "operations-band",
    x: 0,
    y: 2,
    width: 12,
    height: 3,
    colStart: 1,
    rowStart: 3,
    colSpan: 12,
    rowSpan: 3,
    zIndex: 2,
  },
  {
    instanceId: "system-overview-1",
    kind: "system-overview",
    x: 8,
    y: 5,
    width: 4,
    height: 4,
    colStart: 9,
    rowStart: 6,
    colSpan: 4,
    rowSpan: 4,
    zIndex: 3,
  },
  {
    instanceId: "workspace-band-1",
    kind: "workspace-band",
    x: 0,
    y: 5,
    width: 8,
    height: 7,
    colStart: 1,
    rowStart: 6,
    colSpan: 8,
    rowSpan: 7,
    zIndex: 4,
  },
  {
    instanceId: "attention-band-1",
    kind: "attention-band",
    x: 8,
    y: 9,
    width: 4,
    height: 3,
    colStart: 9,
    rowStart: 10,
    colSpan: 4,
    rowSpan: 3,
    zIndex: 5,
  },
];

export function isBuiltinNextHomeCardKind(kind: NextHomeCardKind): kind is NextHomeBuiltinCardKind {
  return kind in NEXT_HOME_CARD_LAYOUTS;
}

export function isNextHomePluginCardKind(kind: string): kind is NextHomePluginCardKind {
  return kind.startsWith("plugin:");
}

function normalizeHostCardCols(section: NextHomeHostCardSection, value?: 4 | 6): 4 | 6 {
  if (value === 6) {
    return 6;
  }

  return section === "operations" ? 6 : 4;
}

function normalizeHostCardRows(section: NextHomeHostCardSection, value?: 3 | 4 | 5 | 6): 3 | 4 | 5 | 6 {
  if (value) {
    return value;
  }

  return section === "operations" ? 3 : 4;
}

export function createNextHomeHostCardLayoutMeta(
  definition: NextHomeHostCardDefinition,
): NextHomeCardLayoutMeta {
  const defaultCols = normalizeHostCardCols(definition.section, definition.defaultCols);
  const defaultRows = normalizeHostCardRows(definition.section, definition.defaultRows);

  return {
    id: definition.kind,
    title: definition.title.trim() || definition.kind,
    titleKey: null,
    section: definition.section,
    minCols: defaultCols,
    defaultCols,
    maxCols: defaultCols,
    minRows: defaultRows,
    defaultRows,
    maxRows: defaultRows,
    resizable: false,
    movable: true,
    removable: true,
    maxInstances: 1,
    provider: "plugin",
    pluginRendererKey: definition.kind,
  };
}
