import NextHomeAttentionCardRenderer from "@src/components/home/renderers/NextHomeAttentionCardRenderer.vue";
import NextHomeMissingCardRenderer from "@src/components/home/renderers/NextHomeMissingCardRenderer.vue";
import NextHomeOperationsCardRenderer from "@src/components/home/renderers/NextHomeOperationsCardRenderer.vue";
import NextHomePluginCardRenderer from "@src/components/home/renderers/NextHomePluginCardRenderer.vue";
import NextHomeSummaryCardRenderer from "@src/components/home/renderers/NextHomeSummaryCardRenderer.vue";
import NextHomeSystemOverviewCardRenderer from "@src/components/home/renderers/NextHomeSystemOverviewCardRenderer.vue";
import NextHomeWorkspaceCardRenderer from "@src/components/home/renderers/NextHomeWorkspaceCardRenderer.vue";
import {
  createNextHomeHostCardLayoutMeta,
  isBuiltinNextHomeCardKind,
  isNextHomePluginCardKind,
  type NextHomeCardKind,
  type NextHomeCardRegistry,
} from "./layoutContract";
import type {
  NextHomeCardDefinitionRegistry,
  NextHomeCardRendererRegistry,
  NextHomeResolvedCardRenderer,
} from "./cardRendererContract";

export const NEXT_HOME_BUILTIN_CARD_RENDERERS: NextHomeCardRendererRegistry = {
  "summary-band": NextHomeSummaryCardRenderer,
  "operations-band": NextHomeOperationsCardRenderer,
  "system-overview": NextHomeSystemOverviewCardRenderer,
  "workspace-band": NextHomeWorkspaceCardRenderer,
  "attention-band": NextHomeAttentionCardRenderer,
};

export function mergeNextHomeCardDefinitionRegistries(
  ...registries: Array<NextHomeCardDefinitionRegistry | null | undefined>
): NextHomeCardDefinitionRegistry {
  const merged: NextHomeCardDefinitionRegistry = {};

  for (const registry of registries) {
    if (!registry) {
      continue;
    }

    for (const [rawKind, entry] of Object.entries(registry)) {
      if (!entry || !isNextHomePluginCardKind(rawKind)) {
        continue;
      }

      merged[rawKind] = entry;
    }
  }

  return merged;
}

export function mergeNextHomeCardRendererRegistries(
  ...registries: Array<NextHomeCardRendererRegistry | null | undefined>
): NextHomeCardRendererRegistry {
  const merged: NextHomeCardRendererRegistry = {};

  for (const registry of registries) {
    if (!registry) {
      continue;
    }

    for (const [rawKind, component] of Object.entries(registry)) {
      if (!component) {
        continue;
      }

      const kind = rawKind as NextHomeCardKind;
      if (isBuiltinNextHomeCardKind(kind) && kind in merged) {
        continue;
      }

      merged[kind] = component;
    }
  }

  return merged;
}

export function createNextHomeHostCardRegistry(
  definitions: NextHomeCardDefinitionRegistry,
): NextHomeCardRegistry {
  const entries = Object.values(definitions)
    .map((entry) => entry?.definition)
    .filter((entry): entry is NonNullable<typeof entry> => Boolean(entry))
    .map((definition) => [definition.kind, createNextHomeHostCardLayoutMeta(definition)]);

  return Object.fromEntries(entries) as NextHomeCardRegistry;
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
