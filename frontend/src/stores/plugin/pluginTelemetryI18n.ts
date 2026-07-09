import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { addPluginTranslations, registerPluginLocale, removePluginTranslations } from "@language";
import { pluginLogger } from "@stores/plugin/pluginLogger";

interface PluginI18nEvent {
  plugin_id: string;
  action: string;
  locale: string;
  payload: string;
}

interface PluginTelemetryI18nOptions {
  isBrowserEnv: () => boolean;
}

function parseI18nPayload(payload: string, action: string) {
  try {
    return JSON.parse(payload || "{}");
  } catch (error) {
    pluginLogger.error("I18n", `插件语言事件解析失败: ${action}`, error);
    return null;
  }
}

export function createPluginTelemetryI18n(options: PluginTelemetryI18nOptions) {
  let i18nEventUnlisten: UnlistenFn | null = null;

  async function initI18nEventListener() {
    if (options.isBrowserEnv() || i18nEventUnlisten) {
      return;
    }

    try {
      i18nEventUnlisten = await listen<PluginI18nEvent>("plugin-i18n-event", (event) => {
        const { plugin_id, action, locale, payload } = event.payload;

        if (action === "register_locale") {
          const data = parseI18nPayload(payload, action);
          if (!data) {
            return;
          }

          registerPluginLocale(locale, data.displayName || locale);
          return;
        }

        if (action === "add_translations") {
          const data = parseI18nPayload(payload, action);
          if (!data) {
            return;
          }

          addPluginTranslations(plugin_id, locale, data);
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

  return {
    initI18nEventListener,
    cleanupI18nEventListener,
  };
}
