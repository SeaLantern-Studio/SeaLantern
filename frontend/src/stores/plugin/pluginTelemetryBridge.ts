import type { PluginInfo } from "@type/plugin";
import { createPluginTelemetryI18n } from "./pluginTelemetryI18n";
import { createPluginTelemetryListeners } from "./pluginTelemetryListeners";
import { createPluginTelemetryState } from "./pluginTelemetryState";
import { getPluginHighRiskPermissions } from "./pluginTelemetryShared";

export function createPluginTelemetryBridge(
  getPlugins: () => PluginInfo[],
  isBrowserEnv: () => boolean,
) {
  const state = createPluginTelemetryState();
  const listeners = createPluginTelemetryListeners({
    isBrowserEnv,
    addPermissionLog: state.addPermissionLog,
    addPluginLog: state.addPluginLog,
  });
  const i18n = createPluginTelemetryI18n({ isBrowserEnv });

  return {
    permissionLogs: state.permissionLogs,
    pluginLogs: state.pluginLogs,
    getPluginLogs: state.getPluginLogs,
    getPermissionLogs: state.getPermissionLogs,
    clearPermissionLogs: state.clearPermissionLogs,
    getHighRiskPermissions: (pluginId: string) =>
      getPluginHighRiskPermissions(getPlugins, pluginId),
    initPermissionLogListener: listeners.initPermissionLogListener,
    cleanupPermissionLogListener: listeners.cleanupPermissionLogListener,
    initPluginLogListener: listeners.initPluginLogListener,
    cleanupPluginLogListener: listeners.cleanupPluginLogListener,
    initI18nEventListener: i18n.initI18nEventListener,
    cleanupI18nEventListener: i18n.cleanupI18nEventListener,
    hydratePermissionLogs: state.hydratePermissionLogs,
    slicePermissionLogs: state.slicePermissionLogs,
  };
}
