const ALLOWED_RUNTIME_STYLE_PROPS = new Set([
  "align-items",
  "color",
  "display",
  "flex",
  "flex-direction",
  "flex-wrap",
  "font-size",
  "font-weight",
  "gap",
  "justify-content",
  "line-height",
  "margin",
  "margin-bottom",
  "margin-left",
  "margin-right",
  "margin-top",
  "max-width",
  "overflow",
  "overflow-x",
  "overflow-y",
  "padding",
  "padding-bottom",
  "padding-left",
  "padding-right",
  "padding-top",
  "text-align",
  "white-space",
  "word-break",
]);

export const PLUGIN_RUNTIME_HOST_CLASS = "plugin-runtime-host";
export const PLUGIN_RUNTIME_SURFACE_CLASS = "plugin-runtime-surface";
export const PLUGIN_RUNTIME_CHROME_CLASS = "plugin-runtime-chrome";
export const PLUGIN_RUNTIME_CONTENT_CLASS = "plugin-runtime-content";

function ensurePluginRuntimeStyles() {
  let style = document.getElementById("plugin-runtime-host-style") as HTMLStyleElement | null;
  if (style) {
    return;
  }

  style = document.createElement("style");
  style.id = "plugin-runtime-host-style";
  style.textContent = `
    #plugin-ui-container {
      position: fixed;
      right: 12px;
      bottom: 12px;
      width: min(420px, calc(100vw - 24px));
      max-width: calc(100vw - 24px);
      max-height: calc(100vh - 24px);
      display: flex;
      flex-direction: column;
      align-items: stretch;
      justify-content: flex-end;
      gap: 12px;
      pointer-events: none;
      z-index: 30;
      box-sizing: border-box;
      overflow: auto;
      overscroll-behavior: contain;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} {
      width: 100%;
      max-width: 100%;
      max-height: min(360px, calc(100vh - 48px));
      display: flex;
      flex-direction: column;
      align-items: stretch;
      pointer-events: auto;
      box-sizing: border-box;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} {
      width: 100%;
      max-width: 100%;
      max-height: min(360px, calc(100vh - 48px));
      overflow: hidden;
      display: grid;
      grid-template-rows: auto minmax(0, 1fr);
      background: color-mix(in srgb, var(--sl-surface) 92%, var(--sl-primary) 8%);
      color: var(--sl-text-primary);
      border: 1px solid var(--sl-border-light);
      border-style: dashed;
      border-radius: var(--sl-radius-md);
      box-shadow: none;
      box-sizing: border-box;
      isolation: isolate;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} > .${PLUGIN_RUNTIME_CHROME_CLASS} {
      display: flex;
      align-items: center;
      gap: 8px;
      min-height: 36px;
      padding: 10px 12px;
      border-bottom: 1px solid var(--sl-border-light);
      background: color-mix(in srgb, var(--sl-surface) 88%, var(--sl-primary) 12%);
      color: var(--sl-text-secondary);
      font-size: 12px;
      line-height: 1.4;
      user-select: none;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} > .${PLUGIN_RUNTIME_CHROME_CLASS} > .plugin-runtime-label {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-family: var(--sl-font-family-mono, ui-monospace, SFMono-Regular, Consolas, monospace);
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} > .${PLUGIN_RUNTIME_CONTENT_CLASS} {
      min-height: 0;
      overflow: auto;
      padding: 12px;
      box-sizing: border-box;
      contain: content;
      background: color-mix(in srgb, var(--sl-surface) 96%, var(--sl-primary) 4%);
      font-size: 13px;
      line-height: 1.5;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} > .${PLUGIN_RUNTIME_CONTENT_CLASS} [hidden] {
      display: none !important;
    }
  `;
  document.head.appendChild(style);
}

const ALLOWED_RUNTIME_ATTRIBUTES = new Set([
  "aria-label",
  "aria-hidden",
  "aria-pressed",
  "aria-selected",
  "data-state",
  "data-variant",
  "disabled",
  "role",
  "tabindex",
  "title",
  "value",
]);

export function getPluginRootSelector(pluginId: string): string {
  return `[data-plugin-id="${pluginId}"][data-plugin-runtime="content"]`;
}

export function getPluginRuntimeRoots(pluginId: string): HTMLElement[] {
  return Array.from(document.querySelectorAll(getPluginRootSelector(pluginId))) as HTMLElement[];
}

export function resolveScopedTargets(pluginId: string, selector: string): Element[] {
  const roots = getPluginRuntimeRoots(pluginId);
  if (roots.length === 0) {
    return [];
  }

  const targets: Element[] = [];
  for (const root of roots) {
    if (root.matches(selector)) {
      targets.push(root);
    }
    root.querySelectorAll(selector).forEach((element) => {
      targets.push(element);
    });
  }
  return targets;
}

export function resolveScopedTarget(pluginId: string, selector: string): Element | null {
  return resolveScopedTargets(pluginId, selector)[0] ?? null;
}

export function normalizeStyleProperty(property: string): string {
  return property.replace(/([A-Z])/g, "-$1").toLowerCase();
}

export function isAllowedStyleProperty(property: string): boolean {
  const normalized = normalizeStyleProperty(property);
  if (normalized.startsWith("--")) {
    return true;
  }
  return ALLOWED_RUNTIME_STYLE_PROPS.has(normalized);
}

export function isAllowedAttribute(attribute: string): boolean {
  const normalized = attribute.trim().toLowerCase();
  if (normalized.startsWith("aria-")) {
    return true;
  }
  if (normalized.startsWith("data-plugin-")) {
    return true;
  }
  return ALLOWED_RUNTIME_ATTRIBUTES.has(normalized);
}

export function getPluginUiContainer(): HTMLElement {
  let container = document.getElementById("plugin-ui-container");
  if (!container) {
    container = document.createElement("div");
    container.id = "plugin-ui-container";
    ensurePluginRuntimeStyles();
    document.body.appendChild(container);
  }
  return container;
}

export function createPluginRuntimeHost(pluginId: string, elementId: string): HTMLDivElement {
  const host = document.createElement("div");
  host.id = `plugin-ui-${pluginId}-${elementId}`;
  host.className = PLUGIN_RUNTIME_HOST_CLASS;
  host.setAttribute("data-plugin-id", pluginId);
  host.setAttribute("data-plugin-runtime", "host");

  const surface = document.createElement("div");
  surface.className = PLUGIN_RUNTIME_SURFACE_CLASS;
  surface.setAttribute("data-plugin-id", pluginId);
  surface.setAttribute("data-plugin-runtime", "surface");

  const chrome = document.createElement("div");
  chrome.className = PLUGIN_RUNTIME_CHROME_CLASS;
  chrome.setAttribute("data-plugin-id", pluginId);
  chrome.setAttribute("data-plugin-runtime", "chrome");

  const label = document.createElement("span");
  label.className = "plugin-runtime-label";
  label.textContent = pluginId;
  chrome.appendChild(label);

  const content = document.createElement("div");
  content.className = PLUGIN_RUNTIME_CONTENT_CLASS;
  content.setAttribute("data-plugin-id", pluginId);
  content.setAttribute("data-plugin-runtime", "content");
  content.setAttribute("aria-label", `Plugin panel ${pluginId}`);

  surface.appendChild(chrome);
  surface.appendChild(content);
  host.appendChild(surface);

  return host;
}

export function getPluginRuntimeSurface(host: Element | null): HTMLElement | null {
  if (!(host instanceof HTMLElement)) {
    return null;
  }
  return host.querySelector(`.${PLUGIN_RUNTIME_CONTENT_CLASS}`) as HTMLElement | null;
}

export function getScopedRuntimeCssSelector(pluginId: string): string {
  return `#plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS}[data-plugin-id="${pluginId}"] > .${PLUGIN_RUNTIME_SURFACE_CLASS} > .${PLUGIN_RUNTIME_CONTENT_CLASS}`;
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
