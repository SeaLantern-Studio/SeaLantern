<script setup lang="ts">
import { Check, Languages } from "@lucide/vue";
import { computed, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import { i18n } from "@language";
import { useI18nStore } from "@stores/i18nStore";

interface Props {
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
});

const i18nStore = useI18nStore();
const containerRef = useTemplateRef<HTMLElement>("containerRef");
const triggerRef = useTemplateRef<HTMLButtonElement>("triggerRef");
const overlayRef = useTemplateRef<HTMLElement>("overlayRef");
const open = shallowRef(false);
const pendingLocale = shallowRef<string | null>(null);
const overlayPosition = shallowRef({ top: 0, left: 0 });

const localeEntries = computed(() =>
  i18nStore.localeOptions.map((option) => ({
    code: option.code,
    label: option.label,
    current: option.code === i18nStore.currentLocale,
  })),
);

function closePanel(): void {
  open.value = false;
}

function togglePanel(): void {
  if (props.disabled) {
    return;
  }

  open.value = !open.value;
}

function updateOverlayPosition(): void {
  const rect = triggerRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  overlayPosition.value = {
    top: rect.bottom + 10,
    left: Math.max(16, rect.right - 320),
  };
}

async function switchLocale(localeCode: string): Promise<void> {
  if (pendingLocale.value || localeCode === i18nStore.currentLocale) {
    closePanel();
    return;
  }

  pendingLocale.value = localeCode;
  try {
    const changed = await i18nStore.setLocale(localeCode);
    if (changed) {
      closePanel();
    }
  } finally {
    pendingLocale.value = null;
  }
}

function handleWindowPointerDown(event: PointerEvent): void {
  const target = event.target;
  if (!(target instanceof Node)) {
    return;
  }

  if (containerRef.value?.contains(target) || overlayRef.value?.contains(target)) {
    return;
  }

  closePanel();
}

function handleWindowKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    closePanel();
  }
}

watch(
  open,
  (value) => {
    if (value) {
      updateOverlayPosition();
      window.addEventListener("pointerdown", handleWindowPointerDown);
      window.addEventListener("keydown", handleWindowKeydown);
      window.addEventListener("resize", updateOverlayPosition);
      window.addEventListener("scroll", updateOverlayPosition, true);
      return;
    }

    window.removeEventListener("pointerdown", handleWindowPointerDown);
    window.removeEventListener("keydown", handleWindowKeydown);
    window.removeEventListener("resize", updateOverlayPosition);
    window.removeEventListener("scroll", updateOverlayPosition, true);
  },
  { immediate: true },
);

onUnmounted(() => {
  window.removeEventListener("pointerdown", handleWindowPointerDown);
  window.removeEventListener("keydown", handleWindowKeydown);
  window.removeEventListener("resize", updateOverlayPosition);
  window.removeEventListener("scroll", updateOverlayPosition, true);
});
</script>

<template>
  <div ref="containerRef" class="titlebar-locale-switcher">
    <button
      ref="triggerRef"
      class="titlebar-locale-switcher__trigger"
      type="button"
      :disabled="disabled"
      :aria-label="i18n.t('common.switch_language')"
      :title="i18n.t('common.switch_language')"
      :aria-expanded="open ? 'true' : 'false'"
      @click="togglePanel"
    >
      <Languages :size="15" />
    </button>

    <Teleport to="body">
      <div
        v-if="open"
        ref="overlayRef"
        class="titlebar-locale-switcher__overlay"
        :style="{ top: `${overlayPosition.top}px`, left: `${overlayPosition.left}px` }"
        role="dialog"
        aria-modal="false"
      >
        <div class="titlebar-locale-switcher__panel">
          <header class="titlebar-locale-switcher__header">
            <span class="titlebar-locale-switcher__eyebrow">{{ i18n.t("settings.language") }}</span>
            <strong>{{ i18n.t("common.switch_language") }}</strong>
          </header>

          <div
            class="titlebar-locale-switcher__list"
            role="listbox"
            :aria-label="i18n.t('settings.language')"
          >
            <button
              v-for="entry in localeEntries"
              :key="entry.code"
              class="titlebar-locale-switcher__item"
              :class="{ 'is-current': entry.current }"
              type="button"
              :disabled="pendingLocale !== null"
              :aria-selected="entry.current ? 'true' : 'false'"
              @click="switchLocale(entry.code)"
            >
              <span class="titlebar-locale-switcher__item-copy">
                <span class="titlebar-locale-switcher__item-label">{{ entry.label }}</span>
                <span class="titlebar-locale-switcher__item-code">{{ entry.code }}</span>
              </span>

              <Check v-if="entry.current" :size="15" class="titlebar-locale-switcher__check" />
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.titlebar-locale-switcher {
  position: relative;
  flex: 0 0 auto;
}

.titlebar-locale-switcher__trigger {
  width: 30px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
}

.titlebar-locale-switcher__trigger:disabled {
  opacity: 0.56;
  cursor: not-allowed;
}

.titlebar-locale-switcher__trigger:focus-visible,
.titlebar-locale-switcher__item:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sl-primary) 24%, transparent);
}

.titlebar-locale-switcher__overlay {
  position: fixed;
  z-index: 10000;
}

.titlebar-locale-switcher__panel {
  width: min(320px, calc(100vw - 32px));
  max-height: min(420px, calc(100vh - 32px));
  border: 1px solid color-mix(in srgb, var(--sl-primary) 18%, var(--sl-border));
  border-radius: 18px;
  background: color-mix(in srgb, var(--sl-bg) 92%, var(--sl-surface));
  box-shadow: var(--sl-shadow-lg);
  backdrop-filter: blur(18px);
  padding: 12px;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 10px;
  overflow: hidden;
}

.titlebar-locale-switcher__header {
  display: grid;
  gap: 4px;
}

.titlebar-locale-switcher__eyebrow {
  font-size: 0.78rem;
  color: var(--sl-text-secondary);
}

.titlebar-locale-switcher__list {
  min-height: 0;
  display: grid;
  gap: 8px;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding-right: 2px;
}

.titlebar-locale-switcher__item {
  width: 100%;
  min-width: 0;
  appearance: none;
  -webkit-appearance: none;
  padding: 10px 12px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 74%, transparent);
  border-radius: 12px;
  background: color-mix(in srgb, var(--sl-surface) 88%, transparent);
  color: var(--sl-text-primary);
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  text-align: left;
  cursor: pointer;
  font: inherit;
  line-height: 1.3;
}

.titlebar-locale-switcher__item.is-current {
  border-color: color-mix(in srgb, var(--sl-primary) 42%, var(--sl-border));
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
}

.titlebar-locale-switcher__item:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.titlebar-locale-switcher__item-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.titlebar-locale-switcher__item-label {
  min-width: 0;
  display: block;
  font-size: 0.88rem;
  font-weight: 600;
  white-space: normal;
  word-break: break-word;
}

.titlebar-locale-switcher__item-code,
.titlebar-locale-switcher__check {
  color: var(--sl-text-secondary);
}

.titlebar-locale-switcher__item-code {
  display: block;
  font-size: 0.72rem;
  white-space: nowrap;
}

@media (hover: hover) and (pointer: fine) {
  .titlebar-locale-switcher__trigger:not(:disabled):hover {
    color: var(--sl-primary);
  }

  .titlebar-locale-switcher__item:not(:disabled):hover {
    border-color: color-mix(in srgb, var(--sl-primary) 38%, var(--sl-border));
  }
}
</style>
