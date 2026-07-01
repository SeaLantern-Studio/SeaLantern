<script setup lang="ts">
import { LayoutGrid, LogOut } from "@lucide/vue";
import { RouterLink } from "vue-router";
import type { NextShellNavItem } from "../../contracts/shell";

interface Props {
  brand: string;
  railLabel: string;
  pageTitle: string;
  pageSubtitle: string;
  logoutLabel: string;
  showLogout?: boolean;
  navItems: NextShellNavItem[];
  railLocked?: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  logout: [];
}>();
</script>

<template>
  <div class="next-shell-frame">
    <aside class="next-shell-frame__rail" :class="{ 'next-shell-frame__rail--locked': railLocked }">
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
      <header class="next-shell-frame__header">
        <div class="next-shell-frame__header-copy">
          <span class="next-shell-frame__header-eyebrow">{{ railLabel }}</span>
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

      <div class="next-shell-frame__overlay-layer">
        <slot name="overlay" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.next-shell-frame {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 104px minmax(0, 1fr);
  background:
    radial-gradient(
      circle at top left,
      color-mix(in srgb, var(--sl-primary) 12%, transparent),
      transparent 24%
    ),
    linear-gradient(180deg, color-mix(in srgb, var(--sl-surface) 88%, transparent), var(--sl-bg));
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

.next-shell-frame__brand-copy span,
.next-shell-frame__header-eyebrow {
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
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow-y: auto;
  overflow-x: hidden;
}

.next-shell-frame__header {
  padding: 24px 28px 0 12px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
}

.next-shell-frame__header-copy {
  display: grid;
  gap: 10px;
}

.next-shell-frame__header-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.next-shell-frame__header-title-group {
  display: grid;
  gap: 6px;
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
  padding: 24px 28px 28px 12px;
  overflow: visible;
}

.next-shell-frame__overlay-layer {
  pointer-events: none;
}

@media (hover: hover) and (pointer: fine) {
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within {
    width: 272px;
  }

  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover .next-shell-frame__brand-copy,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover .next-shell-frame__nav-label,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within
    .next-shell-frame__brand-copy,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within
    .next-shell-frame__nav-label {
    max-width: 160px;
    opacity: 1;
    transform: translateX(0);
    pointer-events: auto;
  }

  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover .next-shell-frame__brand,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within
    .next-shell-frame__brand,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover .next-shell-frame__nav-item,
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within
    .next-shell-frame__nav-item {
    padding-left: 12px;
    padding-right: 12px;
    gap: 12px;
  }

  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover
    .next-shell-frame__sidebar-slot
    :deep(.next-sidebar-host-items__item),
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within
    .next-shell-frame__sidebar-slot
    :deep(.next-sidebar-host-items__item) {
    padding-left: 12px;
    padding-right: 12px;
    gap: 10px;
  }

  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):hover
    .next-shell-frame__sidebar-slot
    :deep(.next-sidebar-host-items__label),
  .next-shell-frame__rail:not(.next-shell-frame__rail--locked):focus-within
    .next-shell-frame__sidebar-slot
    :deep(.next-sidebar-host-items__label) {
    max-width: 160px;
    opacity: 1;
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

  .next-shell-frame__header {
    flex-direction: column;
    align-items: stretch;
    padding-top: 16px;
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

  .next-shell-frame__main {
    padding-top: 18px;
    padding-bottom: 20px;
  }
}
</style>
