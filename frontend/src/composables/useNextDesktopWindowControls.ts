import type { UnlistenFn } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { isBrowserEnv } from "@api/tauri";
import { useSettingsStore } from "@stores/settingsStore";

import { isMacOSPlatform } from "@utils/platform";

type CloseAction = "ask" | "minimize" | "close";

function resolveCloseAction(value: string | undefined): CloseAction {
  if (value === "minimize" || value === "close") {
    return value;
  }

  return "ask";
}

async function loadCurrentWindow() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  return getCurrentWindow();
}

async function exitApp(): Promise<void> {
  const { exit } = await import("@tauri-apps/plugin-process");
  await exit(0);
}

export function useNextDesktopWindowControls() {
  const settingsStore = useSettingsStore();
  const isDesktop = computed(() => !isBrowserEnv());
  const isMacOS = isMacOSPlatform();
  const showCustomControls = computed(() => isDesktop.value && !isMacOS);
  const isMaximized = ref(false);
  const showCloseModal = ref(false);
  const rememberChoice = ref(false);
  const closeAction = ref<CloseAction>("ask");

  let unlistenResize: (() => void) | null = null;
  let unlistenCloseRequested: UnlistenFn | null = null;
  let isUnmounted = false;

  async function syncMaximizedState(): Promise<void> {
    if (!isDesktop.value) {
      return;
    }

    const appWindow = await loadCurrentWindow();
    isMaximized.value = await appWindow.isMaximized();
  }

  async function minimizeWindow(): Promise<void> {
    if (!isDesktop.value) {
      return;
    }

    const appWindow = await loadCurrentWindow();
    await appWindow.minimize();
  }

  async function toggleMaximize(): Promise<void> {
    if (!isDesktop.value) {
      return;
    }

    const appWindow = await loadCurrentWindow();
    await appWindow.toggleMaximize();
    await syncMaximizedState();
  }

  async function minimizeToTray(): Promise<void> {
    if (!isDesktop.value) {
      return;
    }

    const appWindow = await loadCurrentWindow();

    try {
      await appWindow.hide();
      try {
        await appWindow.setSkipTaskbar(true);
      } catch (error) {
        console.warn("Failed to set skip taskbar:", error);
      }
    } catch (error) {
      console.warn("Failed to hide window for tray minimize:", error);
      await appWindow.minimize();
    }
  }

  async function closeWindow(): Promise<void> {
    if (!isDesktop.value) {
      return;
    }

    if (closeAction.value === "ask") {
      showCloseModal.value = true;
      return;
    }

    if (closeAction.value === "minimize") {
      await minimizeToTray();
      return;
    }

    const appWindow = await loadCurrentWindow();
    await appWindow.close();
  }

  async function handleCloseOption(option: Exclude<CloseAction, "ask">): Promise<void> {
    if (rememberChoice.value) {
      closeAction.value = option;
      try {
        await settingsStore.setCloseAction(option);
      } catch (error) {
        console.error("Failed to save settings:", error);
      }
    }

    if (option === "minimize") {
      await minimizeToTray();
    } else {
      await exitApp();
    }

    showCloseModal.value = false;
    rememberChoice.value = false;
  }

  onMounted(async () => {
    isUnmounted = false;

    if (!isDesktop.value) {
      return;
    }

    await settingsStore.ensureLoaded();
    closeAction.value = resolveCloseAction(settingsStore.settings.close_action);

    const appWindow = await loadCurrentWindow();
    await syncMaximizedState();

    unlistenResize = await appWindow.onResized(async () => {
      await syncMaximizedState();
    });

    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen("close-requested", () => {
      showCloseModal.value = true;
    });

    if (isUnmounted) {
      unlisten();
      return;
    }

    unlistenCloseRequested = unlisten;
  });

  onUnmounted(() => {
    isUnmounted = true;
    unlistenResize?.();
    unlistenResize = null;
    unlistenCloseRequested?.();
    unlistenCloseRequested = null;
  });

  return {
    isDesktop,
    isMacOS,
    showCustomControls,
    isMaximized,
    showCloseModal,
    rememberChoice,
    closeAction,
    syncCloseAction(value: string | undefined) {
      closeAction.value = resolveCloseAction(value);
    },
    minimizeWindow,
    toggleMaximize,
    closeWindow,
    handleCloseOption,
  };
}
