import { onActivated, onMounted, watch, type ComputedRef, type Ref } from "vue";

interface UseConfigPageLifecycleOptions {
  routeId: ComputedRef<string>;
  currentServerId: ComputedRef<string | null>;
  serverCount: ComputedRef<number>;
  setCurrentServer: (id: string | null) => void;
  refreshList: () => Promise<void>;
  loadConfigFiles: () => Promise<void>;
  loadProperties: () => Promise<void>;
  loadPlugins: () => Promise<void>;
  compareTargetServerId: Ref<string>;
  compareServerOptions: ComputedRef<Array<{ label: string; value: string }>>;
  hasCompareTargets: Ref<boolean>;
  compareMode: Ref<boolean>;
  loadCompareProperties: () => Promise<void>;
  resetCompareState: (clearTarget?: boolean) => void;
}

export function useConfigPageLifecycle(options: UseConfigPageLifecycleOptions) {
  async function loadCurrentServerConfig() {
    if (!options.currentServerId.value) {
      return;
    }

    await options.loadConfigFiles();
    await Promise.all([options.loadProperties(), options.loadPlugins()]);
  }

  async function initPage() {
    await options.refreshList();

    if (options.routeId.value) {
      options.setCurrentServer(options.routeId.value);
    } else if (!options.currentServerId.value && options.serverCount.value > 0) {
      // 这里保持现有页面的默认行为：由调用方在 refreshList 后决定默认服务器，避免清掉已有选择。
    }

    await loadCurrentServerConfig();
  }

  onMounted(() => {
    void initPage();
  });

  onActivated(() => {
    void loadCurrentServerConfig();
  });

  watch(options.currentServerId, async (serverId) => {
    if (!serverId) {
      return;
    }

    if (options.compareTargetServerId.value === serverId) {
      options.compareTargetServerId.value =
        options.compareServerOptions.value[0]?.value?.toString() || "";
    }

    await loadCurrentServerConfig();
  });

  watch(options.compareTargetServerId, async () => {
    if (options.compareMode.value && options.compareTargetServerId.value) {
      await options.loadCompareProperties();
    }
  });

  watch(options.hasCompareTargets, (hasTargets) => {
    if (!hasTargets) {
      options.resetCompareState(true);
    }
  });
}
