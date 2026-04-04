<script setup lang="ts">
import { ref, computed } from "vue";
import SLMenuBase from "./SLMenuBase.vue";
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

const menuItems = computed(() => {
  return contextMenuStore.items.map((item: ContextMenuItem) => ({
    id: `${item.pluginId}-${item.id}`,
    label: item.label,
    icon: item.icon,
  }));
});

function handleItemClick(item: { id: string; label: string; icon?: string }) {
  const originalItem = contextMenuStore.items.find(
    (i: ContextMenuItem) => `${i.pluginId}-${i.id}` === item.id,
  );
  if (originalItem) {
    contextMenuStore.handleItemClick(originalItem);
  }
}

function hideContextMenu() {
  contextMenuStore.hideContextMenu();
}
</script>

<template>
  <Teleport to="body">
    <Transition name="context-menu-fade">
      <div
        v-if="contextMenuStore.visible"
        class="sl-context-menu-backdrop"
        @click="hideContextMenu"
        @contextmenu.prevent="hideContextMenu"
      >
        <div ref="menuRef" class="sl-context-menu" :style="menuStyle" @click.stop role="menu">
          <SLMenuBase
            :items="menuItems"
            :header="contextMenuStore.targetData || undefined"
            @select="handleItemClick"
          >
            <template #empty>No menu items</template>
          </SLMenuBase>
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
  z-index: 100001;
  transform-origin: top left;
}

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
</style>
