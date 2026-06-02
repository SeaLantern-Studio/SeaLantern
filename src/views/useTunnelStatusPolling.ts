import { onMounted, onUnmounted, ref } from "vue";
import type { TunnelStatus } from "@api/tunnel";
import { tunnelApi } from "@api/tunnel";
import { useGlobalMessage } from "@composables/useMessage";
import { useSerialPolling } from "@composables/useSerialPolling";

interface UseTunnelStatusPollingOptions {
  applyStatus: (status: TunnelStatus) => void;
}

export function useTunnelStatusPolling(options: UseTunnelStatusPollingOptions) {
  const globalMessage = useGlobalMessage();
  const tunnelViewDisposed = ref(false);

  async function refreshStatus(silent = false) {
    try {
      const next = await tunnelApi.status();
      if (!tunnelViewDisposed.value) {
        options.applyStatus(next);
      }
    } catch (error) {
      if (!silent && !tunnelViewDisposed.value) {
        globalMessage.error(String(error));
      }
    }
  }

  const statusPolling = useSerialPolling({
    intervalMs: 2000,
    task: async () => {
      await refreshStatus(true);
    },
  });

  function startStatusPolling() {
    tunnelViewDisposed.value = false;
    statusPolling.start();
  }

  function stopStatusPolling() {
    statusPolling.stop();
  }

  onMounted(() => {
    tunnelViewDisposed.value = false;
  });

  onUnmounted(() => {
    tunnelViewDisposed.value = true;
    stopStatusPolling();
  });

  return {
    refreshStatus,
    startStatusPolling,
    stopStatusPolling,
  };
}
