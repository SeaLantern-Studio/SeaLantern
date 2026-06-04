import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginLogEvent, PluginPermissionLog } from "@type/plugin";

interface PluginTelemetryListenersOptions {
  isBrowserEnv: () => boolean;
  addPermissionLog: (log: PluginPermissionLog) => void;
  addPluginLog: (log: PluginLogEvent) => void;
}

export function createPluginTelemetryListeners(options: PluginTelemetryListenersOptions) {
  let permissionLogUnlisten: UnlistenFn | null = null;
  let pluginLogUnlisten: UnlistenFn | null = null;

  async function initPermissionLogListener() {
    if (options.isBrowserEnv() || permissionLogUnlisten) {
      return;
    }

    try {
      permissionLogUnlisten = await listen<PluginPermissionLog>(
        "plugin-permission-log",
        (event) => {
          const log = event.payload;
          options.addPermissionLog(log);
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
    if (options.isBrowserEnv() || pluginLogUnlisten) {
      return;
    }

    try {
      pluginLogUnlisten = await listen<PluginLogEvent>("plugin-log-event", (event) => {
        options.addPluginLog(event.payload);
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

  return {
    initPermissionLogListener,
    cleanupPermissionLogListener,
    initPluginLogListener,
    cleanupPluginLogListener,
  };
}
