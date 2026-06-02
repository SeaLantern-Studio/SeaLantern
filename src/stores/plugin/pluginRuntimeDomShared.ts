const ALLOWED_RUNTIME_STYLE_PROPS = new Set([
  "align-items",
  "background",
  "background-color",
  "border",
  "border-color",
  "border-radius",
  "border-style",
  "border-width",
  "box-shadow",
  "color",
  "display",
  "flex",
  "flex-direction",
  "font-size",
  "font-weight",
  "gap",
  "justify-content",
  "margin",
  "margin-bottom",
  "margin-left",
  "margin-right",
  "margin-top",
  "max-height",
  "max-width",
  "min-height",
  "min-width",
  "opacity",
  "overflow",
  "padding",
  "padding-bottom",
  "padding-left",
  "padding-right",
  "padding-top",
  "text-align",
  "transform",
  "visibility",
]);

export const PLUGIN_RUNTIME_HOST_CLASS = "plugin-runtime-host";
export const PLUGIN_RUNTIME_SURFACE_CLASS = "plugin-runtime-surface";

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
      top: 0;
      right: 0;
      width: min(420px, calc(100vw - 24px));
      max-width: 100vw;
      height: 100%;
      padding: 12px;
      display: flex;
      flex-direction: column;
      align-items: stretch;
      justify-content: flex-start;
      gap: 12px;
      pointer-events: none;
      z-index: 30;
      box-sizing: border-box;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} {
      width: 100%;
      max-width: 100%;
      max-height: calc(100vh - 24px);
      display: flex;
      flex-direction: column;
      align-items: stretch;
      pointer-events: auto;
      box-sizing: border-box;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} {
      width: 100%;
      max-width: 100%;
      max-height: calc(100vh - 24px);
      overflow: auto;
      background: var(--sl-surface);
      color: var(--sl-text-primary);
      border: 1px solid var(--sl-border-light);
      border-radius: var(--sl-radius-md);
      box-shadow: var(--sl-shadow-lg);
      padding: 12px;
      box-sizing: border-box;
    }

    #plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS} > .${PLUGIN_RUNTIME_SURFACE_CLASS} [hidden] {
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
  return `[data-plugin-id="${pluginId}"]`;
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
  host.appendChild(surface);

  return host;
}

export function getPluginRuntimeSurface(host: Element | null): HTMLElement | null {
  if (!(host instanceof HTMLElement)) {
    return null;
  }
  return host.querySelector(`.${PLUGIN_RUNTIME_SURFACE_CLASS}`) as HTMLElement | null;
}

export function getScopedRuntimeCssSelector(pluginId: string): string {
  return `#plugin-ui-container > .${PLUGIN_RUNTIME_HOST_CLASS}[data-plugin-id="${pluginId}"] > .${PLUGIN_RUNTIME_SURFACE_CLASS}`;
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
