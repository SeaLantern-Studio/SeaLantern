<script setup lang="ts">
import { Copy, Minus, Square, X } from "@lucide/vue";
import { i18n } from "@language";
import TitlebarLocaleSwitcher from "./TitlebarLocaleSwitcher.vue";

interface Props {
  macos?: boolean;
  showControls?: boolean;
  isMaximized?: boolean;
  appDisplayName?: string;
}

defineProps<Props>();

const emit = defineEmits<{
  minimize: [];
  toggleMaximize: [];
  close: [];
}>();
</script>

<template>
  <div class="next-desktop-titlebar" :class="{ 'next-desktop-titlebar--macos': macos }">
    <div
      class="next-desktop-titlebar__drag-surface"
      data-tauri-drag-region
      aria-hidden="true"
    ></div>

    <div v-if="showControls" class="next-desktop-titlebar__trailing">
      <TitlebarLocaleSwitcher />

      <div
        v-if="appDisplayName"
        class="next-desktop-titlebar__app-name"
        data-tauri-drag-region
        :title="appDisplayName"
      >
        {{ appDisplayName }}
      </div>

      <div class="next-desktop-titlebar__controls">
        <button
          class="next-desktop-titlebar__button"
          type="button"
          :aria-label="i18n.t('common.minimize')"
          :title="i18n.t('common.minimize')"
          @click="emit('minimize')"
        >
          <Minus :size="14" />
        </button>

        <button
          class="next-desktop-titlebar__button"
          type="button"
          :aria-label="isMaximized ? i18n.t('common.restore') : i18n.t('common.maximize')"
          :title="isMaximized ? i18n.t('common.restore') : i18n.t('common.maximize')"
          @click="emit('toggleMaximize')"
        >
          <Copy v-if="isMaximized" :size="14" />
          <Square v-else :size="14" />
        </button>

        <button
          class="next-desktop-titlebar__button next-desktop-titlebar__button--close"
          type="button"
          :aria-label="i18n.t('common.close')"
          :title="i18n.t('common.close')"
          @click="emit('close')"
        >
          <X :size="14" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.next-desktop-titlebar {
  position: relative;
  z-index: 20;
  min-height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 28px 0 12px;
}

.next-desktop-titlebar--macos {
  padding-top: 18px;
}

.next-desktop-titlebar__drag-surface {
  flex: 1 1 auto;
  align-self: stretch;
  min-width: 0;
}

.next-desktop-titlebar__controls {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 2px;
}

.next-desktop-titlebar__trailing {
  min-width: 0;
  flex: 0 1 auto;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.next-desktop-titlebar__app-name {
  min-width: 0;
  max-width: min(320px, 32vw);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
  color: var(--sl-text-primary);
  font-size: 0.95rem;
  line-height: 1;
}

.next-desktop-titlebar__button {
  width: 32px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: var(--sl-radius-sm);
  background: transparent;
  color: var(--sl-text-secondary);
  cursor: pointer;
  transition: none;
}

.next-desktop-titlebar__button:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sl-primary) 24%, transparent);
}

.next-desktop-titlebar__button--close:focus-visible {
  box-shadow: 0 0 0 2px rgba(var(--sl-error), 0.18);
}

@media (hover: hover) and (pointer: fine) {
  .next-desktop-titlebar__button:hover {
    background: var(--sl-bg-tertiary);
    color: var(--sl-text-primary);
  }

  .next-desktop-titlebar__button--close:hover {
    background: var(--sl-error);
    color: var(--sl-text-inverse);
  }
}

@media (max-width: 767px) {
  .next-desktop-titlebar {
    padding-inline: 12px;
  }

  .next-desktop-titlebar__trailing {
    gap: 8px;
  }

  .next-desktop-titlebar__app-name {
    max-width: 42vw;
    font-size: 0.88rem;
  }
}
</style>
