import { onMounted, onUnmounted, type Ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isBrowserEnv } from "@api/tauri";

export type CreateServerSourceType = "archive" | "folder" | "";

export const CREATE_SERVER_SOURCE_DROP_EVENT = "create-server-source-drop";
const CREATE_SERVER_DND_DEBUG = import.meta.env.DEV;

function logCreateServerDnd(message: string, payload?: unknown) {
  if (!CREATE_SERVER_DND_DEBUG) return;
  if (payload === undefined) {
    console.debug(message);
    return;
  }
  console.debug(message, payload);
}

function inferSourceType(path: string): CreateServerSourceType {
  const lowerPath = path.toLowerCase();
  if (
    lowerPath.endsWith(".zip") ||
    lowerPath.endsWith(".tar") ||
    lowerPath.endsWith(".tar.gz") ||
    lowerPath.endsWith(".tgz") ||
    lowerPath.endsWith(".jar") ||
    lowerPath.endsWith(".exe")
  ) {
    return "archive";
  }

  return "folder";
}

interface UseCreateServerDropSourceOptions {
  sourcePath: Ref<string>;
  sourceType: Ref<CreateServerSourceType>;
}

export function useCreateServerDropSource(options: UseCreateServerDropSourceOptions) {
  let unlistenSourceDropEvent: UnlistenFn | null = null;

  function applyDroppedPaths(paths: string[]) {
    if (paths.length === 0) {
      return;
    }

    const path = paths[0];
    options.sourcePath.value = path;
    options.sourceType.value = inferSourceType(path);
  }

  onMounted(async () => {
    if (isBrowserEnv()) {
      return;
    }

    try {
      unlistenSourceDropEvent = await listen<string[]>(CREATE_SERVER_SOURCE_DROP_EVENT, (event) => {
        const droppedPaths = Array.isArray(event.payload) ? event.payload : [];
        logCreateServerDnd("[useCreateServerDropSource] Received source drop event", droppedPaths);
        applyDroppedPaths(droppedPaths);
      });
    } catch (error) {
      logCreateServerDnd(
        "[useCreateServerDropSource] Failed to register source drop listener",
        error,
      );
    }
  });

  onUnmounted(() => {
    if (!unlistenSourceDropEvent) {
      return;
    }

    unlistenSourceDropEvent();
    unlistenSourceDropEvent = null;
  });

  return {
    applyDroppedPaths,
  };
}
