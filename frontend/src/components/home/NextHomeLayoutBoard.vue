<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, shallowRef, watch } from "vue";
import {
  HOME_GRID_COLUMNS,
  HOME_GRID_GAP,
  HOME_GRID_ROW_HEIGHT,
} from "@src/pages/home/layoutContract";
import type {
  NextHomeCardInstance,
  NextHomeCardKind,
  NextHomeCardRegistry,
} from "@src/pages/home/layoutContract";
import NextHomeEditableCard from "./NextHomeEditableCard.vue";

interface ResizeStartPayload {
  instanceId: string;
  axis: "horizontal" | "vertical" | "both";
  event: PointerEvent;
}

interface MoveStartPayload {
  instanceId: string;
  event: PointerEvent;
}

interface BoardDropPayload {
  kind: NextHomeCardKind;
  colStart: number;
  rowStart: number;
}

interface DragPreview {
  instanceId: string;
  colStart: number;
  rowStart: number;
  colSpan: number;
  rowSpan: number;
}

const props = defineProps<{
  instances: NextHomeCardInstance[];
  registry: NextHomeCardRegistry;
  selectedInstanceId: string | null;
  editMode: boolean;
  snapMode: boolean;
}>();

const emit = defineEmits<{
  selectInstance: [instanceId: string | null];
  moveInstance: [
    payload: { instanceId: string; x?: number; y?: number; colStart?: number; rowStart?: number },
  ];
  resizeInstance: [
    payload: {
      instanceId: string;
      width?: number;
      height?: number;
      colSpan?: number;
      rowSpan?: number;
    },
  ];
  autoHeightSync: [payload: { instanceId: string; rowSpan: number }];
  removeInstance: [instanceId: string];
  deployCard: [payload: BoardDropPayload];
}>();

const boardRef = shallowRef<HTMLElement | null>(null);
const compactMode = shallowRef(typeof window !== "undefined" ? window.innerWidth < 960 : false);
const dragPreview = shallowRef<DragPreview | null>(null);
const dragActiveId = shallowRef<string | null>(null);
const itemRefs = new Map<string, HTMLElement>();
const resizeObservers = new Map<string, ResizeObserver>();
let autoHeightSyncFrame: number | null = null;

const orderedInstances = computed(() =>
  [...props.instances].toSorted((left, right) => {
    if (left.zIndex !== right.zIndex) return left.zIndex - right.zIndex;
    return left.instanceId.localeCompare(right.instanceId);
  }),
);

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function syncCompactMode(): void {
  if (typeof window === "undefined") return;
  compactMode.value = window.innerWidth < 960;
}

function getBoardMetrics() {
  const element = boardRef.value;
  if (!element) return null;
  const rect = element.getBoundingClientRect();
  const cellWidth = (rect.width - HOME_GRID_GAP * (HOME_GRID_COLUMNS - 1)) / HOME_GRID_COLUMNS;
  return {
    rect,
    cellWidth,
    cellHeight: HOME_GRID_ROW_HEIGHT,
    toLeft: (x: number) => x * cellWidth + Math.max(0, x) * HOME_GRID_GAP,
    toTop: (y: number) => y * HOME_GRID_ROW_HEIGHT + Math.max(0, y) * HOME_GRID_GAP,
    toWidth: (width: number) => width * cellWidth + Math.max(0, width - 1) * HOME_GRID_GAP,
    toHeight: (height: number) =>
      height * HOME_GRID_ROW_HEIGHT + Math.max(0, height - 1) * HOME_GRID_GAP,
  };
}

function getCardStyle(instance: NextHomeCardInstance): Record<string, string> {
  if (compactMode.value) return { gridColumn: "1 / -1", gridRow: "auto" };
  const metrics = getBoardMetrics();
  if (!metrics) return {};
  const meta = props.registry[instance.kind];
  return {
    position: "absolute",
    left: `${metrics.toLeft(instance.x)}px`,
    top: `${metrics.toTop(instance.y)}px`,
    width: `${metrics.toWidth(instance.width)}px`,
    ...(meta?.autoHeight
      ? { minHeight: `${metrics.toHeight(instance.height)}px` }
      : { height: `${metrics.toHeight(instance.height)}px` }),
    zIndex: String(instance.zIndex),
    transform: "none",
  };
}

function measureAutoHeightRowSpan(instanceId: string): number | null {
  const element = itemRefs.get(instanceId);
  if (!element) return null;
  const body = element.querySelector<HTMLElement>(".next-home-editable-card__body");
  const chrome = element.querySelector<HTMLElement>(".next-home-editable-card__chrome");
  const source = body ?? element;
  const contentHeight = source.scrollHeight;
  if (!Number.isFinite(contentHeight) || contentHeight <= 0) return null;

  const style = window.getComputedStyle(element);
  const paddingTop = parseFloat(style.paddingTop) || 0;
  const paddingBottom = parseFloat(style.paddingBottom) || 0;
  const borderTop = parseFloat(style.borderTopWidth) || 0;
  const borderBottom = parseFloat(style.borderBottomWidth) || 0;
  const chromeBottom = chrome ? chrome.offsetTop + chrome.offsetHeight + 12 : 0;
  const totalHeight = Math.max(
    contentHeight + paddingTop + paddingBottom + borderTop + borderBottom,
    chromeBottom + paddingBottom + borderBottom,
  );
  return Math.max(
    1,
    Math.ceil((totalHeight + HOME_GRID_GAP) / (HOME_GRID_ROW_HEIGHT + HOME_GRID_GAP)),
  );
}

function syncAutoHeightRows(): void {
  if (compactMode.value || dragActiveId.value) return;

  for (const instance of props.instances) {
    const meta = props.registry[instance.kind];
    if (!meta?.autoHeight) continue;
    const measuredRowSpan = measureAutoHeightRowSpan(instance.instanceId);
    if (!measuredRowSpan || measuredRowSpan === instance.rowSpan) continue;
    emit("autoHeightSync", { instanceId: instance.instanceId, rowSpan: measuredRowSpan });
  }
}

function scheduleAutoHeightSync(): void {
  if (typeof window === "undefined") return;
  if (autoHeightSyncFrame !== null) {
    window.cancelAnimationFrame(autoHeightSyncFrame);
  }
  autoHeightSyncFrame = window.requestAnimationFrame(() => {
    autoHeightSyncFrame = null;
    syncAutoHeightRows();
  });
}

function cleanupItemObserver(instanceId: string): void {
  const observer = resizeObservers.get(instanceId);
  if (!observer) return;
  observer.disconnect();
  resizeObservers.delete(instanceId);
}

function setItemRef(instanceId: string, element: unknown): void {
  cleanupItemObserver(instanceId);
  const resolvedElement =
    element instanceof HTMLElement
      ? element
      : typeof element === "object" && element !== null && "$el" in element
        ? (element as { $el?: unknown }).$el
        : null;

  if (!(resolvedElement instanceof HTMLElement)) {
    itemRefs.delete(instanceId);
    return;
  }

  itemRefs.set(instanceId, resolvedElement);
  if (typeof ResizeObserver === "undefined") return;
  const observer = new ResizeObserver(() => {
    scheduleAutoHeightSync();
  });
  observer.observe(resolvedElement);
  resizeObservers.set(instanceId, observer);
  scheduleAutoHeightSync();
}

function getPreviewStyle(): Record<string, string> | null {
  if (!dragPreview.value || compactMode.value) return null;
  const metrics = getBoardMetrics();
  if (!metrics) return null;
  return {
    position: "absolute",
    left: `${metrics.toLeft(dragPreview.value.colStart - 1)}px`,
    top: `${metrics.toTop(dragPreview.value.rowStart - 1)}px`,
    width: `${metrics.toWidth(dragPreview.value.colSpan)}px`,
    height: `${metrics.toHeight(dragPreview.value.rowSpan)}px`,
  };
}

function clampPreviewPlacement(
  colStart: number,
  rowStart: number,
  colSpan: number,
  rowSpan: number,
): DragPreview {
  const maxColStart = Math.max(1, HOME_GRID_COLUMNS - colSpan + 1);
  return {
    instanceId: dragActiveId.value ?? "preview",
    colStart: clamp(colStart, 1, maxColStart),
    rowStart: clamp(rowStart, 1, Number.MAX_SAFE_INTEGER),
    colSpan,
    rowSpan,
  };
}

const boardHeight = computed(() => {
  if (compactMode.value) return "auto";
  const metrics = getBoardMetrics();
  if (!metrics) return "auto";
  const maxBottom = props.instances.reduce(
    (maxValue, instance) => Math.max(maxValue, instance.y + instance.height),
    0,
  );
  return `${Math.max(metrics.toTop(maxBottom), 320)}px`;
});

function startMove(payload: MoveStartPayload): void {
  if (!props.editMode || compactMode.value) return;
  const source = props.instances.find((instance) => instance.instanceId === payload.instanceId);
  const metrics = getBoardMetrics();
  if (!source || !metrics) return;

  const originX = payload.event.clientX;
  const originY = payload.event.clientY;
  dragActiveId.value = payload.instanceId;
  dragPreview.value = {
    instanceId: payload.instanceId,
    colStart: source.colStart,
    rowStart: source.rowStart,
    colSpan: source.colSpan,
    rowSpan: source.rowSpan,
  };

  const onMove = (moveEvent: PointerEvent) => {
    const deltaX = moveEvent.clientX - originX;
    const deltaY = moveEvent.clientY - originY;
    const nextX = source.x + deltaX / (metrics.cellWidth + HOME_GRID_GAP);
    const nextY = source.y + deltaY / (metrics.cellHeight + HOME_GRID_GAP);
    dragPreview.value = clampPreviewPlacement(
      Math.floor(nextX) + 1,
      Math.floor(nextY) + 1,
      source.colSpan,
      source.rowSpan,
    );
    dragPreview.value.instanceId = payload.instanceId;
    if (!props.snapMode) {
      emit("moveInstance", {
        instanceId: payload.instanceId,
        x: nextX,
        y: nextY,
      });
    }
  };

  const onUp = () => {
    window.removeEventListener("pointermove", onMove);
    if (dragPreview.value && props.snapMode) {
      emit("moveInstance", {
        instanceId: payload.instanceId,
        colStart: dragPreview.value.colStart,
        rowStart: dragPreview.value.rowStart,
      });
    }
    dragPreview.value = null;
    dragActiveId.value = null;
    document.body.style.removeProperty("user-select");
  };

  document.body.style.setProperty("user-select", "none");
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp, { once: true });
}

function startResize(payload: ResizeStartPayload): void {
  if (!props.editMode || compactMode.value) return;
  const source = props.instances.find((instance) => instance.instanceId === payload.instanceId);
  const metrics = getBoardMetrics();
  if (!source || !metrics) return;

  const originX = payload.event.clientX;
  const originY = payload.event.clientY;
  const initialColSpan = source.colSpan;
  const initialRowSpan = source.rowSpan;
  dragActiveId.value = payload.instanceId;
  dragPreview.value = {
    instanceId: payload.instanceId,
    colStart: source.colStart,
    rowStart: source.rowStart,
    colSpan: source.colSpan,
    rowSpan: source.rowSpan,
  };

  const onMove = (moveEvent: PointerEvent) => {
    const deltaWidth = (moveEvent.clientX - originX) / (metrics.cellWidth + HOME_GRID_GAP);
    const deltaHeight = (moveEvent.clientY - originY) / (metrics.cellHeight + HOME_GRID_GAP);
    dragPreview.value = clampPreviewPlacement(
      source.colStart,
      source.rowStart,
      payload.axis === "vertical"
        ? initialColSpan
        : Math.max(1, Math.ceil(source.width + deltaWidth)),
      payload.axis === "horizontal"
        ? initialRowSpan
        : Math.max(1, Math.ceil(source.height + deltaHeight)),
    );
    dragPreview.value.instanceId = payload.instanceId;
    if (!props.snapMode) {
      emit("resizeInstance", {
        instanceId: payload.instanceId,
        width: payload.axis === "vertical" ? source.width : Math.max(1, source.width + deltaWidth),
        height:
          payload.axis === "horizontal" ? source.height : Math.max(1, source.height + deltaHeight),
      });
    }
  };

  const onUp = () => {
    window.removeEventListener("pointermove", onMove);
    if (dragPreview.value && props.snapMode) {
      emit("resizeInstance", {
        instanceId: payload.instanceId,
        colSpan: dragPreview.value.colSpan,
        rowSpan: dragPreview.value.rowSpan,
      });
    }
    dragPreview.value = null;
    dragActiveId.value = null;
    document.body.style.removeProperty("user-select");
  };

  document.body.style.setProperty("user-select", "none");
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp, { once: true });
}

function resolveDropPlacement(event: DragEvent): BoardDropPayload | null {
  const kind = event.dataTransfer?.getData("application/x-sealantern-home-card") as
    | NextHomeCardKind
    | "";
  if (!kind) return null;
  const metrics = getBoardMetrics();
  if (!metrics) return null;
  const relativeX = clamp(event.clientX - metrics.rect.left, 0, metrics.rect.width);
  const relativeY = Math.max(event.clientY - metrics.rect.top, 0);
  return {
    kind,
    colStart: Math.floor(relativeX / (metrics.cellWidth + HOME_GRID_GAP)) + 1,
    rowStart: Math.floor(relativeY / (metrics.cellHeight + HOME_GRID_GAP)) + 1,
  };
}

function handleDrop(event: DragEvent): void {
  if (!props.editMode) return;
  event.preventDefault();
  const placement = resolveDropPlacement(event);
  if (!placement) return;
  emit("deployCard", placement);
}

onMounted(() => {
  syncCompactMode();
  if (typeof window !== "undefined") window.addEventListener("resize", syncCompactMode);
  void nextTick(() => {
    scheduleAutoHeightSync();
  });
});

onUnmounted(() => {
  if (typeof window !== "undefined") {
    window.removeEventListener("resize", syncCompactMode);
    if (autoHeightSyncFrame !== null) {
      window.cancelAnimationFrame(autoHeightSyncFrame);
      autoHeightSyncFrame = null;
    }
  }
  for (const instanceId of resizeObservers.keys()) {
    cleanupItemObserver(instanceId);
  }
  itemRefs.clear();
});

watch(
  () => props.instances,
  () => {
    void nextTick(() => {
      scheduleAutoHeightSync();
    });
  },
  { deep: true },
);

watch(
  () => props.registry,
  () => {
    void nextTick(() => {
      scheduleAutoHeightSync();
    });
  },
  { deep: true },
);
</script>

<template>
  <div
    class="next-home-layout-board-shell"
    :class="{ 'next-home-layout-board-shell--edit-mode': editMode }"
  >
    <div
      ref="boardRef"
      class="next-home-layout-board"
      :class="{ 'next-home-layout-board--compact': compactMode }"
      :style="{ height: boardHeight }"
      @click="emit('selectInstance', null)"
      @dragover.prevent
      @drop="handleDrop"
    >
      <div
        v-for="instance in orderedInstances"
        :key="instance.instanceId"
        :ref="(element) => setItemRef(instance.instanceId, element)"
        class="next-home-layout-board__item"
        :class="{
          'next-home-layout-board__item--dragging':
            dragActiveId === instance.instanceId && !snapMode,
        }"
        :style="getCardStyle(instance)"
      >
        <NextHomeEditableCard
          :instance="instance"
          :meta="registry[instance.kind]"
          :edit-mode="editMode"
          :selected="selectedInstanceId === instance.instanceId"
          :compact-mode="compactMode"
          :drag-active="dragActiveId === instance.instanceId"
          @select="emit('selectInstance', $event)"
          @remove="emit('removeInstance', $event)"
          @move-start="startMove"
          @resize-start="startResize"
          @click.stop
        >
          <slot name="card" :instance="instance" :meta="registry[instance.kind]" />
        </NextHomeEditableCard>
      </div>

      <div
        v-if="editMode && snapMode && dragPreview && getPreviewStyle()"
        class="next-home-layout-board__preview"
        :style="getPreviewStyle()!"
      />
    </div>
  </div>
</template>

<style scoped>
.next-home-layout-board-shell {
  min-width: 0;
}

.next-home-layout-board {
  min-width: 0;
  position: relative;
  min-height: 320px;
}

.next-home-layout-board--compact {
  display: grid;
  gap: 16px;
}

.next-home-layout-board__item {
  min-width: 0;
  transition: transform 90ms linear;
}

.next-home-layout-board__item--dragging {
  pointer-events: none;
}

.next-home-layout-board-shell--edit-mode .next-home-layout-board {
  padding: 4px;
  border-radius: 28px;
}

.next-home-layout-board__preview {
  min-width: 0;
  border-radius: 24px;
  border: 2px dashed color-mix(in srgb, var(--sl-primary) 62%, white);
  background: color-mix(in srgb, var(--sl-primary) 8%, transparent);
  pointer-events: none;
  z-index: 999;
}
</style>
