import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  cleanupPluginEventListeners,
  removePluginUiElements,
} from "@stores/plugin/pluginRuntimeDomCleanup";
import {
  executePluginScripts,
  sanitizeCss,
  sanitizeHtml,
} from "@stores/plugin/pluginRuntimeDomSanitizer";
import {
  getPluginUiContainer,
  isAllowedAttribute,
  isAllowedStyleProperty,
  normalizeStyleProperty,
  resolveScopedTarget,
  resolveScopedTargets,
} from "@stores/plugin/pluginRuntimeDomShared";
import type {
  PluginRuntimeEventListenerRegistry,
  PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDomTypes";

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

  switch (action) {
    case "inject": {
      document.getElementById(fullElementId)?.remove();

      const container = getPluginUiContainer();
      const wrapper = document.createElement("div");
      wrapper.id = fullElementId;
      wrapper.setAttribute("data-plugin-id", plugin_id);
      wrapper.style.pointerEvents = "auto";
      wrapper.innerHTML = sanitizeHtml(html);
      container.appendChild(wrapper);
      executePluginScripts(wrapper, html);
      return true;
    }

    case "remove": {
      document.getElementById(fullElementId)?.remove();
      return true;
    }

    case "update": {
      const element = document.getElementById(fullElementId);
      if (element) {
        element.innerHTML = sanitizeHtml(html);
        executePluginScripts(element, html);
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

    case "hide": {
      if (!target) return true;
      resolveScopedTargets(plugin_id, target).forEach((element) => {
        (element as HTMLElement).style.display = "none";
        (element as HTMLElement).dataset.pluginHidden = plugin_id;
      });
      return true;
    }

    case "show": {
      if (!target) return true;
      resolveScopedTargets(plugin_id, target).forEach((element) => {
        (element as HTMLElement).style.display = "";
        delete (element as HTMLElement).dataset.pluginHidden;
      });
      return true;
    }

    case "disable": {
      if (!target) return true;
      resolveScopedTargets(plugin_id, target).forEach((element) => {
        (element as HTMLElement).setAttribute("disabled", "true");
        (element as HTMLElement).style.pointerEvents = "none";
        (element as HTMLElement).style.opacity = "0.5";
        (element as HTMLElement).dataset.pluginDisabled = plugin_id;
      });
      return true;
    }

    case "enable": {
      if (!target) return true;
      resolveScopedTargets(plugin_id, target).forEach((element) => {
        (element as HTMLElement).removeAttribute("disabled");
        (element as HTMLElement).style.pointerEvents = "";
        (element as HTMLElement).style.opacity = "";
        delete (element as HTMLElement).dataset.pluginDisabled;
      });
      return true;
    }

    case "insert": {
      if (!target) return true;
      const [placement, selector] = target.split("|");
      const targetElement = selector ? resolveScopedTarget(plugin_id, selector) : null;
      if (!targetElement) return true;

      const wrapper = document.createElement("div");
      wrapper.dataset.pluginInserted = plugin_id;
      wrapper.innerHTML = sanitizeHtml(html);
      executePluginScripts(wrapper, html);

      switch (placement) {
        case "before":
          targetElement.parentNode?.insertBefore(wrapper, targetElement);
          break;
        case "after":
          targetElement.parentNode?.insertBefore(wrapper, targetElement.nextSibling);
          break;
        case "prepend":
          targetElement.prepend(wrapper);
          break;
        case "append":
          targetElement.append(wrapper);
          break;
      }
      return true;
    }

    case "remove_selector": {
      if (!target) return true;
      resolveScopedTargets(plugin_id, target).forEach((element) => {
        if ((element as HTMLElement).dataset.pluginInserted === plugin_id) {
          element.remove();
        }
      });

      document.querySelectorAll(`[data-plugin-inserted="${plugin_id}"]`).forEach((element) => {
        if (element.querySelector(target) || element.matches(target)) {
          element.remove();
        }
      });
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

    case "set_attribute": {
      if (!target) return true;
      try {
        const { attribute, value } = JSON.parse(html || "{}") as {
          attribute?: string;
          value?: string;
        };
        if (!attribute) return true;
        if (!isAllowedAttribute(attribute)) {
          pluginLogger.warn("RuntimeUI", "已拦截插件属性更新", {
            pluginId: plugin_id,
            attribute,
          });
          return true;
        }
        resolveScopedTargets(plugin_id, target).forEach((element) => {
          element.setAttribute(attribute, value ?? "");
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "插件属性更新内容无效", error);
      }
      return true;
    }

    case "inject_css": {
      const styleId = `plugin-css-${plugin_id}-${element_id}`;
      const css = sanitizeCss(html);
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
