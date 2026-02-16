import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { serverApi } from "../api/server";
import type { ServerInstance } from "../types/server";
import type { ServerStatusInfo } from "../api/server";

export const useServerStore = defineStore("server", () => {
  const servers = ref<ServerInstance[]>([]);
  const currentServerId = ref<string | null>(null);
  const statuses = ref<Record<string, ServerStatusInfo>>({});
  const loading = ref(false);
  const error = ref<string | null>(null);

  const currentServer = computed(() => {
    if (!currentServerId.value) return null;
    return servers.value.find((s) => s.id === currentServerId.value) || null;
  });

  async function refreshList() {
    loading.value = true;
    error.value = null;
    try {
      servers.value = await serverApi.getList();

      // 如果服务器列表不为空，自动选择第一个服务器
      if (servers.value.length > 0) {
        currentServerId.value = servers.value[0].id;
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  async function refreshStatus(id: string) {
    try {
      statuses.value[id] = await serverApi.getStatus(id);
    } catch (e) {
      console.error("Failed to get status:", e);
    }
  }

  function setCurrentServer(id: string | null) {
    currentServerId.value = id;
  }

  return {
    servers,
    currentServerId,
    currentServer,
    statuses,
    loading,
    error,
    refreshList,
    refreshStatus,
    setCurrentServer,
  };
});
