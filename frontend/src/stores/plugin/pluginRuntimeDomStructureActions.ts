import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  cleanupPluginEventListeners,
  removePluginUiElements,
} from "@stores/plugin/pluginRuntimeDomCleanup";
import { scopeRuntimeCss, sanitizeHtml } from "@stores/plugin/pluginRuntimeDomSanitizer";
import {
  createPluginRuntimeHost,
  getPluginUiContainer,
  getPluginRuntimeSurface,
  getScopedRuntimeCssSelector,
  isAllowedStyleProperty,
  normalizeStyleProperty,
  resolveScopedTargets,
} from "@stores/plugin/pluginRuntimeDomShared";
import type {
  PluginRuntimeEventListenerRegistry,
  PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDomTypes";

const BLOCKED_RUNTIME_MUTATION_ACTIONS = new Set([
  "hide",
  "show",
  "disable",
  "enable",
  "insert",
  "remove_selector",
  "set_attribute",
]);

interface HandleRuntimeStructureActionOptions {
  handlePluginRuntimeDomEvent: (
    event: PluginRuntimeUiEvent,
    eventListenerRegistry: PluginRuntimeEventListenerRegistry,
  ) => Promise<boolean>;
}

export async function handlePluginRuntimeStructureAction(
  event: PluginRuntimeUiEvent,
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
  options: HandleRuntimeStructureActionOptions,
): Promise<boolean> {
  const { plugin_id, action, element_id, html, target } = event;
  const fullElementId = `plugin-ui-${plugin_id}-${element_id}`;

  if (BLOCKED_RUNTIME_MUTATION_ACTIONS.has(action)) {
    pluginLogger.warn("RuntimeUI", "已拦截插件界面控制操作", {
      pluginId: plugin_id,
      action,
      target,
    });
    return true;
  }

  switch (action) {
    case "inject": {
      document.getElementById(fullElementId)?.remove();

      const container = getPluginUiContainer();
      const wrapper = createPluginRuntimeHost(plugin_id, element_id);
      const surface = getPluginRuntimeSurface(wrapper);
      if (surface) {
        surface.innerHTML = sanitizeHtml(html);
      }
      container.appendChild(wrapper);
      return true;
    }

    case "remove": {
      document.getElementById(fullElementId)?.remove();
      return true;
    }

    case "update": {
      const element = document.getElementById(fullElementId);
      const surface = getPluginRuntimeSurface(element);
      if (surface) {
        surface.innerHTML = sanitizeHtml(html);
      } else {
        await options.handlePluginRuntimeDomEvent(
          { ...event, action: "inject" },
          eventListenerRegistry,
        );
      }
      return true;
    }

    case "remove_all": {
      removePluginUiElements(plugin_id);
      cleanupPluginEventListeners(plugin_id, eventListenerRegistry);
      return true;
    }

    case "set_style": {
      if (!target) return true;
      try {
        const styles = JSON.parse(html) as Record<string, string>;
        resolveScopedTargets(plugin_id, target).forEach((element) => {
          Object.entries(styles).forEach(([prop, value]) => {
            if (!isAllowedStyleProperty(prop)) {
              pluginLogger.warn("RuntimeUI", "已拦截插件样式属性", {
                pluginId: plugin_id,
                property: prop,
              });
              return;
            }
            (element as HTMLElement).style.setProperty(normalizeStyleProperty(prop), value);
          });
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "插件样式更新内容无效", error);
      }
      return true;
    }

    case "inject_css": {
      const styleId = `plugin-css-${plugin_id}-${element_id}`;
      const css = scopeRuntimeCss(html, getScopedRuntimeCssSelector(plugin_id));
      if (!css.trim()) {
        pluginLogger.warn("RuntimeUI", "已忽略超出白名单的插件样式", {
          pluginId: plugin_id,
          elementId: element_id,
        });
        getPluginUiContainer().querySelector(`#${styleId}`)?.remove();
        return true;
      }

      const pluginRoot = getPluginUiContainer();
      const existingStyle = pluginRoot.querySelector(`#${styleId}`) as HTMLStyleElement | null;
      if (existingStyle) {
        existingStyle.textContent = css;
        existingStyle.setAttribute("data-plugin-id", plugin_id);
        existingStyle.setAttribute("data-plugin-source", "runtime");
        return true;
      }

      const style = document.createElement("style");
      style.id = styleId;
      style.setAttribute("data-plugin-id", plugin_id);
      style.setAttribute("data-plugin-source", "runtime");
      style.textContent = css;
      pluginRoot.appendChild(style);
      return true;
    }

    case "remove_css": {
      getPluginUiContainer().querySelector(`#plugin-css-${plugin_id}-${element_id}`)?.remove();
      return true;
    }

    default:
      return false;
  }
}
