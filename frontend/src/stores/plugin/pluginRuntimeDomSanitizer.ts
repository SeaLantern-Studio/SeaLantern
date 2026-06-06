import DOMPurify from "dompurify";
import { pluginLogger } from "@stores/plugin/pluginLogger";

const ALLOWED_RUNTIME_DECLARATIONS = new Set([
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

const RUNTIME_CSS_BLOCKLIST = [
  /\b(position|z-index|top|right|bottom|left|inset|pointer-events)\s*:[^;]+;?/gi,
  /\b(content|cursor|backdrop-filter|filter|clip-path)\s*:[^;]+;?/gi,
  /\b(body|html|:root)\b/gi,
  /\b(\*|body\s+\*|html\s+\*|:root\s+\*)\b/gi,
];

const FORBIDDEN_RUNTIME_SELECTOR_PATTERNS = [
  /::?(before|after|backdrop|marker|selection|file-selector-button)/i,
  /\b(body|html|:root)\b/i,
  /\*/,
  />/,
  /~/,
  /\+/,
  /\[/,
];

export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    FORBID_TAGS: [
      "script",
      "iframe",
      "object",
      "embed",
      "form",
      "link",
      "meta",
      "style",
      "dialog",
      "header",
      "footer",
      "nav",
      "aside",
      "button",
      "input",
      "select",
      "textarea",
      "label",
      "img",
      "svg",
      "canvas",
      "video",
      "audio",
    ],
    FORBID_ATTR: ["style"],
    ALLOW_DATA_ATTR: false,
  });
}

function filterRuntimeDeclarations(body: string): string {
  const declarations = body
    .split(";")
    .map((declaration) => declaration.trim())
    .filter(Boolean);

  const allowed: string[] = [];
  for (const declaration of declarations) {
    const separatorIndex = declaration.indexOf(":");
    if (separatorIndex === -1) {
      continue;
    }

    const property = declaration.slice(0, separatorIndex).trim().toLowerCase();
    const value = declaration.slice(separatorIndex + 1).trim();
    if (!property || !value) {
      continue;
    }

    if (!ALLOWED_RUNTIME_DECLARATIONS.has(property) && !property.startsWith("--")) {
      continue;
    }

    allowed.push(`${property}: ${value}`);
  }

  return allowed.join("; ");
}

function isAllowedRuntimeSelector(selector: string): boolean {
  const normalized = selector.replace(/\s+/g, " ").trim().toLowerCase();
  if (!normalized) {
    return false;
  }

  for (const pattern of FORBIDDEN_RUNTIME_SELECTOR_PATTERNS) {
    if (pattern.test(normalized)) {
      return false;
    }
  }

  if (normalized.startsWith("#") || normalized.includes(" #")) {
    return false;
  }

  return true;
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
  for (const pattern of RUNTIME_CSS_BLOCKLIST) {
    sanitized = sanitized.replace(pattern, "");
  }
  return sanitized;
}

export function scopeRuntimeCss(css: string, scopeSelector: string): string {
  const sanitized = sanitizeCss(css);
  const output: string[] = [];
  const blocks = sanitized.split("}");

  for (const rawBlock of blocks) {
    const block = rawBlock.trim();
    if (!block) {
      continue;
    }

    const separatorIndex = block.indexOf("{");
    if (separatorIndex === -1) {
      continue;
    }

    const selectorPart = block.slice(0, separatorIndex).trim();
    const bodyPart = block.slice(separatorIndex + 1).trim();
    if (!selectorPart || !bodyPart) {
      continue;
    }

    if (selectorPart.startsWith("@")) {
      continue;
    }

    const filteredBody = filterRuntimeDeclarations(bodyPart);
    if (!filteredBody) {
      continue;
    }

    const scopedSelectors = selectorPart
      .split(",")
      .map((selector) => selector.trim())
      .filter(Boolean)
      .filter((selector) => isAllowedRuntimeSelector(selector))
      .map((selector) => {
        if (selector === ":scope") {
          return scopeSelector;
        }
        if (selector.startsWith(scopeSelector)) {
          return selector;
        }
        return `${scopeSelector} ${selector}`;
      });

    if (scopedSelectors.length === 0) {
      continue;
    }

    output.push(`${scopedSelectors.join(", ")} { ${filteredBody} }`);
  }

  return output.join("\n");
}

export function executePluginScripts(container: HTMLElement, rawHtml: string) {
  const documentFragment = document.createElement("template");
  documentFragment.innerHTML = rawHtml;
  const scripts = Array.from(documentFragment.content.querySelectorAll("script"));

  for (const script of scripts) {
    const executable = document.createElement("script");
    for (const attribute of Array.from(script.attributes)) {
      executable.setAttribute(attribute.name, attribute.value);
    }
    executable.textContent = script.textContent || "";
    container.appendChild(executable);
    executable.remove();
  }

  if (scripts.length > 0) {
    pluginLogger.info("RuntimeUI", "已执行插件脚本注入", {
      containerId: container.id,
      count: scripts.length,
    });
  }
}
