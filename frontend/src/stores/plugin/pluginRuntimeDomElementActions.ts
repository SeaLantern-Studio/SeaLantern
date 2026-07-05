import { emit } from "@tauri-apps/api/event";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import {
  resolveScopedTarget,
  resolveScopedTargets,
  setFormFieldValue,
} from "@stores/plugin/pluginRuntimeDomShared";
import type {
  PluginRuntimeEventListenerRegistry,
  PluginRuntimeUiEvent,
} from "@stores/plugin/pluginRuntimeDomTypes";

export async function handlePluginRuntimeElementAction(
  event: PluginRuntimeUiEvent,
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
): Promise<boolean> {
  // This bridge accepts runtime UI commands from plugins and translates them into
  // tightly scoped DOM reads/writes. Returning true means the action name belongs
  // to the element-action surface even when the concrete target/payload is invalid.
  const { plugin_id, action, html, target } = event;

  switch (action) {
    case "query": {
      if (!target) return true;
      // Query responses are normalized into a small transport-friendly snapshot so the
      // plugin runtime never receives live DOM handles and large text nodes do not flood
      // the Tauri event channel.
      const results = resolveScopedTargets(plugin_id, target).map((element) => ({
        id: element.id || "",
        tag: element.tagName.toLowerCase(),
        classes: Array.from(element.classList),
        text: (element as HTMLElement).innerText?.substring(0, 500) || "",
        visible: (element as HTMLElement).offsetParent !== null,
        enabled: !(element as HTMLElement).hasAttribute("disabled"),
      }));
      await emit("plugin-ui-query-result", { plugin_id, elements: results });
      return true;
    }

    case "element_exists": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string };
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: resolveScopedTarget(plugin_id, target) !== null ? "true" : "false",
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素存在性查询失败", error);
      }
      return true;
    }

    case "element_is_visible": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string };
        const element = resolveScopedTarget(plugin_id, target) as HTMLElement | null;
        const isVisible =
          !!element &&
          element.isConnected &&
          !!(element.offsetWidth || element.offsetHeight || element.getClientRects().length);
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: isVisible ? "true" : "false",
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素可见性查询失败", error);
      }
      return true;
    }

    case "element_is_enabled": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string };
        const element = resolveScopedTarget(plugin_id, target) as HTMLElement | null;
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: !!element && !element.hasAttribute("disabled") ? "true" : "false",
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素可用性查询失败", error);
      }
      return true;
    }

    case "element_get_text": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string };
        const element = resolveScopedTarget(plugin_id, target);
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: element ? (element as HTMLElement).innerText || "" : "",
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素文本读取失败", error);
      }
      return true;
    }

    case "element_get_value": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string };
        const element = resolveScopedTarget(plugin_id, target) as
          | HTMLInputElement
          | HTMLSelectElement
          | HTMLTextAreaElement
          | null;
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: element?.value || "",
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素值读取失败", error);
      }
      return true;
    }

    case "element_get_attribute": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string; attr?: string };
        const element = resolveScopedTarget(plugin_id, target);
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: parsed.attr && element ? element.getAttribute(parsed.attr) || "" : "",
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素属性读取失败", error);
      }
      return true;
    }

    case "element_get_attributes": {
      if (!target) return true;
      try {
        const parsed = JSON.parse(html || "{}") as { request_id?: string };
        const element = resolveScopedTarget(plugin_id, target);
        const attrs: Record<string, string> = {};
        if (element) {
          Array.from(element.attributes).forEach((attr) => {
            attrs[attr.name] = attr.value;
          });
        }
        await emit("plugin-element-response", {
          plugin_id,
          request_id: parsed.request_id,
          data: JSON.stringify(attrs),
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素属性列表读取失败", error);
      }
      return true;
    }

    case "element_click": {
      if (!target) return true;
      (resolveScopedTarget(plugin_id, target) as HTMLElement | null)?.click();
      return true;
    }

    case "element_set_value": {
      if (!target) return true;
      const element = resolveScopedTarget(plugin_id, target) as HTMLInputElement | null;
      if (!element) return true;
      try {
        const { value } = JSON.parse(html || "{}") as { value?: string };
        // Dispatch both input and change so framework-managed controls and plain DOM forms
        // observe the same mutation. Some pages only react to one of the two events.
        element.value = value ?? "";
        element.dispatchEvent(new Event("input", { bubbles: true }));
        element.dispatchEvent(new Event("change", { bubbles: true }));
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素值写入内容无效", error);
      }
      return true;
    }

    case "element_check": {
      if (!target) return true;
      const element = resolveScopedTarget(plugin_id, target) as HTMLInputElement | null;
      if (!element) return true;
      try {
        const { checked } = JSON.parse(html || "{}") as { checked?: boolean };
        // Checkbox/radio toggles intentionally emit change only; browsers do not fire a
        // meaningful input event for every control type, and the host code listens on change.
        element.checked = Boolean(checked);
        element.dispatchEvent(new Event("change", { bubbles: true }));
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素勾选内容无效", error);
      }
      return true;
    }

    case "element_select": {
      if (!target) return true;
      const element = resolveScopedTarget(plugin_id, target) as HTMLSelectElement | null;
      if (!element) return true;
      try {
        const { value } = JSON.parse(html || "{}") as { value?: string };
        // Select controls also use change as the compatibility signal because Vue/native
        // forms in this codebase both commit selection updates from that event.
        element.value = value ?? "";
        element.dispatchEvent(new Event("change", { bubbles: true }));
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素选择内容无效", error);
      }
      return true;
    }

    case "element_focus": {
      if (!target) return true;
      (resolveScopedTarget(plugin_id, target) as HTMLElement | null)?.focus();
      return true;
    }

    case "element_blur": {
      if (!target) return true;
      (resolveScopedTarget(plugin_id, target) as HTMLElement | null)?.blur();
      return true;
    }

    case "element_on_change": {
      if (!target) return true;
      const element = resolveScopedTarget(plugin_id, target) as HTMLElement | null;
      if (!element) return true;

      // Replace any existing listener for the same plugin/element pair before adding
      // the new one. Plugins re-register handlers after rerenders, and leaking the old
      // bridge would duplicate callback traffic back into the runtime.
      const listeners = eventListenerRegistry.get(plugin_id) ?? [];
      listeners
        .filter((entry) => entry.eventType === "change" && entry.element === element)
        .forEach((entry) => {
          entry.element.removeEventListener(entry.eventType, entry.handler);
        });

      const nextListeners = listeners.filter(
        (entry) => !(entry.eventType === "change" && entry.element === element),
      );

      const handler = (domEvent: Event) => {
        const value = (
          domEvent.target as HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement
        ).value;
        void emit("plugin-element-change", {
          plugin_id,
          selector: target,
          value,
        });
      };

      element.addEventListener("change", handler);
      nextListeners.push({ element, eventType: "change", handler });
      eventListenerRegistry.set(plugin_id, nextListeners);
      return true;
    }

    case "element_off_change": {
      if (!target) return true;
      const listeners = eventListenerRegistry.get(plugin_id);
      if (!listeners) return true;

      // Use selector matching here instead of strict element identity so plugins can tear
      // down listeners after a rerender even when the original node has already been replaced.
      const remaining = listeners.filter((entry) => {
        const shouldRemove = entry.eventType === "change" && entry.element.matches(target);
        if (shouldRemove) {
          entry.element.removeEventListener(entry.eventType, entry.handler);
        }
        return !shouldRemove;
      });

      if (remaining.length === 0) {
        eventListenerRegistry.delete(plugin_id);
      } else {
        eventListenerRegistry.set(plugin_id, remaining);
      }
      return true;
    }

    case "element_form_fill": {
      if (!target) return true;
      try {
        const payload = JSON.parse(html || "{}") as { fields?: Record<string, unknown> };
        const form = resolveScopedTarget(plugin_id, target);
        const fields = payload.fields;
        if (!form || !fields || typeof fields !== "object") {
          return true;
        }

        Object.entries(fields).forEach(([name, value]) => {
          // Prefer CSS.escape when available so plugin-provided field names cannot break
          // selector parsing. The string fallback only preserves compatibility with older
          // runtimes that do not expose CSS.escape.
          const cssApi = window as Window & { CSS?: { escape?: (input: string) => string } };
          const escapedName = cssApi.CSS?.escape?.(name);
          const selector = escapedName
            ? `[name="${escapedName}"]`
            : `[name="${name.replace(/"/g, '\\"')}"]`;
          const matches = Array.from(form.querySelectorAll(selector));

          if (matches.length === 0) {
            return;
          }

          if (Array.isArray(value)) {
            matches.forEach((field) => {
              if (field instanceof HTMLInputElement && field.type.toLowerCase() === "checkbox") {
                field.checked = value.some((item) => String(item) === field.value);
                field.dispatchEvent(new Event("change", { bubbles: true }));
              }
            });
            return;
          }

          if (
            matches.some(
              (field) => field instanceof HTMLInputElement && field.type.toLowerCase() === "radio",
            )
          ) {
            // Radio groups must be processed as a set so the browser can clear the old
            // selection consistently; treating them like a single field leaves stale state.
            matches.forEach((field) => {
              setFormFieldValue(field, value);
            });
            return;
          }

          setFormFieldValue(matches[0], value);
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "表单填充失败", error);
      }
      return true;
    }

    default:
      return false;
  }
}
