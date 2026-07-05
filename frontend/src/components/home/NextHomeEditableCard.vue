<script setup lang="ts">
import { GripVertical, Trash2 } from "@lucide/vue";
import { i18n } from "@language";
import type { NextHomeCardInstance, NextHomeCardLayoutMeta } from "@src/pages/home/layoutContract";

interface ResizeHandlePayload {
  instanceId: string;
  axis: "horizontal" | "vertical" | "both";
  event: PointerEvent;
}

interface MoveHandlePayload {
  instanceId: string;
  event: PointerEvent;
}

const props = defineProps<{
  instance: NextHomeCardInstance;
  meta: NextHomeCardLayoutMeta;
  editMode: boolean;
  selected: boolean;
  compactMode: boolean;
  dragActive: boolean;
}>();

const emit = defineEmits<{
  select: [instanceId: string];
  remove: [instanceId: string];
  moveStart: [payload: MoveHandlePayload];
  resizeStart: [payload: ResizeHandlePayload];
}>();

const autoHeight = props.meta.autoHeight === true;

function handleSelect(): void {
  emit("select", props.instance.instanceId);
}

function handleMoveStart(event: PointerEvent): void {
  if (!props.editMode || props.compactMode || !props.meta.movable) return;
  emit("select", props.instance.instanceId);
  emit("moveStart", { instanceId: props.instance.instanceId, event });
}

function handleResizeStart(axis: ResizeHandlePayload["axis"], event: PointerEvent): void {
  if (!props.editMode || props.compactMode || !props.meta.resizable) return;
  emit("select", props.instance.instanceId);
  emit("resizeStart", { instanceId: props.instance.instanceId, axis, event });
}
</script>

<template>
  <article
    class="next-home-editable-card"
    :class="{
      'next-home-editable-card--edit-mode': editMode,
      'next-home-editable-card--selected': selected,
      'next-home-editable-card--compact': compactMode,
      'next-home-editable-card--drag-active': dragActive,
      'next-home-editable-card--auto-height': autoHeight && !compactMode,
    }"
    @pointerdown="handleSelect"
  >
    <button
      v-if="editMode && meta.movable && !compactMode"
      class="next-home-editable-card__move-zone"
      type="button"
      :title="i18n.t('shell.home_edit_move')"
      @pointerdown.stop.prevent="handleMoveStart"
    />

    <div v-if="editMode" class="next-home-editable-card__chrome">
      <button
        v-if="meta.movable && !compactMode"
        class="next-home-editable-card__tool"
        type="button"
        :title="i18n.t('shell.home_edit_move')"
        @pointerdown.stop.prevent="handleMoveStart"
      >
        <GripVertical :size="14" />
      </button>

      <button
        v-if="meta.removable"
        class="next-home-editable-card__tool next-home-editable-card__tool--danger"
        type="button"
        :title="i18n.t('shell.home_edit_delete')"
        @click.stop="emit('remove', instance.instanceId)"
      >
        <Trash2 :size="14" />
      </button>
    </div>

    <div class="next-home-editable-card__body">
      <slot />
    </div>

    <template v-if="editMode && meta.resizable && !compactMode">
      <button
        class="next-home-editable-card__resize-handle next-home-editable-card__resize-handle--east"
        type="button"
        :title="i18n.t('shell.home_edit_resize_width')"
        @pointerdown.stop.prevent="handleResizeStart('horizontal', $event)"
      />
      <button
        class="next-home-editable-card__resize-handle next-home-editable-card__resize-handle--south"
        type="button"
        :title="i18n.t('shell.home_edit_resize_height')"
        :disabled="autoHeight"
        @pointerdown.stop.prevent="handleResizeStart('vertical', $event)"
      />
      <button
        class="next-home-editable-card__resize-handle next-home-editable-card__resize-handle--corner"
        type="button"
        :title="i18n.t('shell.home_edit_resize_both')"
        :disabled="autoHeight"
        @pointerdown.stop.prevent="handleResizeStart('both', $event)"
      />
    </template>
  </article>
</template>

<style scoped>
.next-home-editable-card {
  position: relative;
  min-width: 0;
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  border-radius: 26px;
  overflow: hidden;
}

.next-home-editable-card--edit-mode {
  padding: 8px;
  border: 1px dashed color-mix(in srgb, var(--sl-border) 82%, transparent);
  background: color-mix(in srgb, var(--sl-surface) 22%, transparent);
}

.next-home-editable-card--selected {
  border-color: color-mix(in srgb, var(--sl-primary) 58%, white);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--sl-primary) 26%, transparent);
}

.next-home-editable-card--drag-active {
  box-shadow: 0 18px 40px rgba(15, 23, 42, 0.14);
}

.next-home-editable-card__move-zone {
  position: absolute;
  top: 8px;
  left: 8px;
  right: 8px;
  height: 15%;
  min-height: 48px;
  z-index: 2;
  border: 0;
  border-radius: 18px 18px 12px 12px;
  background: transparent;
  cursor: grab;
}

.next-home-editable-card__move-zone:active {
  cursor: grabbing;
}

.next-home-editable-card__chrome {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 3;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.next-home-editable-card__tool,
.next-home-editable-card__resize-handle {
  border: 0;
  color: var(--sl-text-primary);
  background: color-mix(in srgb, var(--sl-surface) 94%, transparent);
}

.next-home-editable-card__tool {
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  box-shadow: var(--sl-shadow-sm);
  cursor: pointer;
}

.next-home-editable-card__tool--danger {
  color: color-mix(in srgb, var(--sl-danger) 88%, white);
}

.next-home-editable-card__body {
  min-width: 0;
  height: 100%;
  box-sizing: border-box;
  overflow: hidden;
}

.next-home-editable-card--auto-height {
  height: auto;
}

.next-home-editable-card--auto-height .next-home-editable-card__body {
  height: auto;
  overflow: visible;
}

.next-home-editable-card__resize-handle {
  position: absolute;
  z-index: 2;
  opacity: 0.92;
  cursor: pointer;
}

.next-home-editable-card__resize-handle:disabled {
  opacity: 0.32;
  cursor: not-allowed;
}

.next-home-editable-card__resize-handle--east {
  top: 50%;
  right: 4px;
  width: 10px;
  height: 72px;
  border-radius: 999px;
  transform: translateY(-50%);
  cursor: ew-resize;
}

.next-home-editable-card__resize-handle--south {
  bottom: 4px;
  left: 50%;
  width: 72px;
  height: 10px;
  border-radius: 999px;
  transform: translateX(-50%);
  cursor: ns-resize;
}

.next-home-editable-card__resize-handle--corner {
  right: 6px;
  bottom: 6px;
  width: 18px;
  height: 18px;
  border-radius: 8px;
  cursor: nwse-resize;
}
</style>
