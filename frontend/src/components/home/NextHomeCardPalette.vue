<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { LayoutGrid, Magnet, Move, Plus, RotateCcw } from "@lucide/vue";
import { i18n } from "@language";
import type { NextHomeCardKind } from "@src/pages/home/layoutContract";
import { resolveNextHomeCardTitle } from "@src/pages/home/cardMeta";

interface PaletteEntry {
  kind: NextHomeCardKind;
  meta: {
    id: NextHomeCardKind;
    titleKey?: string | null;
    title?: string | null;
  };
  count: number;
  limit: number;
  canAdd: boolean;
}

const props = defineProps<{
  entries: PaletteEntry[];
  anchorRect: { top: number; left: number; width: number; height: number } | null;
}>();

const emit = defineEmits<{
  deploy: [kind: NextHomeCardKind];
  reset: [];
  updateSnapMode: [value: boolean];
}>();

const snapMode = defineModel<boolean>("snapMode", { default: false });
let closeTimer: ReturnType<typeof setTimeout> | null = null;

function handleDragStart(kind: NextHomeCardKind, event: DragEvent): void {
  if (!event.dataTransfer) return;
  event.dataTransfer.effectAllowed = "copy";
  event.dataTransfer.setData("application/x-sealantern-home-card", kind);
  event.dataTransfer.setData("text/plain", kind);
}

const isOpen = shallowRef(false);
const totalCountLabel = computed(() => `${props.entries.length}`);
const paletteStyle = computed(() => {
  const anchor = props.anchorRect;
  if (!anchor) return {};
  return {
    top: `${Math.max(anchor.top, 18)}px`,
    left: `${Math.max(anchor.left - anchor.width - 10, 18)}px`,
  };
});

function openPalette(): void {
  if (closeTimer) {
    clearTimeout(closeTimer);
    closeTimer = null;
  }
  isOpen.value = true;
}

function closePalette(): void {
  if (closeTimer) {
    clearTimeout(closeTimer);
  }
  closeTimer = setTimeout(() => {
    isOpen.value = false;
    closeTimer = null;
  }, 160);
}
</script>

<template>
  <aside
    class="next-home-card-palette"
    :class="{ 'next-home-card-palette--open': isOpen }"
    :style="paletteStyle"
    @mouseenter="openPalette"
    @mouseleave="closePalette"
  >
    <button
      class="next-home-card-palette__trigger"
      type="button"
      :aria-expanded="isOpen"
      :title="isOpen ? i18n.t('shell.home_palette_collapse') : i18n.t('shell.home_palette_expand')"
      @focus="openPalette"
    >
      <LayoutGrid :size="16" />
      <span class="next-home-card-palette__trigger-count">{{ totalCountLabel }}</span>
    </button>

    <div class="next-home-card-palette__panel">
      <header class="next-home-card-palette__header">
        <span class="next-home-card-palette__eyebrow">{{
          i18n.t("shell.home_palette_eyebrow")
        }}</span>
        <h2>{{ i18n.t("shell.home_palette_title") }}</h2>
        <button
          class="next-home-card-palette__reset"
          type="button"
          :title="i18n.t('shell.home_palette_reset')"
          @click="emit('reset')"
        >
          <RotateCcw :size="14" />
        </button>
      </header>

      <div class="next-home-card-palette__settings">
        <button
          class="next-home-card-palette__mode-toggle"
          type="button"
          :aria-pressed="snapMode"
          :title="
            snapMode
              ? i18n.t('shell.home_palette_snap_active')
              : i18n.t('shell.home_palette_free_active')
          "
          @click="
            snapMode = !snapMode;
            emit('updateSnapMode', snapMode);
          "
        >
          <Magnet v-if="snapMode" :size="14" />
          <Move v-else :size="14" />
          <span>{{
            snapMode
              ? i18n.t("shell.home_palette_snap_label")
              : i18n.t("shell.home_palette_free_label")
          }}</span>
        </button>
      </div>

      <div class="next-home-card-palette__list">
        <article
          v-for="entry in entries"
          :key="entry.kind"
          class="next-home-card-palette__item"
          :class="{ 'next-home-card-palette__item--disabled': !entry.canAdd }"
          :draggable="entry.canAdd"
          :title="
            entry.canAdd
              ? i18n.t('shell.home_palette_deploy_hint')
              : i18n.t('shell.home_palette_limit_reached')
          "
          @dblclick="emit('deploy', entry.kind)"
          @dragstart="handleDragStart(entry.kind, $event)"
        >
          <span class="next-home-card-palette__item-title">{{ resolveNextHomeCardTitle(entry.meta) }}</span>
          <span class="next-home-card-palette__item-count"
            >{{ entry.count }}/{{ entry.limit }}</span
          >
          <button
            class="next-home-card-palette__item-add"
            type="button"
            :disabled="!entry.canAdd"
            @click.stop="emit('deploy', entry.kind)"
          >
            <Plus :size="14" />
          </button>
        </article>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.next-home-card-palette {
  position: fixed;
  top: 18px;
  left: 18px;
  z-index: 2000;
  display: flex;
  align-items: flex-start;
}

.next-home-card-palette__trigger {
  width: 42px;
  height: 42px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  border: 1px solid color-mix(in srgb, var(--sl-border) 84%, transparent);
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-surface) 92%, transparent);
  color: var(--sl-text-primary);
  box-shadow: var(--sl-shadow-md);
  cursor: pointer;
}

.next-home-card-palette__trigger-count {
  position: absolute;
  right: -6px;
  bottom: -6px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-primary) 18%, white);
  color: var(--sl-primary);
  font-size: 11px;
  font-weight: 700;
}

.next-home-card-palette__panel {
  position: absolute;
  top: 0;
  right: calc(100% + 10px);
  width: 0;
  max-height: 0;
  overflow: hidden;
  opacity: 0;
  pointer-events: none;
  margin-left: 0;
  padding: 0;
  border-radius: 24px;
  border: 1px solid transparent;
  background: color-mix(in srgb, var(--sl-surface) 94%, transparent);
  box-shadow: none;
  transition:
    width var(--sl-transition-normal),
    max-height var(--sl-transition-normal),
    opacity var(--sl-transition-fast),
    padding var(--sl-transition-normal),
    border-color var(--sl-transition-fast),
    box-shadow var(--sl-transition-fast);
}

.next-home-card-palette--open .next-home-card-palette__panel {
  width: 300px;
  max-height: 70vh;
  overflow: auto;
  opacity: 1;
  pointer-events: auto;
  padding: 18px;
  border-color: color-mix(in srgb, var(--sl-border) 84%, transparent);
  box-shadow: var(--sl-shadow-md);
}

.next-home-card-palette__header {
  position: relative;
  display: grid;
  gap: 6px;
}

.next-home-card-palette__reset {
  position: absolute;
  top: 0;
  right: 0;
  width: 30px;
  height: 30px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid color-mix(in srgb, var(--sl-border) 82%, transparent);
  border-radius: 10px;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.next-home-card-palette__eyebrow {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.next-home-card-palette__header h2 {
  margin: 0;
  color: var(--sl-text-primary);
  font-size: 1rem;
}

.next-home-card-palette__settings {
  margin: 14px 0 12px;
}

.next-home-card-palette__mode-toggle {
  width: 100%;
  min-height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 82%, transparent);
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-surface) 90%, transparent);
  color: var(--sl-text-secondary);
  font: inherit;
  cursor: pointer;
}

.next-home-card-palette__list {
  display: grid;
  gap: 10px;
}

.next-home-card-palette__item {
  width: 100%;
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 82%, transparent);
  border-radius: 16px;
  background: color-mix(in srgb, var(--sl-bg-secondary) 74%, white);
  color: var(--sl-text-primary);
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.next-home-card-palette__item-title,
.next-home-card-palette__item-count {
  min-width: 0;
}

.next-home-card-palette__item-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.next-home-card-palette__item-count {
  color: var(--sl-text-tertiary);
  font-size: var(--sl-font-size-sm);
}

.next-home-card-palette__item-add {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 999px;
  background: color-mix(in srgb, var(--sl-primary) 10%, transparent);
  color: var(--sl-primary);
  cursor: pointer;
}

.next-home-card-palette__item--disabled {
  opacity: 0.48;
  cursor: not-allowed;
}

@media (max-width: 1023px) {
  .next-home-card-palette {
    position: fixed;
    top: 18px;
    left: 18px;
  }

  .next-home-card-palette--open .next-home-card-palette__panel {
    width: min(300px, calc(100vw - 120px));
  }
}
</style>
