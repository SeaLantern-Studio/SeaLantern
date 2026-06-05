import { ref } from "vue";
import type { PluginLogEvent, PluginPermissionLog } from "@type/plugin";

const LOG_LIMIT = 500;

function sliceLogs<T>(logs: T[]): T[] {
  return logs.slice(-LOG_LIMIT);
}

export function createPluginTelemetryState() {
  const permissionLogs = ref<Record<string, PluginPermissionLog[]>>({});
  const pluginLogs = ref<Record<string, PluginLogEvent[]>>({});

  function addPermissionLog(log: PluginPermissionLog) {
    const logs = permissionLogs.value[log.plugin_id] || [];
    permissionLogs.value = {
      ...permissionLogs.value,
      [log.plugin_id]: sliceLogs([...logs, log]),
    };
  }

  function addPluginLog(log: PluginLogEvent) {
    const logs = pluginLogs.value[log.plugin_id] || [];
    pluginLogs.value = {
      ...pluginLogs.value,
      [log.plugin_id]: sliceLogs([...logs, log]),
    };
  }

  function getPluginLogs(pluginId: string): PluginLogEvent[] {
    return pluginLogs.value[pluginId] || [];
  }

  function getPermissionLogs(pluginId: string): PluginPermissionLog[] {
    return permissionLogs.value[pluginId] || [];
  }

  function clearPermissionLogs(pluginId: string) {
    delete permissionLogs.value[pluginId];
  }

  function hydratePermissionLogs(entries: Record<string, PluginPermissionLog[]>) {
    permissionLogs.value = { ...permissionLogs.value, ...entries };
  }

  function slicePermissionLogs(logs: PluginPermissionLog[]): PluginPermissionLog[] {
    return sliceLogs(logs);
  }

  return {
    permissionLogs,
    pluginLogs,
    addPermissionLog,
    addPluginLog,
    getPluginLogs,
    getPermissionLogs,
    clearPermissionLogs,
    hydratePermissionLogs,
    slicePermissionLogs,
  };
}
