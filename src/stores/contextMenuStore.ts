import { defineStore } from "pinia";
import { ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  pluginId: string;
}

interface RawMenuItem {
  id: string;
  label: string;
  icon?: string;
}

interface ContextMenuEvent {
  action: "register" | "unregister";
  plugin_id: string;
  context: string;
  items: string;
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  context: string;
  targetData: string;
  items: ContextMenuItem[];
}

export const useContextMenuStore = defineStore("contextMenu", () => {
  const visible = ref(false);
  const x = ref(0);
  const y = ref(0);
  const context = ref("");
  const targetData = ref("");
  const items = ref<ContextMenuItem[]>([]);

  const registeredMenus = new Map<string, Map<string, ContextMenuItem[]>>();

  let contextMenuEventUnlisten: UnlistenFn | null = null;

  function handleContextMenuEvent(event: ContextMenuEvent) {
    const { action, plugin_id, context: ctx, items: rawItems } = event;

    if (action === "register") {
      if (!registeredMenus.has(ctx)) {
        registeredMenus.set(ctx, new Map());
      }
      const contextMap = registeredMenus.get(ctx)!;

      const parsedItems: RawMenuItem[] = JSON.parse(rawItems);
      const menuItems: ContextMenuItem[] = parsedItems.map((item) => ({
        ...item,
        pluginId: plugin_id,
      }));

      contextMap.set(plugin_id, menuItems);
      console.log(
        `[ContextMenu] Registered ${menuItems.length} items for context "${ctx}" from plugin "${plugin_id}"`,
      );
    } else if (action === "unregister") {
      const contextMap = registeredMenus.get(ctx);
      if (contextMap) {
        contextMap.delete(plugin_id);
        console.log(
          `[ContextMenu] Unregistered items for context "${ctx}" from plugin "${plugin_id}"`,
        );
        if (contextMap.size === 0) {
          registeredMenus.delete(ctx);
        }
      }
    }
  }

  async function initContextMenuListener() {
    if (contextMenuEventUnlisten) {
      return;
    }

    try {
      contextMenuEventUnlisten = await listen<ContextMenuEvent>(
        "plugin-context-menu-event",
        (event) => {
          handleContextMenuEvent(event.payload);
        },
      );
      console.log("[ContextMenu] Event listener initialized");
    } catch (e) {
      console.error("[ContextMenu] Failed to initialize event listener:", e);
    }
  }

  function cleanupContextMenuListener() {
    if (contextMenuEventUnlisten) {
      contextMenuEventUnlisten();
      contextMenuEventUnlisten = null;
    }
  }

  function showContextMenu(ctx: string, posX: number, posY: number, data: string) {
    const allItems: ContextMenuItem[] = [];

    const contextMap = registeredMenus.get(ctx);
    if (contextMap && contextMap.size > 0) {
      contextMap.forEach((pluginItems) => {
        allItems.push(...pluginItems);
      });
    }

    if (ctx !== "global") {
      const globalMap = registeredMenus.get("global");
      if (globalMap && globalMap.size > 0) {
        globalMap.forEach((pluginItems) => {
          allItems.push(...pluginItems);
        });
      }
    }

    if (allItems.length === 0) {
      return;
    }

    context.value = ctx;
    targetData.value = data;
    items.value = allItems;
    x.value = posX;
    y.value = posY;
    visible.value = true;

    invoke("context_menu_show_notify", {
      context: ctx,
      targetData: data || null,
      x: posX,
      y: posY,
    }).catch((e) => console.error("[ContextMenu] Failed to notify show:", e));
  }

  function hideContextMenu() {
    visible.value = false;
    items.value = [];
    context.value = "";
    targetData.value = "";
    invoke("context_menu_hide_notify").catch(() => {});
  }

  async function handleItemClick(item: ContextMenuItem) {
    try {
      await invoke("context_menu_callback", {
        pluginId: item.pluginId,
        context: context.value,
        itemId: item.id,
        targetData: targetData.value,
      });
      console.log(
        `[ContextMenu] Callback sent: plugin=${item.pluginId}, context=${context.value}, item=${item.id}`,
      );
    } catch (e) {
      console.error("[ContextMenu] Failed to send callback:", e);
    }

    hideContextMenu();
  }

  function cleanupPluginMenus(pluginId: string) {
    registeredMenus.forEach((contextMap, ctx) => {
      if (contextMap.has(pluginId)) {
        contextMap.delete(pluginId);
        console.log(`[ContextMenu] Cleaned up menus for plugin "${pluginId}" in context "${ctx}"`);
        if (contextMap.size === 0) {
          registeredMenus.delete(ctx);
        }
      }
    });
  }

  function getState(): ContextMenuState {
    return {
      visible: visible.value,
      x: x.value,
      y: y.value,
      context: context.value,
      targetData: targetData.value,
      items: items.value,
    };
  }

  function hasMenuItems(ctx: string): boolean {
    const contextMap = registeredMenus.get(ctx);
    return contextMap !== undefined && contextMap.size > 0;
  }

  return {
    visible,
    x,
    y,
    context,
    targetData,
    items,
    initContextMenuListener,
    cleanupContextMenuListener,
    showContextMenu,
    hideContextMenu,
    handleItemClick,
    cleanupPluginMenus,
    handleContextMenuEvent,
    getState,
    hasMenuItems,
  };
});
