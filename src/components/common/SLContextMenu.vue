<script setup lang="ts">
import { ref, computed } from "vue";
import { useContextMenuStore, type ContextMenuItem } from "@stores/contextMenuStore";

const contextMenuStore = useContextMenuStore();

const menuRef = ref<HTMLElement | null>(null);

const menuStyle = computed(() => {
  if (!contextMenuStore.visible) {
    return { display: "none" };
  }

  let posX = contextMenuStore.x;
  let posY = contextMenuStore.y;

  if (menuRef.value) {
    const menuRect = menuRef.value.getBoundingClientRect();
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;

    if (posX + menuRect.width > windowWidth) {
      posX = windowWidth - menuRect.width - 8;
    }

    if (posY + menuRect.height > windowHeight) {
      posY = windowHeight - menuRect.height - 8;
    }

    posX = Math.max(8, posX);
    posY = Math.max(8, posY);
  }

  return {
    left: `${posX}px`,
    top: `${posY}px`,
  };
});

function handleItemClick(item: ContextMenuItem) {
  contextMenuStore.handleItemClick(item);
}
</script>

<template>
  <Teleport to="body">
    <Transition name="context-menu-fade">
      <div
        v-if="contextMenuStore.visible"
        class="sl-context-menu-backdrop"
        @click="contextMenuStore.hideContextMenu()"
        @contextmenu.prevent="contextMenuStore.hideContextMenu()"
      >
        <div ref="menuRef" class="sl-context-menu" :style="menuStyle" @click.stop>
          <div v-if="contextMenuStore.targetData" class="sl-context-menu-header">
            {{ contextMenuStore.targetData }}
          </div>
          <div
            v-for="item in contextMenuStore.items"
            :key="`${item.pluginId}-${item.id}`"
            class="sl-context-menu-item"
            @click="handleItemClick(item)"
          >
            <span v-if="item.icon" class="sl-context-menu-icon">{{ item.icon }}</span>
            <span class="sl-context-menu-label">{{ item.label }}</span>
          </div>
          <div v-if="contextMenuStore.items.length === 0" class="sl-context-menu-empty">
            No menu items
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.sl-context-menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100000;
}

.sl-context-menu {
  position: fixed;
  background: var(--sl-surface, rgba(30, 30, 46, 0.95));
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--sl-border, rgba(255, 255, 255, 0.1));
  border-radius: var(--sl-radius-lg, 12px);
  padding: var(--sl-space-xs, 4px);
  min-width: 160px;
  max-width: 280px;
  box-shadow:
    0 10px 30px rgba(0, 0, 0, 0.25),
    0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100001;
  user-select: none;
  transform-origin: top left;
}

.sl-context-menu-item {
  display: flex;
  align-items: center;
  gap: var(--sl-space-sm, 8px);
  padding: 10px 12px;
  border-radius: var(--sl-radius-md, 8px);
  cursor: pointer;
  color: var(--sl-text-primary, rgba(255, 255, 255, 0.9));
  font-size: 0.875rem;
  transition:
    background-color 0.15s ease,
    transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
  position: relative;
  overflow: hidden;
}

.sl-context-menu-item::before {
  content: "";
  position: absolute;
  inset: 0;
  background: var(--sl-primary, #0ea5e9);
  opacity: 0;
  transform: scale(0.5);
  transition: opacity 0.2s ease, transform 0.2s ease;
  border-radius: inherit;
}

.sl-context-menu-item:hover {
  background: var(--sl-surface-hover, rgba(255, 255, 255, 0.1));
}

.sl-context-menu-item:active::before {
  opacity: 0.1;
  transform: scale(1);
}

.sl-context-menu-icon {
  flex-shrink: 0;
  width: 16px;
  text-align: center;
  opacity: 0.8;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.6));
}

.sl-context-menu-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sl-context-menu-header {
  padding: 6px 12px;
  font-size: 0.6875rem;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.45));
  border-bottom: 1px solid var(--sl-border, rgba(255, 255, 255, 0.08));
  margin-bottom: var(--sl-space-xs, 4px);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 260px;
}

.sl-context-menu-empty {
  padding: 8px 12px;
  color: var(--sl-text-tertiary, rgba(255, 255, 255, 0.5));
  font-size: 0.75rem;
  text-align: center;
}

/* 弹性入场动画 */
.context-menu-fade-enter-active {
  animation: context-menu-enter 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.context-menu-fade-leave-active {
  animation: context-menu-leave 0.15s ease;
}

@keyframes context-menu-enter {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes context-menu-leave {
  from {
    opacity: 1;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(0.95);
  }
}

/* 菜单项交错入场动画 */
.sl-context-menu-item {
  animation: menu-item-fade-in 0.2s ease backwards;
}

.sl-context-menu-item:nth-child(1) { animation-delay: 0.02s; }
.sl-context-menu-item:nth-child(2) { animation-delay: 0.04s; }
.sl-context-menu-item:nth-child(3) { animation-delay: 0.06s; }
.sl-context-menu-item:nth-child(4) { animation-delay: 0.08s; }
.sl-context-menu-item:nth-child(5) { animation-delay: 0.1s; }
.sl-context-menu-item:nth-child(6) { animation-delay: 0.12s; }
.sl-context-menu-item:nth-child(7) { animation-delay: 0.14s; }
.sl-context-menu-item:nth-child(8) { animation-delay: 0.16s; }

@keyframes menu-item-fade-in {
  from {
    opacity: 0;
    transform: translateX(-8px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

/* 亮色主题适配 */
[data-theme="light"] .sl-context-menu {
  background: var(--sl-surface, rgba(255, 255, 255, 0.95));
  border-color: var(--sl-border, rgba(0, 0, 0, 0.1));
  box-shadow:
    0 10px 30px rgba(0, 0, 0, 0.12),
    0 4px 12px rgba(0, 0, 0, 0.08);
}

[data-theme="light"] .sl-context-menu-item {
  color: var(--sl-text-primary, rgba(0, 0, 0, 0.85));
}

[data-theme="light"] .sl-context-menu-item:hover {
  background: var(--sl-surface-hover, rgba(0, 0, 0, 0.05));
}

[data-theme="light"] .sl-context-menu-header {
  color: var(--sl-text-tertiary, rgba(0, 0, 0, 0.4));
  border-bottom-color: var(--sl-border, rgba(0, 0, 0, 0.08));
}

[data-theme="light"] .sl-context-menu-empty {
  color: var(--sl-text-tertiary, rgba(0, 0, 0, 0.4));
}
</style>
