import { onBeforeUnmount, onMounted, ref } from "vue";
import { emit, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CREATE_SERVER_SOURCE_DROP_EVENT } from "@components/views/create/useCreateServerDropSource";

const CREATE_SERVER_DEBUG = import.meta.env.DEV;

function logCreateServerWindowDrop(message: string, payload?: unknown) {
  if (!CREATE_SERVER_DEBUG) return;
  if (payload === undefined) {
    console.debug(message);
    return;
  }
  console.debug(message, payload);
}

export function useCreateServerWindowDrop() {
  const isDragging = ref(false);
  let unlistenCreateViewDragDrop: UnlistenFn | null = null;

  onMounted(async () => {
    const tauriInternals = Reflect.get(window, "__TAURI_INTERNALS__");
    logCreateServerWindowDrop("[useCreateServerWindowDrop] mounted", {
      hasTauriInternals: Boolean(tauriInternals),
    });

    if (!tauriInternals) {
      logCreateServerWindowDrop(
        "[useCreateServerWindowDrop] Running outside Tauri, skip native drag-drop listener",
      );
      return;
    }

    try {
      const currentWindow = getCurrentWindow();
      logCreateServerWindowDrop("[useCreateServerWindowDrop] Preparing native drag-drop listener");
      unlistenCreateViewDragDrop = await currentWindow.onDragDropEvent((event) => {
        logCreateServerWindowDrop(
          "[useCreateServerWindowDrop] Native drag-drop event",
          event.payload,
        );
        if (event.payload.type === "enter" || event.payload.type === "over") {
          isDragging.value = true;
          return;
        }

        if (event.payload.type === "drop") {
          isDragging.value = false;
          logCreateServerWindowDrop(
            "[useCreateServerWindowDrop] Emitting source drop event",
            event.payload.paths,
          );
          void emit(CREATE_SERVER_SOURCE_DROP_EVENT, event.payload.paths).catch((error) => {
            logCreateServerWindowDrop(
              "[useCreateServerWindowDrop] Failed to emit source drop event",
              error,
            );
          });
          return;
        }

        isDragging.value = false;
      });
      logCreateServerWindowDrop("[useCreateServerWindowDrop] Native drag-drop listener registered");
    } catch (error) {
      logCreateServerWindowDrop(
        "[useCreateServerWindowDrop] Failed to register native drag-drop listener",
        error,
      );
    }
  });

  onBeforeUnmount(() => {
    if (!unlistenCreateViewDragDrop) {
      return;
    }

    unlistenCreateViewDragDrop();
    unlistenCreateViewDragDrop = null;
  });

  return {
    isDragging,
  };
}
