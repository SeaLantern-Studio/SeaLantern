const ALLOWED_RUNTIME_STYLE_PROPS = new Set([
  "align-items",
  "background",
  "background-color",
  "border",
  "border-color",
  "border-radius",
  "border-style",
  "border-width",
  "bottom",
  "box-shadow",
  "color",
  "display",
  "flex",
  "flex-direction",
  "font-size",
  "font-weight",
  "gap",
  "height",
  "justify-content",
  "left",
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
  "pointer-events",
  "position",
  "right",
  "text-align",
  "top",
  "transform",
  "visibility",
  "width",
  "z-index",
]);

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
    container.style.cssText =
      "position: fixed; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 9998;";
    document.body.appendChild(container);
  }
  return container;
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
