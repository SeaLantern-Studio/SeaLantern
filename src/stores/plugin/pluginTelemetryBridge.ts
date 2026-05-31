import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ref } from "vue";
import { addPluginTranslations, registerPluginLocale, removePluginTranslations } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginInfo, PluginLogEvent, PluginPermissionLog } from "@type/plugin";

const HIGH_RISK_PERMISSIONS = new Set([
  "network",
  "fs",
  "server",
  "console",
  "element",
  "system",
  "execute_program",
  "plugin_folder_access",
]);

export function createPluginTelemetryBridge(
  getPlugins: () => PluginInfo[],
  isBrowserEnv: () => boolean,
) {
  const permissionLogs = ref<Record<string, PluginPermissionLog[]>>({});
  const pluginLogs = ref<Record<string, PluginLogEvent[]>>({});
  let permissionLogUnlisten: UnlistenFn | null = null;
  let pluginLogUnlisten: UnlistenFn | null = null;
  let i18nEventUnlisten: UnlistenFn | null = null;

  function addPermissionLog(log: PluginPermissionLog) {
    const logs = permissionLogs.value[log.plugin_id] || [];
    const nextLogs = [...logs, log];
    if (nextLogs.length > 500) {
      nextLogs.splice(0, nextLogs.length - 500);
    }
    permissionLogs.value = { ...permissionLogs.value, [log.plugin_id]: nextLogs };
  }

  function addPluginLog(log: PluginLogEvent) {
    const logs = pluginLogs.value[log.plugin_id] || [];
    const nextLogs = [...logs, log];
    if (nextLogs.length > 500) {
      nextLogs.splice(0, nextLogs.length - 500);
    }
    pluginLogs.value = { ...pluginLogs.value, [log.plugin_id]: nextLogs };
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

  function getHighRiskPermissions(pluginId: string): string[] {
    const plugin = getPlugins().find((item) => item.manifest.id === pluginId);
    if (!plugin?.manifest.permissions) {
      return [];
    }
    return plugin.manifest.permissions.filter((permission) =>
      HIGH_RISK_PERMISSIONS.has(permission),
    );
  }

  async function initPermissionLogListener() {
    if (isBrowserEnv() || permissionLogUnlisten) {
      return;
    }
    try {
      permissionLogUnlisten = await listen<PluginPermissionLog>(
        "plugin-permission-log",
        (event) => {
          const log = event.payload;
          addPermissionLog(log);
          pluginLogger.info("Permission", `${log.plugin_id} ${log.log_type}:${log.action}`);
        },
      );
      pluginLogger.info("Permission", "权限记录监听已就绪");
    } catch (error) {
      pluginLogger.error("Permission", "权限记录监听初始化失败", error);
    }
  }

  function cleanupPermissionLogListener() {
    permissionLogUnlisten?.();
    permissionLogUnlisten = null;
  }

  async function initPluginLogListener() {
    if (isBrowserEnv() || pluginLogUnlisten) {
      return;
    }
    try {
      pluginLogUnlisten = await listen<PluginLogEvent>("plugin-log-event", (event) => {
        addPluginLog(event.payload);
      });
      pluginLogger.info("Log", "插件日志监听已就绪");
    } catch (error) {
      pluginLogger.error("Log", "插件日志监听初始化失败", error);
    }
  }

  function cleanupPluginLogListener() {
    pluginLogUnlisten?.();
    pluginLogUnlisten = null;
  }

  async function initI18nEventListener() {
    if (isBrowserEnv() || i18nEventUnlisten) {
      return;
    }
    try {
      i18nEventUnlisten = await listen<{
        plugin_id: string;
        action: string;
        locale: string;
        payload: string;
      }>("plugin-i18n-event", (event) => {
        const { plugin_id, action, locale, payload } = event.payload;
        if (action === "register_locale") {
          const { displayName } = JSON.parse(payload || "{}");
          registerPluginLocale(locale, displayName || locale);
          return;
        }
        if (action === "add_translations") {
          addPluginTranslations(plugin_id, locale, JSON.parse(payload || "{}"));
          return;
        }
        if (action === "remove_translations") {
          removePluginTranslations(plugin_id);
        }
      });
      pluginLogger.info("I18n", "插件语言监听已就绪");
    } catch (error) {
      pluginLogger.error("I18n", "插件语言监听初始化失败", error);
    }
  }

  function cleanupI18nEventListener() {
    i18nEventUnlisten?.();
    i18nEventUnlisten = null;
  }

  function hydratePermissionLogs(entries: Record<string, PluginPermissionLog[]>) {
    permissionLogs.value = { ...permissionLogs.value, ...entries };
  }

  function slicePermissionLogs(logs: PluginPermissionLog[]): PluginPermissionLog[] {
    return logs.slice(-500);
  }

  return {
    permissionLogs,
    pluginLogs,
    getPluginLogs,
    getPermissionLogs,
    clearPermissionLogs,
    getHighRiskPermissions,
    initPermissionLogListener,
    cleanupPermissionLogListener,
    initPluginLogListener,
    cleanupPluginLogListener,
    initI18nEventListener,
    cleanupI18nEventListener,
    hydratePermissionLogs,
    slicePermissionLogs,
  };
}
