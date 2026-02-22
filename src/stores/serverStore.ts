import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { serverApi } from "../api/server";
import { configApi } from "../api/config";
import { useAsyncByKey, useLoading } from "../composables/useAsync";
import type { ServerInstance } from "../types/server";
import type { ServerStatusInfo } from "../api/server";

/**
 * 服务器状态管理 Store
 * 管理服务器列表、当前选择、状态等
 */
export const useServerStore = defineStore("server", () => {
  const servers = ref<ServerInstance[]>([]);
  const currentServerId = ref<string | null>(null);
  const statuses = ref<Record<string, ServerStatusInfo>>({});
  const error = ref<string | null>(null);

  const { loading: listLoading, withLoading } = useLoading(false);
  const serverActions = useAsyncByKey<string>();

  /**
   * 当前选中的服务器
   */
  const currentServer = computed(() => {
    if (!currentServerId.value) return null;
    return servers.value.find((s) => s.id === currentServerId.value) || null;
  });

  /**
   * 全局加载状态（列表加载或任意服务器操作）
   */
  const loading = computed(() => {
    if (listLoading.value) return true;
    return Object.values(serverActions.loading.value).some(Boolean);
  });

  /**
   * 刷新服务器列表
   */
  async function refreshList() {
    error.value = null;
    try {
      servers.value = await withLoading(() => serverApi.getList());
      // 扫描每个服务器的端口信息
      await scanServerPorts();
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  /**
   * 扫描所有服务器的端口信息
   */
  async function scanServerPorts() {
    try {
      for (const server of servers.value) {
        await scanServerPort(server);
      }
    } catch (e) {
      console.warn("Failed to scan server ports:", e);
    }
  }

  /**
   * 扫描单个服务器的端口信息
   */
  async function scanServerPort(server: ServerInstance) {
    try {
      // 使用 configApi 读取 server.properties 文件
      const serverPath = server.path;
      if (!serverPath) return;

      // 读取 server.properties 文件
      const data = await configApi.readServerProperties(serverPath);
      if (data.raw && data.raw["server-port"]) {
        const port = parseInt(data.raw["server-port"]);
        if (!isNaN(port)) {
          server.port = port;
        }
      }
    } catch (e) {
      console.warn(`Failed to scan port for server ${server.id}:`, e);
    }
  }

  /**
   * 刷新指定服务器的状态
   */
  async function refreshStatus(id: string) {
    try {
      statuses.value[id] = await serverApi.getStatus(id);
    } catch (e) {
      console.error(`Failed to get status for server ${id}:`, e);
    }
  }

  /**
   * 批量刷新所有服务器的状态
   */
  async function refreshAllStatuses() {
    const promises = servers.value.map((server) => refreshStatus(server.id));
    await Promise.allSettled(promises);
  }

  /**
   * 设置当前选中的服务器
   */
  function setCurrentServer(id: string | null) {
    currentServerId.value = id;
  }

  /**
   * 根据 ID 获取服务器
   */
  function getServerById(id: string): ServerInstance | null {
    return servers.value.find((s) => s.id === id) || null;
  }

  /**
   * 检查服务器是否正在执行操作
   */
  function isServerLoading(id: string): boolean {
    return serverActions.isLoading(id);
  }

  /**
   * 清除错误状态
   */
  function clearError() {
    error.value = null;
  }

  return {
    servers,
    currentServerId,
    currentServer,
    statuses,
    loading,
    listLoading,
    error,
    serverActions,
    refreshList,
    refreshStatus,
    refreshAllStatuses,
    setCurrentServer,
    getServerById,
    isServerLoading,
    clearError,
    scanServerPorts,
    scanServerPort,
  };
});
