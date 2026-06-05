import { nextTick, onMounted, onUnmounted } from "vue";
import { useContextMenuStore } from "@stores/contextMenuStore";
import { useSettingsStore } from "@stores/settingsStore";
import { isBrowserEnv } from "@api/tauri";

export function useGlobalContextMenu() {
  const contextMenuStore = useContextMenuStore();
  const settingsStore = useSettingsStore();

  async function handleGlobalContextMenu(event: MouseEvent) {
    if (isBrowserEnv() || settingsStore.settings.developer_mode) {
      return;
    }

    event.preventDefault();

    if (contextMenuStore.visible) {
      contextMenuStore.hideContextMenu();
      await nextTick();
    }

    const allElements = document.elementsFromPoint(event.clientX, event.clientY) as HTMLElement[];
    const filteredElements = allElements.filter((el) => !el.closest(".sl-context-menu-backdrop"));

    let ctx = "global";
    let targetData = "";

    for (const el of filteredElements) {
      if (el.dataset?.contextMenu) {
        ctx = el.dataset.contextMenu;
        targetData = el.dataset.contextMenuTarget ?? "";
        break;
      }
    }

    if (!targetData) {
      const target = filteredElements[0];
      if (target) {
        const tag = target.tagName.toLowerCase();
        const text = target.textContent?.trim() || "";
        targetData =
          text.length > 100
            ? `${tag}(${text.substring(0, 100)}...)`
            : text
              ? `${tag}(${text})`
              : tag;
      }
    }

    if (ctx !== "global" && !contextMenuStore.hasMenuItems(ctx)) {
      ctx = "global";
    }

    if (!contextMenuStore.hasMenuItems(ctx)) {
      return;
    }

    contextMenuStore.showContextMenu(ctx, event.clientX, event.clientY, targetData);
  }

  onMounted(() => {
    contextMenuStore.initContextMenuListener();
    document.addEventListener("contextmenu", handleGlobalContextMenu);
  });

  onUnmounted(() => {
    document.removeEventListener("contextmenu", handleGlobalContextMenu);
    contextMenuStore.cleanupContextMenuListener();
  });
}
