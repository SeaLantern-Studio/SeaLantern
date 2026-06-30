<script setup lang="ts">
import { computed, toRef } from "vue";
import type { NextHomeCardKind } from "./layoutContract";
import NextHomeCardPalette from "../../components/home/NextHomeCardPalette.vue";
import NextHomeCardRendererHost from "../../components/home/NextHomeCardRendererHost.vue";
import NextHomeLayoutBoard from "../../components/home/NextHomeLayoutBoard.vue";
import { NEXT_HOME_BUILTIN_CARD_RENDERERS } from "./builtinCardRenderers";
import type { NextHomeCardRendererRegistry, NextHomeCardRuntimeContext } from "./cardRendererContract";
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

const homePage = useNextHomePage({
  previewMode: props.previewMode,
  sidebarEntryCount: props.sidebarEntryCount,
});

const layoutEditor = useNextHomeLayoutEditor({
  editMode: toRef(props, "editMode"),
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

const cardRenderers = computed<NextHomeCardRendererRegistry>(() => ({
  ...NEXT_HOME_BUILTIN_CARD_RENDERERS,
  ...(props.additionalCardRenderers ?? {}),
}));

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

function handleBoardDeploy(payload: { kind: NextHomeCardKind; colStart: number; rowStart: number }): void {
  layoutEditor.deployCard(payload.kind, {
    colStart: payload.colStart,
    rowStart: payload.rowStart,
  });
}
</script>

<template>
  <div class="next-home-page">
    <NextHomeLayoutBoard
      class="next-home-page__board"
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
</style>
