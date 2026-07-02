<script setup lang="ts">
import { LayoutGrid, LogOut } from "@lucide/vue";
import { RouterLink } from "vue-router";
import { computed, onMounted, useTemplateRef, watch } from "vue";
import { i18n } from "@language";
import { SLButton, SLCheckbox, SLModal } from "@components/common";
import { useShellBackground } from "@composables/useShellBackground";
import { useSettingsStore } from "@stores/settingsStore";
import type { NextShellNavItem } from "@next-src/contracts/shell";
import NextDesktopTitlebar from "./NextDesktopTitlebar.vue";
import { useNextDesktopWindowControls } from "@next-src/composables/useNextDesktopWindowControls";

interface Props {
  brand: string;
  railLabel: string;
  pageTitle: string;
  pageSubtitle: string;
  logoutLabel: string;
  showLogout?: boolean;
  navItems: NextShellNavItem[];
  railLocked?: boolean;
  railExpanded?: boolean;
  pageTransitionClass?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  showLogout: true,
  railLocked: false,
  railExpanded: false,
  pageTransitionClass: null,
});

const emit = defineEmits<{
  logout: [];
  pageTransitionSettled: [];
  railFocusWithinChange: [value: boolean];
  railPointerInsideChange: [value: boolean];
}>();

const railRef = useTemplateRef<HTMLElement>("railRef");
const { backgroundStyle, hasBackgroundImage, hasTransparentWindowBackdrop } = useShellBackground();
const frameClasses = computed(() => ({
  "next-shell-frame--has-background-image": hasBackgroundImage.value,
  "next-shell-frame--native-window-effect": hasTransparentWindowBackdrop.value,
}));

function handleRailFocusOut(event: FocusEvent): void {
  const nextTarget = event.relatedTarget;
  if (nextTarget instanceof Node && railRef.value?.contains(nextTarget)) {
    return;
  }

  emit("railFocusWithinChange", false);
}

const settingsStore = useSettingsStore();
const {
  isDesktop,
  isMacOS,
  showCustomControls,
  isMaximized,
  showCloseModal,
  rememberChoice,
  syncCloseAction,
  minimizeWindow,
  toggleMaximize,
  closeWindow,
  handleCloseOption,
} = useNextDesktopWindowControls();

watch(
  () => settingsStore.settings.close_action,
  (value) => {
    syncCloseAction(value);
  },
  { immediate: true },
);

onMounted(() => {
  emit("railPointerInsideChange", railRef.value?.matches(":hover") ?? false);
});
</script>

<template>
  <div class="next-shell-frame" :class="frameClasses">
    <div class="next-shell-frame__background" :style="backgroundStyle"></div>
    <div class="next-shell-frame__backdrop"></div>

    <aside
      ref="railRef"
      class="next-shell-frame__rail"
      :class="{
        'next-shell-frame__rail--expanded': props.railExpanded,
        'next-shell-frame__rail--locked': props.railLocked,
      }"
      @focusin="emit('railFocusWithinChange', true)"
      @focusout="handleRailFocusOut"
      @pointerenter="emit('railPointerInsideChange', true)"
      @pointerleave="emit('railPointerInsideChange', false)"
    >
      <div class="next-shell-frame__rail-inner">
        <div class="next-shell-frame__brand">
          <div class="next-shell-frame__brand-mark">
            <LayoutGrid :size="18" />
          </div>

          <div class="next-shell-frame__brand-copy">
            <strong>{{ brand }}</strong>
            <span>{{ railLabel }}</span>
          </div>
        </div>

        <nav class="next-shell-frame__nav" :aria-label="railLabel">
          <template v-for="item in navItems" :key="item.id">
            <RouterLink
              v-if="!item.disabled && item.to"
              class="next-shell-frame__nav-item"
              :class="{
                'next-shell-frame__nav-item--active': item.active,
              }"
              :to="item.to"
              :aria-current="item.active ? 'page' : undefined"
            >
              <component :is="item.icon" :size="18" class="next-shell-frame__nav-icon" />
              <span class="next-shell-frame__nav-label">{{ item.label }}</span>
            </RouterLink>

            <button
              v-else
              class="next-shell-frame__nav-item next-shell-frame__nav-item--disabled"
              :class="{
                'next-shell-frame__nav-item--active': item.active,
              }"
              type="button"
              disabled
              :aria-current="item.active ? 'page' : undefined"
              :aria-disabled="true"
            >
              <component :is="item.icon" :size="18" class="next-shell-frame__nav-icon" />
              <span class="next-shell-frame__nav-label">{{ item.label }}</span>
            </button>
          </template>
        </nav>

        <div class="next-shell-frame__sidebar-slot">
          <slot name="sidebar-primary" />
        </div>
      </div>
    </aside>

    <div class="next-shell-frame__body">
      <NextDesktopTitlebar
        v-if="isDesktop"
        :macos="isMacOS"
        :show-controls="showCustomControls"
        :is-maximized="isMaximized"
        @minimize="minimizeWindow"
        @toggle-maximize="toggleMaximize"
        @close="closeWindow"
      />

      <div class="next-shell-frame__content-scroll">
        <div
          class="next-shell-frame__page-stage"
          :class="props.pageTransitionClass"
          @animationend.self="emit('pageTransitionSettled')"
        >
          <header class="next-shell-frame__header">
            <div class="next-shell-frame__header-copy">
              <div class="next-shell-frame__header-title-group">
                <h1>{{ pageTitle }}</h1>
                <p v-if="pageSubtitle">{{ pageSubtitle }}</p>
              </div>
            </div>

            <div class="next-shell-frame__header-actions">
              <slot name="header-primary-actions" />

              <button
                v-if="showLogout !== false"
                class="next-shell-frame__logout"
                type="button"
                @click="emit('logout')"
              >
                <LogOut :size="16" />
                <span>{{ logoutLabel }}</span>
              </button>
            </div>
          </header>

          <main class="next-shell-frame__main">
            <slot />
          </main>
        </div>
      </div>

      <div class="next-shell-frame__overlay-layer">
        <slot name="overlay" />
      </div>
    </div>
  </div>

  <SLModal
    :visible="showCloseModal"
    :title="i18n.t('home.close_window_title')"
    @close="showCloseModal = false"
  >
    <div class="next-shell-frame__close-modal-content">
      <p>{{ i18n.t("home.close_window_message") }}</p>
      <div class="next-shell-frame__remember-option">
        <SLCheckbox v-model="rememberChoice" :label="i18n.t('home.remember_choice')" />
      </div>
      <div class="next-shell-frame__close-options">
        <SLButton variant="secondary" @click="handleCloseOption('minimize')">
          {{ i18n.t("home.close_action_minimize") }}
        </SLButton>
        <SLButton variant="danger" @click="handleCloseOption('close')">
          {{ i18n.t("home.close_action_close") }}
        </SLButton>
      </div>
    </div>
  </SLModal>
</template>

<style scoped>
.next-shell-frame {
  position: relative;
  isolation: isolate;
  min-height: 100vh;
  display: grid;
  grid-template-columns: 104px minmax(0, 1fr);
  overflow: hidden;
  background-color: var(--sl-bg);
}

.next-shell-frame--native-window-effect {
  background-color: transparent;
}

.next-shell-frame__background,
.next-shell-frame__backdrop {
  position: absolute;
  pointer-events: none;
}

.next-shell-frame__background {
  inset: -24px;
  z-index: 0;
  transition:
    opacity 0.3s ease,
    filter 0.3s ease,
    background-size 0.3s ease;
}

.next-shell-frame__backdrop {
  inset: 0;
  z-index: 0;
  background:
    radial-gradient(
      circle at top left,
      color-mix(in srgb, var(--sl-primary) 12%, transparent),
      transparent 24%
    ),
    linear-gradient(180deg, color-mix(in srgb, var(--sl-surface) 88%, transparent), var(--sl-bg));
}

.next-shell-frame--has-background-image .next-shell-frame__backdrop {
  background:
    radial-gradient(
      circle at top left,
      color-mix(in srgb, var(--sl-primary) 14%, transparent),
      transparent 26%
    ),
    linear-gradient(
      180deg,
      color-mix(in srgb, var(--sl-surface) 72%, transparent),
      color-mix(in srgb, var(--sl-bg) 88%, transparent)
    );
}

.next-shell-frame--native-window-effect .next-shell-frame__backdrop {
  background:
    radial-gradient(
      circle at top left,
      color-mix(in srgb, var(--sl-primary) 10%, transparent),
      transparent 24%
    ),
    linear-gradient(
      180deg,
      color-mix(in srgb, var(--sl-surface) 54%, transparent),
      color-mix(in srgb, var(--sl-bg) 70%, transparent)
    );
}

.next-shell-frame__rail {
  position: sticky;
  top: 0;
  width: 104px;
  height: 100vh;
  padding: 18px 16px;
  z-index: 2;
  transition: width var(--sl-transition-normal);
}

.next-shell-frame__rail-inner {
  height: 100%;
  width: 100%;
  display: grid;
  grid-template-rows: auto 1fr;
  gap: 18px;
  padding: 16px 14px;
  border: 1px solid color-mix(in srgb, var(--sl-border) 86%, transparent);
  border-radius: 28px;
  background: color-mix(in srgb, var(--sl-surface) 86%, transparent);
  backdrop-filter: blur(18px);
  box-shadow: var(--sl-shadow-md);
  overflow: hidden;
}

.next-shell-frame__brand,
.next-shell-frame__nav-item,
.next-shell-frame__logout {
  min-width: 0;
}

.next-shell-frame__brand {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding-left: calc(50% - 20px);
  gap: 0;
  transition:
    padding-left var(--sl-transition-normal),
    gap var(--sl-transition-fast);
}

.next-shell-frame__brand-mark {
  width: 40px;
  height: 40px;
  flex: 0 0 auto;
  display: grid;
  place-items: center;
  line-height: 0;
  border-radius: 14px;
  background: color-mix(in srgb, var(--sl-primary) 18%, var(--sl-surface));
  color: var(--sl-primary);
}

.next-shell-frame__brand-mark :deep(svg) {
  display: block;
}

.next-shell-frame__brand-copy,
.next-shell-frame__nav-label {
  opacity: 0;
  transform: translateX(-6px);
  transition:
    max-width var(--sl-transition-fast),
    opacity var(--sl-transition-fast),
    transform var(--sl-transition-fast);
  pointer-events: none;
  white-space: nowrap;
  max-width: 0;
  overflow: hidden;
}

.next-shell-frame__brand-copy {
  display: grid;
  gap: 2px;
  flex: 0 1 auto;
}

.next-shell-frame__brand-copy strong,
.next-shell-frame__header-title-group h1,
.next-shell-frame__header-title-group p {
  margin: 0;
}

.next-shell-frame__brand-copy strong {
  font-size: var(--sl-font-size-sm);
  font-weight: 700;
  color: var(--sl-text-primary);
}

.next-shell-frame__brand-copy span {
  font-size: var(--sl-font-size-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--sl-text-tertiary);
}

.next-shell-frame__nav {
  display: grid;
  align-content: start;
  gap: 8px;
}

.next-shell-frame__sidebar-slot {
  align-self: end;
}

.next-shell-frame__sidebar-slot :deep(.next-sidebar-host-items__item) {
  justify-content: flex-start;
  padding-inline: 0;
  padding-left: calc(50% - 8px);
  gap: 0;
  transition:
    padding-left var(--sl-transition-normal),
    padding-right var(--sl-transition-normal),
    gap var(--sl-transition-fast);
}

.next-shell-frame__sidebar-slot :deep(.next-sidebar-host-items__label) {
  max-width: 0;
  opacity: 0;
  overflow: hidden;
  transition:
    max-width var(--sl-transition-fast),
    opacity var(--sl-transition-fast);
}

.next-shell-frame__nav-item,
.next-shell-frame__logout {
  min-height: 48px;
  padding: 0;
  display: flex;
  align-items: center;
  gap: 0;
  line-height: 0;
  border: 0;
  border-radius: 16px;
  background: transparent;
  color: var(--sl-text-secondary);
  font: inherit;
  text-decoration: none;
}

.next-shell-frame__nav-item {
  cursor: pointer;
  justify-content: flex-start;
  padding-left: calc(50% - 9px);
  transition:
    padding-left var(--sl-transition-normal),
    padding-right var(--sl-transition-normal),
    gap var(--sl-transition-fast),
    background-color var(--sl-transition-fast),
    color var(--sl-transition-fast),
    box-shadow var(--sl-transition-fast);
}

.next-shell-frame__nav-item--active {
  background: color-mix(in srgb, var(--sl-primary) 12%, transparent);
  color: var(--sl-text-primary);
}

.next-shell-frame__nav-item--disabled {
  opacity: 0.58;
  cursor: not-allowed;
}

.next-shell-frame__nav-item:focus-visible,
.next-shell-frame__logout:focus-visible {
  outline: none;
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sl-primary) 40%, transparent);
}

@media (hover: hover) and (pointer: fine) {
  .next-shell-frame__nav-item:not(.next-shell-frame__nav-item--active):not(
      .next-shell-frame__nav-item--disabled
    ):hover {
    background: color-mix(in srgb, var(--sl-primary) 8%, transparent);
    color: var(--sl-text-primary);
  }
}

.next-shell-frame__nav-icon {
  flex: 0 0 auto;
  display: block;
}

.next-shell-frame__logout :deep(svg) {
  display: block;
}

.next-shell-frame__body {
  min-width: 0;
  min-height: 100vh;
  height: 100vh;
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.next-shell-frame__content-scroll {
  min-width: 0;
  min-height: 0;
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
}

.next-shell-frame__page-stage {
  min-width: 0;
  min-height: 100%;
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
}

.next-shell-frame__header {
  padding: 18px 28px 0 12px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
}

.next-shell-frame__header-copy {
  display: grid;
  gap: 6px;
}

.next-shell-frame__header-title-group {
  display: grid;
  gap: 6px;
}

.next-shell-frame__header-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.next-shell-frame__header-title-group h1 {
  font-size: clamp(2rem, 4vw, 3rem);
  line-height: 0.96;
  letter-spacing: -0.045em;
  color: var(--sl-text-primary);
}

.next-shell-frame__header-title-group p {
  max-width: 52ch;
  color: var(--sl-text-secondary);
  font-size: var(--sl-font-size-lg);
  line-height: var(--sl-line-height-relaxed);
}

.next-shell-frame__logout {
  justify-content: center;
  flex: 0 0 auto;
  padding-inline: 16px;
  border: 1px solid var(--sl-border);
  background: color-mix(in srgb, var(--sl-surface) 82%, transparent);
  color: var(--sl-text-primary);
  cursor: pointer;
}

.next-shell-frame__main {
  min-width: 0;
  min-height: 0;
  flex: 1 1 auto;
  padding: 24px 28px 28px 12px;
  overflow: visible;
}

.next-shell-frame__overlay-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.next-shell-frame__close-modal-content {
  display: grid;
  gap: 16px;
}

.next-shell-frame__close-modal-content p {
  margin: 0;
  color: var(--sl-text-secondary);
}

.next-shell-frame__remember-option {
  min-width: 0;
}

.next-shell-frame__close-options {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

@media (hover: hover) and (pointer: fine) {
  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked) {
    width: 272px;
  }

  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked)
    .next-shell-frame__brand-copy,
  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked)
    .next-shell-frame__nav-label {
    max-width: 160px;
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
  }

  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked) .next-shell-frame__brand,
  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked)
    .next-shell-frame__nav-item {
    padding-left: 12px;
    padding-right: 12px;
    gap: 12px;
  }

  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked)
    .next-shell-frame__sidebar-slot
    :deep(.next-sidebar-host-items__item) {
    padding-left: 12px;
    padding-right: 12px;
    gap: 10px;
  }

  .next-shell-frame__rail--expanded:not(.next-shell-frame__rail--locked)
    .next-shell-frame__sidebar-slot
    :deep(.next-sidebar-host-items__label) {
    max-width: 160px;
    opacity: 1;
  }
}

@media (prefers-reduced-motion: no-preference) {
  .next-shell-frame__page-stage--enter-down,
  .next-shell-frame__page-stage--enter-up {
    animation-duration: 320ms;
    animation-timing-function: cubic-bezier(0.22, 1, 0.36, 1);
    animation-fill-mode: both;
  }

  .next-shell-frame__page-stage--enter-down {
    animation-name: next-shell-frame-page-enter-down;
  }

  .next-shell-frame__page-stage--enter-up {
    animation-name: next-shell-frame-page-enter-up;
  }

  @keyframes next-shell-frame-page-enter-down {
    from {
      opacity: 0;
      transform: translateY(-28px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes next-shell-frame-page-enter-up {
    from {
      opacity: 0;
      transform: translateY(28px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
}

@media (prefers-reduced-motion: reduce) {
  .next-shell-frame__rail,
  .next-shell-frame__brand,
  .next-shell-frame__brand-copy,
  .next-shell-frame__nav-item,
  .next-shell-frame__nav-label,
  .next-shell-frame__sidebar-slot :deep(.next-sidebar-host-items__item),
  .next-shell-frame__sidebar-slot :deep(.next-sidebar-host-items__label) {
    transition-duration: 0ms;
  }
}

@media (max-width: 1023px) {
  .next-shell-frame {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto 1fr;
  }

  .next-shell-frame__rail {
    position: static;
    width: auto;
    height: auto;
    padding: 18px 18px 0;
  }

  .next-shell-frame__rail-inner {
    grid-template-rows: auto auto;
    gap: 14px;
    padding: 14px;
  }

  .next-shell-frame__brand-copy,
  .next-shell-frame__nav-label {
    max-width: none;
    opacity: 1;
    transform: none;
    pointer-events: auto;
  }

  .next-shell-frame__brand {
    justify-content: flex-start;
    gap: 12px;
  }

  .next-shell-frame__nav {
    grid-auto-flow: column;
    grid-auto-columns: minmax(0, 1fr);
  }

  .next-shell-frame__nav-item {
    justify-content: center;
    padding-inline: 10px;
  }

  .next-shell-frame__sidebar-slot :deep(.next-sidebar-host-items__item) {
    justify-content: flex-start;
    padding-inline: 12px;
  }

  .next-shell-frame__sidebar-slot :deep(.next-sidebar-host-items__label) {
    max-width: none;
    opacity: 1;
  }

  .next-shell-frame__header,
  .next-shell-frame__main {
    padding-inline: 18px;
  }

  :deep(.next-desktop-titlebar) {
    padding-inline: 18px;
  }

  .next-shell-frame__header {
    padding-top: 18px;
  }
}

@media (max-width: 767px) {
  .next-shell-frame__rail {
    padding: 12px 12px 0;
  }

  .next-shell-frame__rail-inner {
    border-radius: 22px;
  }

  .next-shell-frame__nav {
    grid-auto-flow: row;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .next-shell-frame__nav-item {
    justify-content: flex-start;
  }

  .next-shell-frame__header,
  .next-shell-frame__main {
    padding-inline: 12px;
  }

  :deep(.next-desktop-titlebar) {
    padding-inline: 12px;
  }

  .next-shell-frame__header {
    flex-direction: column;
    align-items: stretch;
    padding-top: 14px;
  }

  .next-shell-frame__header-actions {
    justify-content: stretch;
    flex-direction: column;
    align-items: stretch;
  }

  .next-shell-frame__header-title-group h1 {
    font-size: 2.2rem;
  }

  .next-shell-frame__header-title-group p {
    font-size: var(--sl-font-size-base);
  }

  .next-shell-frame__logout {
    width: 100%;
  }

  .next-shell-frame__close-options {
    flex-direction: column;
  }

  .next-shell-frame__main {
    padding-top: 18px;
    padding-bottom: 20px;
  }
}
</style>
