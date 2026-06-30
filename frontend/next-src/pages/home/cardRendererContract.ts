import type { Component } from "vue";
import type { NextHomeCardInstance, NextHomeCardKind, NextHomeCardLayoutMeta } from "./layoutContract";
import type { NextHomePageSummaryMetric, NextHomeServerCardModel, NextHomeSystemMetric } from "./useNextHomePage";

export interface NextHomeAlertItem {
  server: string;
  line: string;
}

export interface NextHomeCardRuntimeContext {
  previewMode: boolean;
  usingPreviewFallback: boolean;
  summaryMetrics: NextHomePageSummaryMetric[];
  systemMetrics: NextHomeSystemMetric[];
  statsViewMode: "gauge" | "detail";
  featuredServer: NextHomeServerCardModel | null;
  secondaryServers: NextHomeServerCardModel[];
  alertItems: NextHomeAlertItem[];
  lastUpdatedLabel: string;
  totalServerCount: number;
  isRefreshing: boolean;
  refreshNow: () => Promise<void>;
  toggleStatsViewMode: () => void;
  goToCreateServer: () => void;
  goToImportServer: () => void;
  toggleServer: (serverId: string) => Promise<void>;
}

export interface NextHomeCardRendererProps {
  instance: NextHomeCardInstance;
  meta: NextHomeCardLayoutMeta;
  context: NextHomeCardRuntimeContext;
}

export type NextHomeCardRendererRegistry = Partial<Record<NextHomeCardKind, Component>>;

