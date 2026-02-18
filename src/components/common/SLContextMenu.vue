<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useContextMenuStore, type ContextMenuItem } from "@/stores/contextMenuStore";

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

function handleClickOutside(event: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    contextMenuStore.hideContextMenu();
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    contextMenuStore.hideContextMenu();
  }
}

watch(
  () => contextMenuStore.visible,
  (visible) => {
    if (visible) {
      setTimeout(() => {
        document.addEventListener("click", handleClickOutside);
        document.addEventListener("contextmenu", handleClickOutside);
      }, 0);
      document.addEventListener("keydown", handleKeydown);
    } else {
      document.removeEventListener("click", handleClickOutside);
      document.removeEventListener("contextmenu", handleClickOutside);
      document.removeEventListener("keydown", handleKeydown);
    }
  }
);

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  document.removeEventListener("contextmenu", handleClickOutside);
  document.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="context-menu-fade">
      <div
        v-if="contextMenuStore.visible"
        ref="menuRef"
        class="sl-context-menu"
        :style="menuStyle"
      >
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
    </Transition>
  </Teleport>
</template>

<style scoped>
.sl-context-menu {
  position: fixed;
  background: rgba(30, 30, 46, 0.95);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 4px;
  min-width: 160px;
  max-width: 280px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 9999;
  user-select: none;
}

.sl-context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.9);
  font-size: 13px;
  transition: background 0.15s ease;
}

.sl-context-menu-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.sl-context-menu-item:active {
  background: rgba(255, 255, 255, 0.15);
}

.sl-context-menu-icon {
  flex-shrink: 0;
  width: 16px;
  text-align: center;
  opacity: 0.8;
}

.sl-context-menu-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sl-context-menu-empty {
  padding: 8px 12px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  text-align: center;
}

/* 淡入淡出动画 */
.context-menu-fade-enter-active,
.context-menu-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.context-menu-fade-enter-from,
.context-menu-fade-leave-to {
  opacity: 0;
  transform: scale(0.95);
}

/* 亮色主题适配 */
[data-theme="light"] .sl-context-menu {
  background: rgba(255, 255, 255, 0.95);
  border-color: rgba(0, 0, 0, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

[data-theme="light"] .sl-context-menu-item {
  color: rgba(0, 0, 0, 0.85);
}

[data-theme="light"] .sl-context-menu-item:hover {
  background: rgba(0, 0, 0, 0.05);
}

[data-theme="light"] .sl-context-menu-item:active {
  background: rgba(0, 0, 0, 0.1);
}

[data-theme="light"] .sl-context-menu-empty {
  color: rgba(0, 0, 0, 0.4);
}
</style>
