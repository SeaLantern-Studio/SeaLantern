<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";

const route = useRoute();
const appWindow = getCurrentWindow();

const pageTitle = computed(() => {
  return (route.meta?.title as string) || "Sea Lantern";
});

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}
</script>

<template>
  <header class="app-header glass-subtle">
    <div class="header-left">
      <h2 class="page-title">{{ pageTitle }}</h2>
    </div>

    <div class="header-center" data-tauri-drag-region></div>

    <div class="header-right">
      <div class="header-status">
        <span class="status-dot online"></span>
        <span class="status-text">就绪</span>
      </div>

      <div class="window-controls">
        <button class="win-btn" @click="minimizeWindow" title="最小化">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="1" y="5.5" width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="win-btn" @click="toggleMaximize" title="最大化">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="1.5"
              y="1.5"
              width="9"
              height="9"
              rx="1"
              fill="none"
              stroke="currentColor"
              stroke-width="1"
            />
          </svg>
        </button>
        <button class="win-btn win-btn-close" @click="closeWindow" title="关闭">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M2 2l8 8M10 2l-8 8"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--sl-header-height);
  padding: 0 var(--sl-space-md) 0 var(--sl-space-lg);
  border-bottom: 1px solid var(--sl-border-light);
  flex-shrink: 0;
  user-select: none;
  /* 不要在这里加 drag */
}

.header-left,
.header-right {
  -webkit-app-region: no-drag;
}

.page-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--sl-text-primary);
}

.header-center {
  flex: 1;
  height: 100%;
  min-height: var(--sl-header-height);
  -webkit-app-region: drag;
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--sl-space-md);
}

.header-status {
  display: flex;
  align-items: center;
  gap: var(--sl-space-xs);
  padding: 4px 12px;
  background: var(--sl-bg-secondary);
  border-radius: var(--sl-radius-full);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--sl-text-tertiary);
}

.status-dot.online {
  background: var(--sl-success);
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.4);
}

.status-text {
  font-size: 0.8125rem;
  color: var(--sl-text-secondary);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: var(--sl-space-sm);
  -webkit-app-region: no-drag;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  border-radius: var(--sl-radius-sm);
  color: var(--sl-text-secondary);
  transition: all var(--sl-transition-fast);
  -webkit-app-region: no-drag;
  cursor: pointer;
  z-index: 10;
}

.win-btn:hover {
  background: var(--sl-bg-tertiary);
  color: var(--sl-text-primary);
}

.win-btn-close:hover {
  background: var(--sl-error);
  color: white;
}
</style>
