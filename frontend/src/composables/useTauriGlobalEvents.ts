import { onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { serverApi, type AppOperationEvent } from "@api/server";
import { isBrowserEnv } from "@api/tauri";
import { useGlobalMessage } from "@composables/useMessage";
import { i18n } from "@language";
import { isPluginRuntimeUiBridgeAvailable } from "@src/services/hostCapabilities";
import { usePluginStore } from "@stores/pluginStore";

let sharedAudioContext: AudioContext | null = null;
let lastNotificationAt = 0;

async function getSharedAudioContext(): Promise<AudioContext | null> {
  const AudioContextCtor =
    window.AudioContext ||
    (window as Window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AudioContextCtor) {
    return null;
  }

  if (!sharedAudioContext || sharedAudioContext.state === "closed") {
    sharedAudioContext = new AudioContextCtor();
  }

  if (sharedAudioContext.state === "suspended") {
    await sharedAudioContext.resume().catch(() => {});
  }

  return sharedAudioContext;
}

interface ServerStartFallbackEventPayload {
  serverId: string;
  serverName: string;
  fromMode: string;
  toMode: string;
  reason: string;
}

const BUILTIN_PLUGIN_ENABLE_FAILED_ACTION = "builtin_plugin_enable_failed";
const BUILTIN_PLUGIN_ENABLE_FAILED_TOAST = "插件启用失败，详细信息请查看日志。";

function getStartupModeLabel(mode: string): string {
  switch (mode) {
    case "starter":
      return i18n.t("create.startup_mode_starter");
    case "jar":
      return i18n.t("create.startup_mode_jar");
    case "bat":
    case "sh":
    case "ps1":
      return i18n.t("create.startup_mode_script");
    case "custom":
      return i18n.t("create.startup_mode_custom");
    default:
      return mode;
  }
}

async function playNotificationSound() {
  const now = Date.now();
  if (now - lastNotificationAt < 800) {
    return;
  }
  lastNotificationAt = now;

  try {
    const audioContext = await getSharedAudioContext();
    if (!audioContext) {
      return;
    }

    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(880, audioContext.currentTime);
    oscillator.frequency.setValueAtTime(1100, audioContext.currentTime + 0.1);
    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.3);
    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.3);

    oscillator.addEventListener(
      "ended",
      () => {
        oscillator.disconnect();
        gainNode.disconnect();
      },
      { once: true },
    );
  } catch (error) {
    console.warn("播放提示音失败:", error);
  }
}

export function useTauriGlobalEvents() {
  const pluginStore = usePluginStore();
  const globalMessage = useGlobalMessage();
  const cleanups: Array<() => void> = [];
  const seenAppOperationEventIds = new Set<string>();
  let disposed = false;

  function handleBuiltinPluginEnableFailure(event: AppOperationEvent): void {
    if (
      event.kind !== "operation_failed" ||
      event.action !== BUILTIN_PLUGIN_ENABLE_FAILED_ACTION ||
      seenAppOperationEventIds.has(event.event_id)
    ) {
      return;
    }

    seenAppOperationEventIds.add(event.event_id);
    globalMessage.error(BUILTIN_PLUGIN_ENABLE_FAILED_TOAST);
  }

  function registerCleanup(cleanup: () => void) {
    if (disposed) {
      cleanup();
      return;
    }
    cleanups.push(cleanup);
  }

  onMounted(async () => {
    await pluginStore.initPermissionLogListener();
    await pluginStore.initPluginLogListener();
    await pluginStore.initI18nEventListener();

    registerCleanup(() => pluginStore.cleanupPermissionLogListener());
    registerCleanup(() => pluginStore.cleanupPluginLogListener());
    registerCleanup(() => pluginStore.cleanupI18nEventListener());

    if (await isPluginRuntimeUiBridgeAvailable()) {
      await pluginStore.initUiEventListener();
      await pluginStore.initSidebarEventListener();
      await pluginStore.initComponentEventListener();

      registerCleanup(() => pluginStore.cleanupUiEventListener());
      registerCleanup(() => pluginStore.cleanupSidebarEventListener());
      registerCleanup(() => pluginStore.cleanupComponentEventListener());
    }

    if (isBrowserEnv()) {
      return;
    }

    const unlistenServerError = await listen("server-error", () => {
      void playNotificationSound();
    });
    const unlistenAppOperation = await serverApi.onAppOperationEvent(
      (payload: AppOperationEvent) => {
        handleBuiltinPluginEnableFailure(payload);
      },
    );
    const unlistenFallback = await listen<ServerStartFallbackEventPayload>(
      "server-start-fallback",
      ({ payload }) => {
        const displayName = payload.serverName || payload.serverId;
        const fromMode = getStartupModeLabel(payload.fromMode);
        const toMode = getStartupModeLabel(payload.toMode);
        globalMessage.warning(
          i18n.t("common.server_start_fallback", {
            name: displayName,
            from: fromMode,
            to: toMode,
            reason: payload.reason,
          }),
          5000,
        );
      },
    );

    const recentAppOperations = await serverApi.getRecentAppOperationEvents(32).catch(() => []);
    for (const event of recentAppOperations) {
      handleBuiltinPluginEnableFailure(event);
    }

    registerCleanup(unlistenServerError);
    registerCleanup(unlistenAppOperation);
    registerCleanup(unlistenFallback);
  });

  onUnmounted(() => {
    disposed = true;
    while (cleanups.length > 0) {
      const cleanup = cleanups.pop();
      cleanup?.();
    }
  });
}
