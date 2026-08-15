import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { serverApi } from "@api/server";
import { configApi } from "@api/config";
import { isMockMode } from "@utils/mockData";
import { useAsyncByKey, useLoading } from "@composables/useAsync";
import type { ServerInstance } from "@type/server";
import type { ServerStatusInfo } from "@api/server";

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
      await Promise.all(servers.value.map((server) => scanServerPort(server)));
    } catch (e) {
      console.warn("Failed to scan server ports:", e);
    }
  }

  /**
   * 扫描单个服务器的端口信息
   */
  async function scanServerPort(server: ServerInstance) {
    // mock 服务器的路径是假的,跳过扫描,直接用自带的 port 字段
    if (isMockMode()) return;
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

  // 进行中的状态请求,同 id 并发调用复用同一 Promise 避免重复请求
  const pendingStatusRequests = new Map<string, Promise<ServerStatusInfo | undefined>>();

  /**
   * 刷新指定服务器的状态
   * 同一服务器并发的请求会去重,复用同一个 in-flight Promise
   */
  async function refreshStatus(id: string): Promise<ServerStatusInfo | undefined> {
    const existing = pendingStatusRequests.get(id);
    if (existing) return existing;

    const promise = (async () => {
      try {
        const status = await serverApi.getStatus(id);
        statuses.value[id] = status;
        return status;
      } catch (e) {
        console.error(`Failed to get status for server ${id}:`, e);
        return undefined;
      } finally {
        pendingStatusRequests.delete(id);
      }
    })();

    pendingStatusRequests.set(id, promise);
    return promise;
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
