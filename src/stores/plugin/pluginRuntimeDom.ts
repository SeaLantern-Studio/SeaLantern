import { emit } from "@tauri-apps/api/event";
import DOMPurify from "dompurify";
import { pluginLogger } from "@stores/plugin/pluginLogger";
import type { PluginUiAction } from "@type/plugin";

export interface PluginRuntimeUiEvent {
  plugin_id: string;
  action: PluginUiAction;
  element_id: string;
  html: string;
  target?: string;
}

export type PluginRuntimeEventListenerRegistry = Map<
  string,
  Array<{ element: Element; eventType: string; handler: EventListener }>
>;

export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    FORBID_TAGS: ["script", "iframe", "object", "embed", "form", "link", "meta", "style"],
    FORBID_ATTR: ["style"],
    ALLOW_DATA_ATTR: false,
  });
}

export function sanitizeCss(css: string): string {
  let sanitized = css.replace(/@import\s+[^;]+;/gi, "");
  sanitized = sanitized.replace(
    /url\s*\(\s*(['"]?)\s*(https?:\/\/|\/\/)[^)]*\1\s*\)/gi,
    "url(about:blank)",
  );
  sanitized = sanitized.replace(/url\s*\(\s*(['"]?)\s*data:[^)]*\1\s*\)/gi, "url(about:blank)");
  sanitized = sanitized.replace(/expression\s*\(/gi, "(");
  sanitized = sanitized.replace(/-moz-binding\s*:/gi, ":");
  return sanitized;
}

export function executePluginScripts(container: HTMLElement, rawHtml: string) {
  const scriptRegex = /<script\b[^>]*>([\s\S]*?)<\/script>/gi;
  let match: RegExpExecArray | null;
  while ((match = scriptRegex.exec(rawHtml)) !== null) {
    const scriptContent = match[1]?.trim();
    if (!scriptContent) {
      continue;
    }
    try {
      const scriptEl = document.createElement("script");
      scriptEl.textContent = scriptContent;
      container.appendChild(scriptEl);
    } catch (error) {
      pluginLogger.error("RuntimeUI", "插件脚本执行失败", error);
    }
  }
}

export function getPluginUiContainer(): HTMLElement {
  let container = document.getElementById("plugin-ui-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "plugin-ui-container";
    container.style.cssText =
      "position: fixed; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 9998;";
    document.body.appendChild(container);
  }
  return container;
}

export function removePluginUiElements(pluginId: string) {
  const elements = document.querySelectorAll(`[data-plugin-id="${pluginId}"]`);
  elements.forEach((element) => element.remove());

  const insertedElements = document.querySelectorAll(`[data-plugin-inserted="${pluginId}"]`);
  insertedElements.forEach((element) => element.remove());

  const hiddenElements = document.querySelectorAll(`[data-plugin-hidden="${pluginId}"]`);
  hiddenElements.forEach((element) => {
    (element as HTMLElement).style.display = "";
    delete (element as HTMLElement).dataset.pluginHidden;
  });

  const disabledElements = document.querySelectorAll(`[data-plugin-disabled="${pluginId}"]`);
  disabledElements.forEach((element) => {
    (element as HTMLElement).removeAttribute("disabled");
    (element as HTMLElement).style.pointerEvents = "";
    (element as HTMLElement).style.opacity = "";
    delete (element as HTMLElement).dataset.pluginDisabled;
  });

  pluginLogger.info("RuntimeUI", `已清理插件界面元素: ${pluginId}`, {
    removed: elements.length + insertedElements.length,
  });
}

export function cleanupPluginEventListeners(
  pluginId: string,
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
) {
  const listeners = eventListenerRegistry.get(pluginId);
  if (!listeners) {
    return;
  }

  for (const { element, eventType, handler } of listeners) {
    element.removeEventListener(eventType, handler);
  }
  eventListenerRegistry.delete(pluginId);
  pluginLogger.info("RuntimeUI", `已清理插件事件监听: ${pluginId}`, {
    count: listeners.length,
  });
}

export function setFormFieldValue(field: Element, value: unknown) {
  if (field instanceof HTMLInputElement) {
    const type = field.type.toLowerCase();
    if (type === "checkbox") {
      field.checked = Boolean(value);
      field.dispatchEvent(new Event("change", { bubbles: true }));
      return true;
    }
    if (type === "radio") {
      const normalized = value == null ? "" : String(value);
      if (field.value === normalized) {
        field.checked = true;
        field.dispatchEvent(new Event("change", { bubbles: true }));
        return true;
      }
      return false;
    }

    field.value = value == null ? "" : String(value);
    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  }

  if (field instanceof HTMLTextAreaElement) {
    field.value = value == null ? "" : String(value);
    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  }

  if (field instanceof HTMLSelectElement) {
    field.value = value == null ? "" : String(value);
    field.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  }

  return false;
}

export async function handlePluginRuntimeDomEvent(
  event: PluginRuntimeUiEvent,
  eventListenerRegistry: PluginRuntimeEventListenerRegistry,
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
        await handlePluginRuntimeDomEvent({ ...event, action: "inject" }, eventListenerRegistry);
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
      document.querySelectorAll(target).forEach((element) => {
        (element as HTMLElement).style.display = "none";
        (element as HTMLElement).dataset.pluginHidden = plugin_id;
      });
      return true;
    }

    case "show": {
      if (!target) return true;
      document.querySelectorAll(target).forEach((element) => {
        (element as HTMLElement).style.display = "";
        delete (element as HTMLElement).dataset.pluginHidden;
      });
      return true;
    }

    case "disable": {
      if (!target) return true;
      document.querySelectorAll(target).forEach((element) => {
        (element as HTMLElement).setAttribute("disabled", "true");
        (element as HTMLElement).style.pointerEvents = "none";
        (element as HTMLElement).style.opacity = "0.5";
        (element as HTMLElement).dataset.pluginDisabled = plugin_id;
      });
      return true;
    }

    case "enable": {
      if (!target) return true;
      document.querySelectorAll(target).forEach((element) => {
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
      const targetElement = selector ? document.querySelector(selector) : null;
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
      document.querySelectorAll(target).forEach((element) => {
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
        document.querySelectorAll(target).forEach((element) => {
          Object.entries(styles).forEach(([prop, value]) => {
            (element as HTMLElement).style.setProperty(
              prop.replace(/([A-Z])/g, "-$1").toLowerCase(),
              value,
            );
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
        document.querySelectorAll(target).forEach((element) => {
          element.setAttribute(attribute, value ?? "");
        });
      } catch (error) {
        pluginLogger.error("RuntimeUI", "插件属性更新内容无效", error);
      }
      return true;
    }

    case "query": {
      if (!target) return true;
      const results = Array.from(document.querySelectorAll(target)).map((element) => ({
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
          data: document.querySelector(target) !== null ? "true" : "false",
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
        const element = document.querySelector(target) as HTMLElement | null;
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
        const element = document.querySelector(target) as HTMLElement | null;
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
        const element = document.querySelector(target);
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
        const element = document.querySelector(target) as
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
        const element = document.querySelector(target);
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
        const element = document.querySelector(target);
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
      (document.querySelector(target) as HTMLElement | null)?.click();
      return true;
    }

    case "element_set_value": {
      if (!target) return true;
      const element = document.querySelector(target) as HTMLInputElement | null;
      if (!element) return true;
      try {
        const { value } = JSON.parse(html || "{}") as { value?: string };
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
      const element = document.querySelector(target) as HTMLInputElement | null;
      if (!element) return true;
      try {
        const { checked } = JSON.parse(html || "{}") as { checked?: boolean };
        element.checked = Boolean(checked);
        element.dispatchEvent(new Event("change", { bubbles: true }));
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素勾选内容无效", error);
      }
      return true;
    }

    case "element_select": {
      if (!target) return true;
      const element = document.querySelector(target) as HTMLSelectElement | null;
      if (!element) return true;
      try {
        const { value } = JSON.parse(html || "{}") as { value?: string };
        element.value = value ?? "";
        element.dispatchEvent(new Event("change", { bubbles: true }));
      } catch (error) {
        pluginLogger.error("RuntimeUI", "元素选择内容无效", error);
      }
      return true;
    }

    case "element_focus": {
      if (!target) return true;
      (document.querySelector(target) as HTMLElement | null)?.focus();
      return true;
    }

    case "element_blur": {
      if (!target) return true;
      (document.querySelector(target) as HTMLElement | null)?.blur();
      return true;
    }

    case "element_on_change": {
      if (!target) return true;
      const element = document.querySelector(target) as HTMLElement | null;
      if (!element) return true;

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
        const form = document.querySelector(target);
        const fields = payload.fields;
        if (!form || !fields || typeof fields !== "object") {
          return true;
        }

        Object.entries(fields).forEach(([name, value]) => {
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

    case "inject_css": {
      const styleId = `plugin-css-${plugin_id}-${element_id}`;
      const css = sanitizeCss(html);
      const existingStyle = document.getElementById(styleId) as HTMLStyleElement | null;
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
      document.head.appendChild(style);
      return true;
    }

    case "remove_css": {
      document.getElementById(`plugin-css-${plugin_id}-${element_id}`)?.remove();
      return true;
    }

    default:
      return false;
  }
}
