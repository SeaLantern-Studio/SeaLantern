import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  cleanupPluginEventListeners,
  removePluginUiElements,
} from "@stores/plugin/pluginRuntimeDomCleanup";
import {
  executePluginScripts,
  scopeRuntimeCss,
  sanitizeHtml,
} from "@stores/plugin/pluginRuntimeDomSanitizer";
import {
  createPluginRuntimeHost,
  getPluginUiContainer,
  getPluginRuntimeSurface,
  getScopedRuntimeCssSelector,
  isAllowedStyleProperty,
  normalizeStyleProperty,
  resolveScopedTargets,
  setPluginRuntimeHostPlain,
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
  // Structure actions own runtime-host lifecycle and CSS/script injection. They are
  // deliberately narrower than historical plugin UI mutations because unrestricted
  // selector-based DOM edits made plugin surfaces bleed outside their sandbox.
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
      // Re-injection always drops the previous host first so a plugin cannot accumulate
      // multiple runtime roots with the same logical element id across refreshes.
      document.getElementById(fullElementId)?.remove();

      const container = getPluginUiContainer();
      const wrapper = createPluginRuntimeHost(plugin_id, element_id);
      const surface = getPluginRuntimeSurface(wrapper);
      const containsScript = /<script\b/i.test(html);
      // Plain-host mode is recorded before HTML assignment because styling and cleanup logic
      // later needs to know whether this surface came from script-bearing runtime content.
      setPluginRuntimeHostPlain(wrapper, containsScript);
      if (surface) {
        surface.innerHTML = sanitizeHtml(html);
        executePluginScripts(surface, html);
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
        const containsScript = /<script\b/i.test(html);
        if (element instanceof HTMLElement) {
          setPluginRuntimeHostPlain(element, containsScript);
        }
        surface.innerHTML = sanitizeHtml(html);
        executePluginScripts(surface, html);
      } else {
        // Treat updates for a missing host as idempotent recovery. Runtime plugins can
        // race with route transitions or cleanup, and falling back to inject avoids a
        // permanent blank surface after the first missed mount.
        await options.handlePluginRuntimeDomEvent(
          { ...event, action: "inject" },
          eventListenerRegistry,
        );
      }
      return true;
    }

    case "remove_all": {
      // Remove DOM nodes before listener cleanup so detach-time selectors cannot accidentally
      // bind to a freshly re-created subtree during the same plugin teardown sequence.
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
            // Property names are normalized only after the whitelist check so aliases map to
            // safe CSS properties without silently broadening the allowed surface.
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
        // Replace-or-remove keeps the plugin's style slot deterministic. When scoping
        // strips everything, we drop the old node instead of leaving stale CSS active.
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
        // Reuse the existing style node to preserve DOM order and avoid churn for plugins
        // that update runtime CSS frequently while the page remains mounted.
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
