<script setup lang="ts">
import { computed, toRef } from "vue";
import { useNextHostRuntime } from "@src/host/runtime";
import type { NextHomeCardKind } from "./layoutContract";
import NextHomeCardPalette from "@src/components/home/NextHomeCardPalette.vue";
import NextHomeCardRendererHost from "@src/components/home/NextHomeCardRendererHost.vue";
import NextHomeLayoutBoard from "@src/components/home/NextHomeLayoutBoard.vue";
import {
  createNextHomeHostCardRegistry,
  mergeNextHomeCardDefinitionRegistries,
  mergeNextHomeCardRendererRegistries,
  NEXT_HOME_BUILTIN_CARD_RENDERERS,
} from "./cardRendererRegistry";
import type {
  NextHomeCardDefinitionRegistry,
  NextHomeCardRendererRegistry,
  NextHomeCardRuntimeContext,
} from "./cardRendererContract";
import { useNextHomeLayoutEditor } from "./useNextHomeLayoutEditor";
import { useNextHomePage } from "./useNextHomePage";

interface Props {
  instanceHost: string;
  previewMode: boolean;
  sidebarEntryCount: number;
  editMode: boolean;
  paletteAnchorRect: { top: number; left: number; width: number; height: number } | null;
  additionalCardRenderers?: NextHomeCardRendererRegistry;
}

const props = defineProps<Props>();
const nextHostRuntime = useNextHostRuntime();

const homePage = useNextHomePage({
  previewMode: props.previewMode,
  sidebarEntryCount: props.sidebarEntryCount,
});

const hostCardDefinitions = computed<NextHomeCardDefinitionRegistry>(() =>
  mergeNextHomeCardDefinitionRegistries(nextHostRuntime.home.cardDefinitions.value),
);

const hostCardLayouts = computed(() =>
  Object.values(createNextHomeHostCardRegistry(hostCardDefinitions.value)),
);

const layoutEditor = useNextHomeLayoutEditor({
  editMode: toRef(props, "editMode"),
  additionalRegistry: hostCardLayouts,
});

const {
  registry,
  instances,
  paletteEntries,
  selectedInstanceId,
  snapMode,
  selectInstance,
  deployCardToViewportCenter,
  removeInstance,
  moveInstance,
  resizeInstance,
  resetLayout,
} = layoutEditor;

const {
  isInitialLoading,
  summaryMetrics,
  systemMetrics,
  statsViewMode,
  usingPreviewFallback,
  featuredServer,
  secondaryServers,
  alertItems,
  lastUpdatedLabel,
  refreshNow,
  toggleStatsViewMode,
  goToCreateServer,
  goToImportServer,
  toggleServer,
  totalServerCount,
  isRefreshing,
} = homePage;

const cardRenderers = computed<NextHomeCardRendererRegistry>(() =>
  mergeNextHomeCardRendererRegistries(
    NEXT_HOME_BUILTIN_CARD_RENDERERS,
    Object.fromEntries(
      Object.entries(hostCardDefinitions.value).map(([kind, entry]) => [kind, entry?.renderer]),
    ),
    nextHostRuntime.home.cardRenderers.value,
    props.additionalCardRenderers,
  ),
);

const cardRuntimeContext = computed<NextHomeCardRuntimeContext>(() => ({
  previewMode: props.previewMode,
  usingPreviewFallback: usingPreviewFallback.value,
  summaryMetrics: summaryMetrics.value,
  systemMetrics: systemMetrics.value,
  statsViewMode: statsViewMode.value,
  featuredServer: featuredServer.value,
  secondaryServers: secondaryServers.value,
  alertItems: alertItems.value,
  lastUpdatedLabel: lastUpdatedLabel.value,
  totalServerCount: totalServerCount.value,
  isRefreshing: isRefreshing.value,
  refreshNow,
  toggleStatsViewMode,
  goToCreateServer,
  goToImportServer,
  toggleServer,
}));

function handleBoardDeploy(payload: {
  kind: NextHomeCardKind;
  colStart: number;
  rowStart: number;
}): void {
  layoutEditor.deployCard(payload.kind, {
    colStart: payload.colStart,
    rowStart: payload.rowStart,
  });
}
</script>

<template>
  <div class="next-home-page">
    <div class="next-home-page__board-shell" :class="{ 'is-loading': isInitialLoading }">
      <NextHomeLayoutBoard
        class="next-home-page__board"
        :aria-busy="isInitialLoading ? 'true' : 'false'"
        :instances="instances"
        :registry="registry"
        :selected-instance-id="selectedInstanceId"
        :edit-mode="editMode"
        :snap-mode="snapMode"
        @select-instance="selectInstance"
        @move-instance="moveInstance($event.instanceId, $event)"
        @resize-instance="resizeInstance($event.instanceId, $event)"
        @remove-instance="removeInstance"
        @deploy-card="handleBoardDeploy"
      >
        <template #card="{ instance, meta }">
          <NextHomeCardRendererHost
            :instance="instance"
            :meta="meta"
            :context="cardRuntimeContext"
            :registry="cardRenderers"
          />
        </template>
      </NextHomeLayoutBoard>

      <div v-if="isInitialLoading" class="next-home-page__loading-mask" aria-hidden="true">
        <div class="next-home-page__skeleton next-home-page__skeleton--summary">
          <span v-for="index in 4" :key="`summary-${index}`" />
        </div>

        <div class="next-home-page__skeleton-row">
          <div class="next-home-page__skeleton next-home-page__skeleton--operations" />
          <div class="next-home-page__skeleton next-home-page__skeleton--system" />
        </div>

        <div class="next-home-page__skeleton next-home-page__skeleton--workspace" />
        <div class="next-home-page__skeleton next-home-page__skeleton--attention" />
      </div>
    </div>

    <NextHomeCardPalette
      v-if="editMode"
      class="next-home-page__palette"
      :entries="paletteEntries"
      :anchor-rect="paletteAnchorRect"
      v-model:snap-mode="snapMode"
      @deploy="deployCardToViewportCenter"
      @reset="resetLayout"
    />
  </div>
</template>

<style scoped>
.next-home-page {
  min-width: 0;
  display: grid;
  gap: 20px;
  align-items: start;
}

.next-home-page__board,
.next-home-page__palette {
  min-width: 0;
}

.next-home-page__board-shell {
  position: relative;
  min-width: 0;
}

.next-home-page__board-shell.is-loading > .next-home-page__board {
  visibility: hidden;
}

.next-home-page__loading-mask {
  position: absolute;
  inset: 0;
  display: grid;
  gap: 20px;
  pointer-events: none;
}

.next-home-page__skeleton,
.next-home-page__skeleton--summary > span {
  border-radius: 20px;
  background:
    linear-gradient(
      90deg,
      color-mix(in srgb, var(--sl-surface) 92%, transparent) 0%,
      color-mix(in srgb, var(--sl-bg-secondary) 78%, white) 50%,
      color-mix(in srgb, var(--sl-surface) 92%, transparent) 100%
    );
  background-size: 200% 100%;
  animation: next-home-page-skeleton 1.2s ease-in-out infinite;
}

.next-home-page__skeleton-row {
  display: grid;
  gap: 20px;
  grid-template-columns: minmax(0, 1.1fr) minmax(280px, 0.9fr);
}

.next-home-page__skeleton--summary {
  display: grid;
  gap: 0;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
}

.next-home-page__skeleton--summary > span {
  min-height: 120px;
  border-right: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
}

.next-home-page__skeleton--summary > span:last-child {
  border-right: 0;
}

.next-home-page__skeleton--operations,
.next-home-page__skeleton--system {
  min-height: 252px;
}

.next-home-page__skeleton--workspace {
  min-height: 420px;
}

.next-home-page__skeleton--attention {
  min-height: 236px;
}

@keyframes next-home-page-skeleton {
  0% {
    background-position: 100% 50%;
  }

  100% {
    background-position: 0 50%;
  }
}

@media (max-width: 1023px) {
  .next-home-page__skeleton-row {
    grid-template-columns: 1fr;
  }

  .next-home-page__skeleton--summary {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .next-home-page__skeleton--summary > span:nth-child(2n) {
    border-right: 0;
  }
}

@media (max-width: 639px) {
  .next-home-page__skeleton--summary {
    grid-template-columns: 1fr;
  }

  .next-home-page__skeleton--summary > span {
    border-right: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--sl-border) 72%, transparent);
  }

  .next-home-page__skeleton--summary > span:last-child {
    border-bottom: 0;
  }
}
</style>
