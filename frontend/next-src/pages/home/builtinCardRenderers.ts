import NextHomeAttentionCardRenderer from "../../components/home/renderers/NextHomeAttentionCardRenderer.vue";
import NextHomeOperationsCardRenderer from "../../components/home/renderers/NextHomeOperationsCardRenderer.vue";
import NextHomePluginCardRenderer from "../../components/home/renderers/NextHomePluginCardRenderer.vue";
import NextHomeSummaryCardRenderer from "../../components/home/renderers/NextHomeSummaryCardRenderer.vue";
import NextHomeSystemOverviewCardRenderer from "../../components/home/renderers/NextHomeSystemOverviewCardRenderer.vue";
import NextHomeWorkspaceCardRenderer from "../../components/home/renderers/NextHomeWorkspaceCardRenderer.vue";
import type { NextHomeCardRendererRegistry } from "./cardRendererContract";

export const NEXT_HOME_BUILTIN_CARD_RENDERERS: NextHomeCardRendererRegistry = {
  "summary-band": NextHomeSummaryCardRenderer,
  "operations-band": NextHomeOperationsCardRenderer,
  "system-overview": NextHomeSystemOverviewCardRenderer,
  "workspace-band": NextHomeWorkspaceCardRenderer,
  "attention-band": NextHomeAttentionCardRenderer,
};

export { NextHomePluginCardRenderer };

