import DOMPurify from "dompurify";
import { pluginLogger } from "@stores/plugin/pluginLogger";

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
  sanitized = sanitized.replace(/\b(position|z-index)\s*:[^;]+;?/gi, "");
  return sanitized;
}

export function executePluginScripts(container: HTMLElement, rawHtml: string) {
  if (/<script\b/i.test(rawHtml)) {
    pluginLogger.warn("RuntimeUI", "已拦截插件脚本注入", {
      containerId: container.id,
    });
  }
}
