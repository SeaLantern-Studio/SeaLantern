import NextHomeAttentionCardRenderer from "../../components/home/renderers/NextHomeAttentionCardRenderer.vue";
import NextHomeMissingCardRenderer from "../../components/home/renderers/NextHomeMissingCardRenderer.vue";
import NextHomeOperationsCardRenderer from "../../components/home/renderers/NextHomeOperationsCardRenderer.vue";
import NextHomePluginCardRenderer from "../../components/home/renderers/NextHomePluginCardRenderer.vue";
import NextHomeSummaryCardRenderer from "../../components/home/renderers/NextHomeSummaryCardRenderer.vue";
import NextHomeSystemOverviewCardRenderer from "../../components/home/renderers/NextHomeSystemOverviewCardRenderer.vue";
import NextHomeWorkspaceCardRenderer from "../../components/home/renderers/NextHomeWorkspaceCardRenderer.vue";
import { isBuiltinNextHomeCardKind, type NextHomeCardKind } from "./layoutContract";
import type { NextHomeCardRendererRegistry, NextHomeResolvedCardRenderer } from "./cardRendererContract";

export const NEXT_HOME_BUILTIN_CARD_RENDERERS: NextHomeCardRendererRegistry = {
  "summary-band": NextHomeSummaryCardRenderer,
  "operations-band": NextHomeOperationsCardRenderer,
  "system-overview": NextHomeSystemOverviewCardRenderer,
  "workspace-band": NextHomeWorkspaceCardRenderer,
  "attention-band": NextHomeAttentionCardRenderer,
};

export function mergeNextHomeCardRendererRegistries(
  ...registries: Array<NextHomeCardRendererRegistry | null | undefined>
): NextHomeCardRendererRegistry {
  return registries.reduce<NextHomeCardRendererRegistry>((merged, registry) => {
    if (!registry) {
      return merged;
    }

    return {
      ...merged,
      ...registry,
    };
  }, {});
}

export function resolveNextHomeCardRenderer(
  kind: NextHomeCardKind,
  registry: NextHomeCardRendererRegistry,
): NextHomeResolvedCardRenderer {
  const registeredComponent = registry[kind];
  if (registeredComponent) {
    return {
      component: registeredComponent,
      status: "registered",
    };
  }

  if (isBuiltinNextHomeCardKind(kind)) {
    return {
      component: NextHomeMissingCardRenderer,
      status: "missing",
    };
  }

  return {
    component: NextHomePluginCardRenderer,
    status: "plugin-pending",
  };
}
